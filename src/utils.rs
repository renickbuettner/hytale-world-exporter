use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::models::{BackupInfo, LogInfo};

/// Detects the system locale and returns the language code
pub fn detect_system_locale() -> String {
    sys_locale::get_locale()
        .map(|locale| {
            // Extract language code (e.g., "de-DE" -> "de", "en-US" -> "en")
            locale.split('-').next().unwrap_or("en").to_string()
        })
        .unwrap_or_else(|| "en".to_string())
}

/// Formats bytes into human-readable size string
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Calculates the total size of a directory recursively
pub fn get_directory_size(path: &PathBuf) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Gets the last played timestamp from log files
pub fn get_last_played(world_path: &PathBuf) -> Option<String> {
    let logs_path = world_path.join("logs");
    if !logs_path.exists() {
        return None;
    }

    let mut logs: Vec<_> = fs::read_dir(&logs_path)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() &&
            entry.path().extension().map_or(false, |ext| ext == "log")
        })
        .collect();

    // Sort by filename descending (newest first based on timestamp in filename)
    logs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    logs.first().and_then(|entry| {
        let name = entry.file_name().to_str()?.to_string();
        // Parse timestamp from filename like "2026-01-13_19-35-06_server.log"
        if name.len() >= 19 {
            let date_part = &name[0..10]; // "2026-01-13"
            let time_part = &name[11..19]; // "19-35-06"
            let time_formatted = time_part.replace('-', ":");
            Some(format!("{} {}", date_part, time_formatted))
        } else {
            None
        }
    })
}

/// Gets backup files for a world, filtering out system files
pub fn get_world_backups(world_path: &PathBuf) -> Vec<BackupInfo> {
    let backup_path = world_path.join("backup");
    if !backup_path.exists() {
        return Vec::new();
    }

    fs::read_dir(&backup_path)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .filter(|entry| {
                    // Filter out system files
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    !name_str.starts_with(".DS_Store") &&
                    !name_str.starts_with("Thumbs.db") &&
                    !name_str.starts_with("desktop.ini") &&
                    !name_str.starts_with("._")
                })
                .filter_map(|entry| {
                    let name = entry.file_name().to_str()?.to_string();
                    let path = entry.path();
                    let size = entry.metadata().ok()?.len();
                    Some(BackupInfo { name, path, size })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Gets the latest log file for a world
pub fn get_latest_log(world_path: &PathBuf) -> Option<LogInfo> {
    let logs_path = world_path.join("logs");
    if !logs_path.exists() {
        return None;
    }

    let mut logs: Vec<_> = fs::read_dir(&logs_path)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() &&
            entry.path().extension().map_or(false, |ext| ext == "log")
        })
        .collect();

    // Sort by filename descending (newest first based on timestamp in filename)
    logs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    logs.first().and_then(|entry| {
        let name = entry.file_name().to_str()?.to_string();
        let path = entry.path();
        let content = fs::read_to_string(&path).unwrap_or_else(|_| String::from("Could not read log file"));
        Some(LogInfo { name, path, content })
    })
}

/// Opens a file in the system file manager
pub fn open_file_in_finder(path: &PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn();
    }
}

