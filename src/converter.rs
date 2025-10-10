use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

static FFMPEG_PATH: OnceLock<PathBuf> = OnceLock::new();
static FFPROBE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub struct AudioConverter;

impl AudioConverter {
    pub fn new() -> Self {
        // Initialize embedded FFmpeg/FFprobe on first use
        Self::ensure_tools_extracted();
        Self
    }

    /// Ensure embedded FFmpeg and FFprobe are extracted to temp directory
    fn ensure_tools_extracted() {
        FFMPEG_PATH.get_or_init(|| {
            Self::extract_tool("ffmpeg.exe", include_bytes!("../ffmpeg.exe")).unwrap_or_else(|e| {
                eprintln!("Failed to extract ffmpeg.exe: {}", e);
                PathBuf::from("ffmpeg.exe")
            })
        });

        FFPROBE_PATH.get_or_init(|| {
            Self::extract_tool("ffprobe.exe", include_bytes!("../ffprobe.exe")).unwrap_or_else(
                |e| {
                    eprintln!("Failed to extract ffprobe.exe: {}", e);
                    PathBuf::from("ffprobe.exe")
                },
            )
        });
    }

    /// Extract a single tool to temp directory
    fn extract_tool(name: &str, data: &[u8]) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join("audio_converter_tools");
        fs::create_dir_all(&temp_dir)?;

        let tool_path = temp_dir.join(name);
        if !tool_path.exists() || fs::metadata(&tool_path)?.len() != data.len() as u64 {
            fs::write(&tool_path, data)?;
        }

        Ok(tool_path)
    }

    fn get_ffmpeg_path(&self) -> Result<&Path> {
        FFMPEG_PATH
            .get()
            .map(|p| p.as_path())
            .ok_or_else(|| anyhow!("FFmpeg not initialized"))
    }

    fn get_ffprobe_path(&self) -> Result<&Path> {
        FFPROBE_PATH
            .get()
            .map(|p| p.as_path())
            .ok_or_else(|| anyhow!("FFprobe not initialized"))
    }

    /// 获取音频时长(秒)
    fn get_duration(&self, path: &Path) -> Result<f64> {
        let ffprobe_path = self.get_ffprobe_path()?;
        let mut cmd = Command::new(ffprobe_path);
        cmd.args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_str().unwrap(),
        ]);
        
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let output = cmd.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "FFprobe failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = duration_str
            .trim()
            .parse()
            .map_err(|_| anyhow!("Failed to parse duration"))?;

        Ok(duration)
    }

    /// 转换音频文件
    pub fn convert_audio(&self, input: &Path, output: &Path, min_duration: f32) -> Result<()> {
        // 获取原始时长
        let duration = self.get_duration(input)?;
        let min_dur = min_duration as f64;

        if duration >= min_dur {
            // 时长足够,直接转换
            self.convert_simple(input, output)?;
        } else {
            // 时长不足,需要循环拼接
            let loop_count = (min_dur / duration).ceil() as usize;
            self.convert_with_loop(input, output, loop_count)?;
        }

        Ok(())
    }

    /// 简单转换(不循环)
    fn convert_simple(&self, input: &Path, output: &Path) -> Result<()> {
        let ffmpeg_path = self.get_ffmpeg_path()?;
        let mut cmd = Command::new(ffmpeg_path);
        cmd.args([
            "-i",
            input.to_str().unwrap(),
            "-ar",
            "44100",
            "-ac",
            "2",
            "-y",
            output.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
        
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let status = cmd.status()?;

        if !status.success() {
            return Err(anyhow!("FFmpeg conversion failed"));
        }

        Ok(())
    }

    /// 带循环的转换
    fn convert_with_loop(&self, input: &Path, output: &Path, loop_count: usize) -> Result<()> {
        let ffmpeg_path = self.get_ffmpeg_path()?;
        let single = input.to_str().unwrap();
        let concat_str = (0..loop_count)
            .map(|_| single)
            .collect::<Vec<_>>()
            .join("|");
        let concat_input = format!("concat:{}", concat_str);

        let mut cmd = Command::new(ffmpeg_path);
        cmd.args([
            "-i",
            &concat_input,
            "-ar",
            "44100",
            "-ac",
            "2",
            "-y",
            output.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
        
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let status = cmd.status()?;

        if !status.success() {
            // 如果 concat 失败,尝试使用 amovie + loop
            self.convert_with_filter(input, output, loop_count)?;
        }

        Ok(())
    }

    /// 使用 filter 进行循环拼接
    fn convert_with_filter(&self, input: &Path, output: &Path, loop_count: usize) -> Result<()> {
        let ffmpeg_path = self.get_ffmpeg_path()?;
        let filter = format!("amovie={}:loop={}", input.to_str().unwrap(), loop_count);

        let mut cmd = Command::new(ffmpeg_path);
        cmd.args([
            "-filter_complex",
            &filter,
            "-ar",
            "44100",
            "-ac",
            "2",
            "-y",
            output.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
        
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let status = cmd.status()?;

        if !status.success() {
            return Err(anyhow!("FFmpeg conversion with filter failed"));
        }

        Ok(())
    }
}
