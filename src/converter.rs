use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct AudioConverter {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
}

impl AudioConverter {
    pub fn new() -> Self {
        // Try to prefer ffmpeg/ffprobe located next to the running executable.
        // Fall back to the system names (rely on PATH).
        let mut ffmpeg_name = "ffmpeg".to_string();
        let mut ffprobe_name = "ffprobe".to_string();

        #[cfg(target_os = "windows")]
        {
            ffmpeg_name.push_str(".exe");
            ffprobe_name.push_str(".exe");
        }

        let (ffmpeg_path, ffprobe_path) = match std::env::current_exe().ok().and_then(|exe| exe.parent().map(|p| p.to_path_buf())) {
            Some(mut dir) => {
                let candidate_ffmpeg = dir.join(&ffmpeg_name);
                let candidate_ffprobe = dir.join(&ffprobe_name);
                let ffmpeg_path = if candidate_ffmpeg.exists() { candidate_ffmpeg } else { PathBuf::from(&ffmpeg_name) };
                let ffprobe_path = if candidate_ffprobe.exists() { candidate_ffprobe } else { PathBuf::from(&ffprobe_name) };
                (ffmpeg_path, ffprobe_path)
            }
            None => (PathBuf::from(&ffmpeg_name), PathBuf::from(&ffprobe_name)),
        };

        Self {
            ffmpeg_path,
            ffprobe_path,
        }
    }

    /// 获取音频时长(秒)
    fn get_duration(&self, path: &Path) -> Result<f64> {
        let output = Command::new(&self.ffprobe_path)
            .args(&[
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                path.to_str().unwrap(),
            ])
            .output()?;

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
        // 检查 FFmpeg/FFprobe 是否可用
        if !self.check_ffmpeg() {
            return Err(anyhow!(
                "FFmpeg/FFprobe not found. Please install them or place them next to the program."
            ));
        }

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
        let status = Command::new(&self.ffmpeg_path)
            .args(&[
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
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            return Err(anyhow!("FFmpeg conversion failed"));
        }

        Ok(())
    }

    /// 带循环的转换
    fn convert_with_loop(&self, input: &Path, output: &Path, loop_count: usize) -> Result<()> {
        // 使用 FFmpeg 的 concat demuxer 或者 amovie filter 作为回退
        // Build a concat protocol string like: concat:in|in|in
        let single = input.to_str().unwrap();
        let concat_str = (0..loop_count)
            .map(|_| single)
            .collect::<Vec<_>>()
            .join("|");
        let concat_input = format!("concat:{}", concat_str);

        let status = Command::new(&self.ffmpeg_path)
            .args(&[
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
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            // 如果 concat 失败,尝试使用 amovie + loop
            self.convert_with_filter(input, output, loop_count)?;
        }

        Ok(())
    }

    /// 使用 filter 进行循环拼接
    fn convert_with_filter(&self, input: &Path, output: &Path, loop_count: usize) -> Result<()> {
        // Use amovie with loop; note: some ffmpeg builds may require a different filter chain.
        // loop_count here is number of repetitions, pass loop_count-1 to amovie's loop param if needed.
        let filter = format!("amovie={}:loop={}", input.to_str().unwrap(), loop_count);

        let status = Command::new(&self.ffmpeg_path)
            .args(&[
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
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            return Err(anyhow!("FFmpeg conversion with filter failed"));
        }

        Ok(())
    }

    /// 检查 FFmpeg/FFprobe 是否可用
    fn check_ffmpeg(&self) -> bool {
        let ffmpeg_ok = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let ffprobe_ok = Command::new(&self.ffprobe_path)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        ffmpeg_ok && ffprobe_ok
    }
}
