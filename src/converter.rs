use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct AudioConverter;

impl AudioConverter {
    pub fn new() -> Self {
        Self
    }

    /// 获取音频时长(秒)
    fn get_duration(&self, path: &Path) -> Result<f64> {
        let output = Command::new("ffprobe")
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
        // 检查 FFmpeg 是否可用
        if !self.check_ffmpeg() {
            return Err(anyhow!(
                "FFmpeg not found. Please install FFmpeg and add it to PATH."
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
        let status = Command::new("ffmpeg")
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
        // 使用 FFmpeg 的 concat demuxer
        let concat_str = format!("concat:{}|", input.to_str().unwrap()).repeat(loop_count);
        let concat_str = concat_str.trim_end_matches('|');

        let status = Command::new("ffmpeg")
            .args(&[
                "-i",
                &concat_str,
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
            // 如果 concat 失败,尝试使用 amovie filter
            self.convert_with_filter(input, output, loop_count)?;
        }

        Ok(())
    }

    /// 使用 filter 进行循环拼接
    fn convert_with_filter(&self, input: &Path, output: &Path, loop_count: usize) -> Result<()> {
        let filter = format!("amovie={}:loop={}", input.to_str().unwrap(), loop_count);

        let status = Command::new("ffmpeg")
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

    /// 检查 FFmpeg 是否可用
    fn check_ffmpeg(&self) -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}
