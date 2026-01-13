use eframe::egui;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 680.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Hytale Backup",
        options,
        Box::new(|_cc| Ok(Box::new(HytaleBackupApp::new()))),
    )
}

struct HytaleBackupApp {
    status_message: String,
    worlds: Vec<String>,
    selected_world: Option<usize>,
}

impl HytaleBackupApp {
    fn new() -> Self {
        let worlds = Self::load_worlds();
        Self {
            status_message: String::new(),
            worlds,
            selected_world: None,
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
                ui.heading("Hytale Welten Backup");
            });
            
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.label("Verfügbare Welten:");
                if ui.button("🔄 Aktualisieren").clicked() {
                    self.refresh_worlds();
                }
            });

            ui.add_space(10.0);

            if self.worlds.is_empty() {
                ui.label("Keine Welten gefunden.");
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
            
            ui.vertical_centered(|ui| {
                let button_enabled = self.selected_world.is_some();
                if ui.add_enabled(button_enabled, egui::Button::new("🗜️ Welt komprimieren")).clicked() {
                    if let Some(index) = self.selected_world {
                        let world_name = &self.worlds[index];
                        self.status_message = match backup_world(world_name) {
                            Ok(path) => format!("✓ Backup erfolgreich erstellt:\n{}", path),
                            Err(e) => format!("✗ Fehler: {}", e),
                        };
                    }
                }
            });
            
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
        Err("APPDATA Umgebungsvariable nicht gefunden".to_string())
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
        Err("Home-Verzeichnis nicht gefunden".to_string())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err("Plattform wird nicht unterstützt".to_string())
    }
}

fn get_downloads_path() -> Result<PathBuf, String> {
    dirs::download_dir().ok_or_else(|| "Downloads-Ordner nicht gefunden".to_string())
}


fn backup_world(world_name: &str) -> Result<String, String> {
    // Get the worlds directory
    let worlds_path = get_hytale_worlds_path()?;
    let world_path = worlds_path.join(world_name);

    if !world_path.exists() {
        return Err(format!("Welt '{}' nicht gefunden", world_name));
    }

    // Get the downloads directory
    let downloads_path = get_downloads_path()?;

    // Create timestamp for filename
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let zip_filename = format!("{}_{}.zip", world_name, timestamp);
    let zip_path = downloads_path.join(&zip_filename);

    // Create the ZIP file
    let file = File::create(&zip_path)
        .map_err(|e| format!("Konnte ZIP-Datei nicht erstellen: {}", e))?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Walk through all files in the world directory
    for entry in WalkDir::new(&world_path) {
        let entry = entry.map_err(|e| format!("Fehler beim Lesen der Dateien: {}", e))?;
        let path = entry.path();
        let name = path
            .strip_prefix(&world_path)
            .map_err(|e| format!("Fehler beim Verarbeiten des Pfades: {}", e))?;

        // Skip empty directory names
        if name.as_os_str().is_empty() {
            continue;
        }

        if path.is_file() {
            // Add file to ZIP
            zip.start_file(name.to_string_lossy().to_string(), options)
                .map_err(|e| format!("Konnte Datei nicht zum ZIP hinzufügen: {}", e))?;

            let file_content = fs::read(path)
                .map_err(|e| format!("Konnte Datei nicht lesen: {}", e))?;

            zip.write_all(&file_content)
                .map_err(|e| format!("Konnte Daten nicht in ZIP schreiben: {}", e))?;
        } else if path.is_dir() {
            // Add directory to ZIP
            zip.add_directory(name.to_string_lossy().to_string(), options)
                .map_err(|e| format!("Konnte Verzeichnis nicht zum ZIP hinzufügen: {}", e))?;
        }
    }

    zip.finish()
        .map_err(|e| format!("Konnte ZIP-Datei nicht fertigstellen: {}", e))?;

    Ok(zip_path.to_string_lossy().to_string())
}
