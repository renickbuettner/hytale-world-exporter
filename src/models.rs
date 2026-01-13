use std::path::PathBuf;

#[derive(Clone)]
pub struct WorldInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub last_played: Option<String>,
}

#[derive(Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Clone)]
pub struct LogInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Clone)]
pub struct BackupProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub is_running: bool,
    pub result: Option<Result<String, String>>,
}

impl Default for BackupProgress {
    fn default() -> Self {
        Self {
            current: 0,
            total: 0,
            current_file: String::new(),
            is_running: false,
            result: None,
        }
    }
}

