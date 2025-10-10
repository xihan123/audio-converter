use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "aac", "m4a", "opus", "wma", "ape", "alac", "aiff", "wav",
];

pub fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        AUDIO_EXTENSIONS.contains(&ext_str.as_str())
    } else {
        false
    }
}

pub fn is_wav_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        ext.to_string_lossy().to_lowercase() == "wav"
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn get_output_path(input_path: &Path) -> PathBuf {
    let mut output = input_path.to_path_buf();
    output.set_extension("wav");
    output
}

pub fn find_audio_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_audio_file(p))
        .collect()
}

/// Generate a path for the output WAV placed inside `output_dir`, avoiding overwrite by
/// adding a numeric suffix if needed. Creates `output_dir` if it doesn't exist.
pub fn generate_unique_output_path(input_path: &Path, output_dir: &Path) -> PathBuf {
    if fs::create_dir_all(output_dir).is_err() {
        // If creation fails, fall back to current directory
    }

    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());

    let mut candidate = output_dir.join(format!("{}.wav", stem));
    let mut idx = 1;

    while candidate.exists() {
        candidate = output_dir.join(format!("{} ({}){}.wav", stem, idx, ""));
        idx += 1;
    }

    candidate
}
