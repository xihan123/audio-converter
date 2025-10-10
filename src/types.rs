use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FileTask {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub status: TaskStatus,
    pub progress: f32,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}
