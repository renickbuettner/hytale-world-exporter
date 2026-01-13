use eframe::egui;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");

fn detect_system_locale() -> String {
    sys_locale::get_locale()
        .map(|locale| {
            // Extract language code (e.g., "de-DE" -> "de", "en-US" -> "en")
            locale.split('-').next().unwrap_or("en").to_string()
        })
        .unwrap_or_else(|| "en".to_string())
}

fn main() -> Result<(), eframe::Error> {
    // Set locale based on system language
    let locale = detect_system_locale();
    rust_i18n::set_locale(&locale);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 680.0])
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
    worlds: Vec<String>,
    selected_world: Option<usize>,
    progress: Arc<Mutex<BackupProgress>>,
}

impl HytaleBackupApp {
    fn new() -> Self {
        let worlds = Self::load_worlds();
        Self {
            status_message: String::new(),
            worlds,
            selected_world: None,
            progress: Arc::new(Mutex::new(BackupProgress::default())),
        }
    }

    fn load_worlds() -> Vec<String> {
        match get_hytale_worlds_path() {
            Ok(path) => {
                if path.exists() {
                    fs::read_dir(&path)
                        .map(|entries| {
                            entries
                                .filter_map(|entry| entry.ok())
                                .filter(|entry| entry.path().is_dir())
                                .filter_map(|entry| {
                                    entry.file_name().to_str().map(|s| s.to_string())
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            
            ui.vertical_centered(|ui| {
                ui.heading(t!("app.title"));
            });
            
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.label(t!("app.available_worlds"));
                if ui.button(t!("app.refresh")).clicked() {
                    self.refresh_worlds();
                }
            });

            ui.add_space(10.0);

            if self.worlds.is_empty() {
                ui.label(t!("app.no_worlds_found"));
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (index, world_name) in self.worlds.iter().enumerate() {
                            let is_selected = self.selected_world == Some(index);
                            if ui.selectable_label(is_selected, format!("🌍 {}", world_name)).clicked() {
                                self.selected_world = Some(index);
                            }
                        }
                    });
            }

            ui.add_space(20.0);
            
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

                ui.vertical_centered(|ui| {
                    let button_enabled = self.selected_world.is_some();
                    if ui.add_enabled(button_enabled, egui::Button::new(t!("app.compress_world"))).clicked() {
                        if let Some(index) = self.selected_world {
                            let world_name = self.worlds[index].clone();

                            // Create default filename with timestamp
                            let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
                            let default_filename = format!("{}_{}.zip", world_name, timestamp);

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
                                    let result = backup_world_to_path_with_progress(&world_name, &save_path, &progress, &ctx);

                                    let mut p = progress.lock().unwrap();
                                    p.is_running = false;
                                    p.result = Some(result);
                                    ctx.request_repaint();
                                });
                            }
                        }
                    }
                });
            }

            ui.add_space(20.0);
            
            if !self.status_message.is_empty() {
                ui.separator();
                ui.add_space(10.0);
                ui.label(&self.status_message);
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
    progress: &Arc<Mutex<BackupProgress>>,
    ctx: &egui::Context,
) -> Result<String, String> {
    // Get the worlds directory
    let worlds_path = get_hytale_worlds_path()?;
    let world_path = worlds_path.join(world_name);

    if !world_path.exists() {
        return Err(t!("errors.world_not_found", name = world_name).to_string());
    }

    // Count total files first
    let total_files: usize = WalkDir::new(&world_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
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
            // Add directory to ZIP
            zip.add_directory(name.to_string_lossy().to_string(), options)
                .map_err(|e| t!("errors.add_dir_failed", error = e.to_string()).to_string())?;
        }
    }

    zip.finish()
        .map_err(|e| t!("errors.finish_zip_failed", error = e.to_string()).to_string())?;

    Ok(zip_path.to_string_lossy().to_string())
}
