use eframe::egui;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");

const TAB_CONTENT_MAX_HEIGHT: f32 = 260.0;

fn detect_system_locale() -> String {
    sys_locale::get_locale()
        .map(|locale| {
            // Extract language code (e.g., "de-DE" -> "de", "en-US" -> "en")
            locale.split('-').next().unwrap_or("en").to_string()
        })
        .unwrap_or_else(|| "en".to_string())
}

fn format_size(bytes: u64) -> String {
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

fn get_directory_size(path: &PathBuf) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}


fn get_last_played(world_path: &PathBuf) -> Option<String> {
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

#[derive(Clone)]
struct WorldInfo {
    name: String,
    path: PathBuf,
    size: u64,
    last_played: Option<String>,
}

#[derive(Clone)]
struct BackupInfo {
    name: String,
    path: PathBuf,
    size: u64,
}

#[derive(Clone)]
struct LogInfo {
    name: String,
    path: PathBuf,
    content: String,
}

fn get_world_backups(world_path: &PathBuf) -> Vec<BackupInfo> {
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

fn get_latest_log(world_path: &PathBuf) -> Option<LogInfo> {
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

fn open_file_in_finder(path: &PathBuf) {
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

fn main() -> Result<(), eframe::Error> {
    // Set locale based on system language
    let locale = detect_system_locale();
    rust_i18n::set_locale(&locale);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 660.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        &t!("app.title"),
        options,
        Box::new(|_cc| Ok(Box::new(HytaleBackupApp::new()))),
    )
}

#[derive(Clone)]
struct BackupProgress {
    current: usize,
    total: usize,
    current_file: String,
    is_running: bool,
    result: Option<Result<String, String>>,
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

struct HytaleBackupApp {
    status_message: String,
    worlds: Vec<WorldInfo>,
    selected_world: Option<usize>,
    selected_tab: usize,
    include_logs: bool,
    include_backups: bool,
    progress: Arc<Mutex<BackupProgress>>,
    pending_delete_backup: Option<PathBuf>,
    pending_import: Option<(PathBuf, String)>, // (zip_path, world_name)
}

impl HytaleBackupApp {
    fn new() -> Self {
        let worlds = Self::load_worlds();
        Self {
            status_message: String::new(),
            worlds,
            selected_world: None,
            selected_tab: 0,
            include_logs: true,
            include_backups: true,
            progress: Arc::new(Mutex::new(BackupProgress::default())),
            pending_delete_backup: None,
            pending_import: None,
        }
    }

    fn load_worlds() -> Vec<WorldInfo> {
        match get_hytale_worlds_path() {
            Ok(base_path) => {
                if base_path.exists() {
                    fs::read_dir(&base_path)
                        .map(|entries| {
                            entries
                                .filter_map(|entry| entry.ok())
                                .filter(|entry| entry.path().is_dir())
                                .filter_map(|entry| {
                                    let name = entry.file_name().to_str()?.to_string();
                                    let path = entry.path();
                                    let size = get_directory_size(&path);
                                    let last_played = get_last_played(&path);
                                    Some(WorldInfo {
                                        name,
                                        path,
                                        size,
                                        last_played,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            Err(_) => Vec::new(),
        }
    }

    fn refresh_worlds(&mut self) {
        self.worlds = Self::load_worlds();
        self.selected_world = None;
    }
}

impl eframe::App for HytaleBackupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Delete confirmation dialog
        if let Some(backup_path) = self.pending_delete_backup.clone() {
            egui::Window::new(t!("app.confirm_delete_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(t!("app.confirm_delete_message"));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(backup_path.file_name().unwrap_or_default().to_string_lossy().to_string()).strong());
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button(t!("app.cancel")).clicked() {
                            self.pending_delete_backup = None;
                        }

                        if ui.button(egui::RichText::new(t!("app.delete")).color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                            if let Err(e) = fs::remove_file(&backup_path) {
                                self.status_message = format!("{} {}", t!("app.error"), e);
                            } else {
                                self.status_message = t!("app.backup_deleted").to_string();
                            }
                            self.pending_delete_backup = None;
                        }
                    });
                });
        }

        // Import confirmation dialog
        if let Some((zip_path, world_name)) = self.pending_import.clone() {
            egui::Window::new(t!("app.confirm_import_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(t!("app.confirm_import_message"));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(&world_name).strong().size(16.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(t!("app.confirm_import_warning")).color(egui::Color32::from_rgb(255, 180, 100)));
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button(t!("app.cancel")).clicked() {
                            self.pending_import = None;
                        }

                        if ui.button(egui::RichText::new(t!("app.import")).color(egui::Color32::from_rgb(100, 200, 100))).clicked() {
                            match import_world(&zip_path, &world_name) {
                                Ok(_) => {
                                    self.status_message = t!("app.import_success").to_string();
                                    self.refresh_worlds();
                                }
                                Err(e) => {
                                    self.status_message = format!("{} {}", t!("app.error"), e);
                                }
                            }
                            self.pending_import = None;
                        }
                    });
                });
        }

        // Bottom toolbar
        egui::TopBottomPanel::bottom("toolbar").show(ctx, |ui| {
            ui.add_space(10.0);

            // Check progress status
            let progress_state = self.progress.lock().unwrap().clone();

            if progress_state.is_running {
                // Show progress bar while backup is running
                ui.vertical_centered(|ui| {
                    ui.label(t!("app.compressing"));

                    let progress_fraction = if progress_state.total > 0 {
                        progress_state.current as f32 / progress_state.total as f32
                    } else {
                        0.0
                    };

                    ui.add(egui::ProgressBar::new(progress_fraction)
                        .show_percentage()
                        .animate(true));

                    ui.label(format!("{} / {}", progress_state.current, progress_state.total));

                    if !progress_state.current_file.is_empty() {
                        ui.label(&progress_state.current_file);
                    }
                });

                // Request repaint to update progress
                ctx.request_repaint();
            } else {
                // Check if there's a result from a completed backup
                if let Some(result) = progress_state.result.clone() {
                    self.status_message = match result {
                        Ok(path) => format!("{}\n{}", t!("app.backup_success"), path),
                        Err(e) => format!("{} {}", t!("app.error"), e),
                    };
                    // Clear the result
                    self.progress.lock().unwrap().result = None;
                }

                ui.horizontal(|ui| {
                    // Checkboxes for including logs and backups
                    ui.checkbox(&mut self.include_logs, t!("app.include_logs"));
                    ui.add_space(20.0);
                    ui.checkbox(&mut self.include_backups, t!("app.include_backups"));

                    // Button aligned to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button_enabled = self.selected_world.is_some();
                        if ui.add_enabled(button_enabled, egui::Button::new(t!("app.compress_world"))).clicked() {
                            if let Some(index) = self.selected_world {
                                let world = self.worlds[index].clone();
                                let include_logs = self.include_logs;
                                let include_backups = self.include_backups;

                                // Create default filename with timestamp
                                let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
                                let default_filename = format!("{}_{}.zip", world.name, timestamp);

                                // Show save file dialog
                                let file_dialog = rfd::FileDialog::new()
                                    .set_file_name(&default_filename)
                                    .add_filter("ZIP", &["zip"]);

                                // Set default directory to Downloads if available
                                let file_dialog = if let Some(downloads) = dirs::download_dir() {
                                    file_dialog.set_directory(&downloads)
                                } else {
                                    file_dialog
                                };

                                if let Some(save_path) = file_dialog.save_file() {
                                    // Start backup in background thread
                                    let progress = Arc::clone(&self.progress);
                                    let ctx = ctx.clone();

                                    // Reset progress
                                    {
                                        let mut p = progress.lock().unwrap();
                                        p.is_running = true;
                                        p.current = 0;
                                        p.total = 0;
                                        p.current_file = String::new();
                                        p.result = None;
                                    }

                                    thread::spawn(move || {
                                        let result = backup_world_to_path_with_progress(
                                            &world.name,
                                            &save_path,
                                            include_logs,
                                            include_backups,
                                            &progress,
                                            &ctx
                                        );

                                        let mut p = progress.lock().unwrap();
                                        p.is_running = false;
                                        p.result = Some(result);
                                        ctx.request_repaint();
                                    });
                                }
                            }
                        }
                    });
                });
            }

            // Show status message in toolbar if present
            if !self.status_message.is_empty() {
                ui.add_space(5.0);
                ui.separator();
                ui.label(&self.status_message);
            }

            ui.add_space(10.0);
        });

        // Main content panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                ui.heading(t!("app.title"));
            });

            ui.add_space(20.0);

            // World list section
            ui.horizontal(|ui| {
                ui.label(t!("app.available_worlds"));
                if ui.button(t!("app.refresh")).clicked() {
                    self.refresh_worlds();
                }
                if ui.button(t!("app.import_world")).clicked() {
                    // Show file dialog to select a ZIP file
                    let file_dialog = rfd::FileDialog::new()
                        .add_filter("ZIP", &["zip"]);

                    if let Some(zip_path) = file_dialog.pick_file() {
                        // Check if it's a ZIP file
                        if zip_path.extension().map_or(false, |ext| ext == "zip") {
                            // Extract world name from filename (remove date suffix)
                            // Format: WorldName_2026-01-13_19-35-06.zip -> WorldName
                            if let Some(filename) = zip_path.file_stem() {
                                let filename_str = filename.to_string_lossy().to_string();
                                // Remove timestamp suffix (last 20 characters: _YYYY-MM-DD_HH-MM-SS)
                                let world_name = if filename_str.len() > 20 && filename_str.chars().rev().nth(19) == Some('_') {
                                    filename_str[..filename_str.len() - 20].to_string()
                                } else {
                                    filename_str
                                };
                                self.pending_import = Some((zip_path, world_name));
                            }
                        } else {
                            self.status_message = t!("app.error_not_zip").to_string();
                        }
                    }
                }
            });

            ui.add_space(10.0);

            if self.worlds.is_empty() {
                ui.label(t!("app.no_worlds_found"));
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("worlds_list")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for (index, world) in self.worlds.iter().enumerate() {
                            let is_selected = self.selected_world == Some(index);
                            if ui.selectable_label(is_selected, format!("🌍 {}", world.name)).clicked() {
                                self.selected_world = Some(index);
                            }
                        }
                    });
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Details section
            ui.label(t!("app.details"));
            ui.add_space(10.0);

            if let Some(index) = self.selected_world {
                if let Some(world) = self.worlds.get(index) {
                    let available_width = ui.available_width();
                    egui::Frame::group(ui.style())
                        .inner_margin(10.0)
                        .rounding(5.0)
                        .show(ui, |ui| {
                            ui.set_width(available_width - 20.0);
                            ui.label(egui::RichText::new(&world.name).strong().size(16.0));
                            ui.add_space(10.0);

                            egui::Grid::new("world_details")
                                .num_columns(2)
                                .spacing([10.0, 5.0])
                                .show(ui, |ui| {
                                    ui.label(t!("app.detail_size"));
                                    ui.label(format_size(world.size));
                                    ui.end_row();

                                    ui.label(t!("app.detail_last_played"));
                                    ui.label(world.last_played.clone().unwrap_or_else(|| t!("app.unknown").to_string()));
                                    ui.end_row();

                                    ui.label(t!("app.detail_path"));
                                    ui.label(egui::RichText::new(world.path.to_string_lossy().to_string()).small().weak());
                                    ui.end_row();
                                });
                        });

                    ui.add_space(15.0);

                    // Tab navigation
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.selected_tab == 0, t!("app.tab_backups")).clicked() {
                            self.selected_tab = 0;
                        }
                        ui.separator();
                        if ui.selectable_label(self.selected_tab == 1, t!("app.tab_logs")).clicked() {
                            self.selected_tab = 1;
                        }
                    });

                    ui.add_space(10.0);

                    // Tab content
                    let world_path = world.path.clone();

                    match self.selected_tab {
                        0 => {
                            // Backups tab
                            let backups = get_world_backups(&world_path);

                            if backups.is_empty() {
                                ui.label(t!("app.no_backups_found"));
                            } else {
                                egui::ScrollArea::vertical()
                                    .id_salt("backups_list")
                                    .max_height(TAB_CONTENT_MAX_HEIGHT)
                                    .show(ui, |ui| {
                                        for backup in &backups {
                                            egui::Frame::group(ui.style())
                                                .inner_margin(5.0)
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.vertical(|ui| {
                                                            ui.label(egui::RichText::new(&backup.name).strong());
                                                            ui.label(egui::RichText::new(format_size(backup.size)).weak());
                                                        });

                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            if ui.button("🗑").on_hover_text(t!("app.delete_backup")).clicked() {
                                                                self.pending_delete_backup = Some(backup.path.clone());
                                                            }
                                                            if ui.button("📂").on_hover_text(t!("app.open_in_finder")).clicked() {
                                                                open_file_in_finder(&backup.path);
                                                            }
                                                        });
                                                    });
                                                });
                                            ui.add_space(5.0);
                                        }
                                    });
                            }
                        },
                        1 => {
                            // Logs tab
                            if let Some(log) = get_latest_log(&world_path) {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&log.name).strong());
                                    if ui.button("📂").on_hover_text(t!("app.open_in_finder")).clicked() {
                                        open_file_in_finder(&log.path);
                                    }
                                });
                                ui.add_space(5.0);

                                egui::ScrollArea::vertical()
                                    .id_salt("logs_content")
                                    .max_height(TAB_CONTENT_MAX_HEIGHT)
                                    .show(ui, |ui| {
                                        for line in log.content.lines() {
                                            let text = if line.contains("ERROR") {
                                                egui::RichText::new(line)
                                                    .monospace()
                                                    .color(egui::Color32::from_rgb(255, 100, 100))
                                            } else if line.contains("WARN") {
                                                egui::RichText::new(line)
                                                    .monospace()
                                                    .color(egui::Color32::from_rgb(255, 180, 100))
                                            } else {
                                                egui::RichText::new(line).monospace()
                                            };
                                            ui.label(text);
                                        }
                                    });
                            } else {
                                ui.label(t!("app.no_logs_found"));
                            }
                        },
                        _ => {}
                    }
                }
            } else {
                ui.label(t!("app.select_world_hint"));
            }
        });
    }
}

fn get_hytale_worlds_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(appdata).join("Hytale").join("worlds"));
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



fn backup_world_to_path_with_progress(
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

fn import_world(zip_path: &PathBuf, world_name: &str) -> Result<(), String> {
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

