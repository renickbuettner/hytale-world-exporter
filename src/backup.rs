use eframe::egui;
use rust_i18n::t;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::models::BackupProgress;

/// Gets the path to Hytale world saves
pub fn get_hytale_worlds_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(appdata)
                .join("Hytale")
                .join("UserData")
                .join("Saves"));
        }
        Err(t!("errors.appdata_not_found").to_string())
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return Ok(home
                .join("Library")
                .join("Application Support")
                .join("Hytale")
                .join("UserData")
                .join("Saves"));
        }
        Err(t!("errors.home_not_found").to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(t!("errors.platform_not_supported").to_string())
    }
}

/// Backs up a world to a ZIP file with progress tracking
pub fn backup_world_to_path_with_progress(
    world_name: &str,
    zip_path: &PathBuf,
    include_logs: bool,
    include_backups: bool,
    progress: &Arc<Mutex<BackupProgress>>,
    ctx: &egui::Context,
) -> Result<String, String> {
    // Get the worlds directory
    let worlds_path = get_hytale_worlds_path()?;
    let world_path = worlds_path.join(world_name);

    if !world_path.exists() {
        return Err(t!("errors.world_not_found", name = world_name).to_string());
    }

    // Helper function to check if path should be excluded
    let should_exclude = |path: &std::path::Path| -> bool {
        let path_str = path.to_string_lossy();
        if !include_logs && path_str.contains("/logs/") {
            return true;
        }
        if !include_backups && path_str.contains("/backup/") {
            return true;
        }
        false
    };

    // Count total files first (excluding filtered directories)
    let total_files: usize = WalkDir::new(&world_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| !should_exclude(e.path()))
        .count();

    {
        let mut p = progress.lock().unwrap();
        p.total = total_files;
    }

    // Create the ZIP file
    let file = File::create(zip_path)
        .map_err(|e| t!("errors.zip_create_failed", error = e.to_string()).to_string())?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut current_count = 0;

    // Walk through all files in the world directory
    for entry in WalkDir::new(&world_path) {
        let entry = entry.map_err(|e| t!("errors.read_files_failed", error = e.to_string()).to_string())?;
        let path = entry.path();

        // Skip excluded directories
        if should_exclude(path) {
            continue;
        }

        let name = path
            .strip_prefix(&world_path)
            .map_err(|e| t!("errors.process_path_failed", error = e.to_string()).to_string())?;

        // Skip empty directory names
        if name.as_os_str().is_empty() {
            continue;
        }

        if path.is_file() {
            current_count += 1;

            // Update progress
            {
                let mut p = progress.lock().unwrap();
                p.current = current_count;
                p.current_file = name.to_string_lossy().to_string();
            }
            ctx.request_repaint();

            // Add file to ZIP
            zip.start_file(name.to_string_lossy().to_string(), options)
                .map_err(|e| t!("errors.add_file_failed", error = e.to_string()).to_string())?;

            let file_content = fs::read(path)
                .map_err(|e| t!("errors.read_file_failed", error = e.to_string()).to_string())?;

            zip.write_all(&file_content)
                .map_err(|e| t!("errors.write_zip_failed", error = e.to_string()).to_string())?;
        } else if path.is_dir() {
            // Skip excluded directories entirely
            let name_str = name.to_string_lossy();
            if (!include_logs && name_str == "logs") || (!include_backups && name_str == "backup") {
                continue;
            }

            // Add directory to ZIP
            zip.add_directory(name.to_string_lossy().to_string(), options)
                .map_err(|e| t!("errors.add_dir_failed", error = e.to_string()).to_string())?;
        }
    }

    zip.finish()
        .map_err(|e| t!("errors.finish_zip_failed", error = e.to_string()).to_string())?;

    Ok(zip_path.to_string_lossy().to_string())
}

/// Imports a world from a ZIP file
pub fn import_world(zip_path: &PathBuf, world_name: &str) -> Result<(), String> {
    // Get the saves directory
    let saves_path = get_hytale_worlds_path()?;
    let world_path = saves_path.join(world_name);

    // If the world folder exists, delete it first
    if world_path.exists() {
        fs::remove_dir_all(&world_path)
            .map_err(|e| t!("errors.delete_world_failed", error = e.to_string()).to_string())?;
    }

    // Create the world directory
    fs::create_dir_all(&world_path)
        .map_err(|e| t!("errors.create_dir_failed", error = e.to_string()).to_string())?;

    // Open the ZIP file
    let file = File::open(zip_path)
        .map_err(|e| t!("errors.open_zip_failed", error = e.to_string()).to_string())?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| t!("errors.read_zip_failed", error = e.to_string()).to_string())?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| t!("errors.read_zip_entry_failed", error = e.to_string()).to_string())?;

        let outpath = match file.enclosed_name() {
            Some(path) => world_path.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            // Create directory
            fs::create_dir_all(&outpath)
                .map_err(|e| t!("errors.create_dir_failed", error = e.to_string()).to_string())?;
        } else {
            // Create parent directories if needed
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| t!("errors.create_dir_failed", error = e.to_string()).to_string())?;
                }
            }

            // Extract file
            let mut outfile = File::create(&outpath)
                .map_err(|e| t!("errors.create_file_failed", error = e.to_string()).to_string())?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| t!("errors.read_zip_entry_failed", error = e.to_string()).to_string())?;

            outfile.write_all(&buffer)
                .map_err(|e| t!("errors.write_file_failed", error = e.to_string()).to_string())?;
        }
    }

    Ok(())
}

