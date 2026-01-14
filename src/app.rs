use eframe::egui;
use rust_i18n::t;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::backup::{backup_world_to_path_with_progress, get_hytale_worlds_path, import_world};
use crate::models::{BackupProgress, WorldInfo};
use crate::utils::{
    format_size, get_directory_size, get_last_played, get_latest_log, get_world_backups,
    open_file_in_finder,
};

/// Maximum height for tab content areas
pub const TAB_CONTENT_MAX_HEIGHT: f32 = 260.0;

pub struct HytaleBackupApp {
    pub status_message: String,
    pub worlds: Vec<WorldInfo>,
    pub selected_world: Option<usize>,
    pub selected_tab: usize,
    pub include_logs: bool,
    pub include_backups: bool,
    pub progress: Arc<Mutex<BackupProgress>>,
    pub pending_delete_backup: Option<PathBuf>,
    pub pending_import: Option<(PathBuf, String)>,
}

impl HytaleBackupApp {
    pub fn new() -> Self {
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

    pub fn load_worlds() -> Vec<WorldInfo> {
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

    pub fn refresh_worlds(&mut self) {
        self.worlds = Self::load_worlds();
        self.selected_world = None;
    }

    fn render_delete_dialog(&mut self, ctx: &egui::Context) {
        if let Some(backup_path) = self.pending_delete_backup.clone() {
            egui::Window::new(t!("app.confirm_delete_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(t!("app.confirm_delete_message"));
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(
                            backup_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                        )
                        .strong(),
                    );
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button(t!("app.cancel")).clicked() {
                            self.pending_delete_backup = None;
                        }

                        if ui
                            .button(
                                egui::RichText::new(t!("app.delete"))
                                    .color(egui::Color32::from_rgb(255, 100, 100)),
                            )
                            .clicked()
                        {
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
    }

    fn render_import_dialog(&mut self, ctx: &egui::Context) {
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
                    ui.label(
                        egui::RichText::new(t!("app.confirm_import_warning"))
                            .color(egui::Color32::from_rgb(255, 180, 100)),
                    );
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button(t!("app.cancel")).clicked() {
                            self.pending_import = None;
                        }

                        if ui
                            .button(
                                egui::RichText::new(t!("app.import"))
                                    .color(egui::Color32::from_rgb(100, 200, 100)),
                            )
                            .clicked()
                        {
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
    }

    fn render_toolbar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.add_space(10.0);

        let progress_state = self.progress.lock().unwrap().clone();

        if progress_state.is_running {
            self.render_progress_bar(ui, &progress_state);
            ctx.request_repaint();
        } else {
            if let Some(result) = progress_state.result.clone() {
                self.status_message = match result {
                    Ok(path) => format!("{}\n{}", t!("app.backup_success"), path),
                    Err(e) => format!("{} {}", t!("app.error"), e),
                };
                self.progress.lock().unwrap().result = None;
            }

            self.render_toolbar_controls(ctx, ui);
        }

        if !self.status_message.is_empty() {
            ui.add_space(5.0);
            ui.separator();
            ui.label(&self.status_message);
        }

        ui.add_space(10.0);
    }

    fn render_progress_bar(&self, ui: &mut egui::Ui, progress_state: &BackupProgress) {
        ui.vertical_centered(|ui| {
            ui.label(t!("app.compressing"));

            let progress_fraction = if progress_state.total > 0 {
                progress_state.current as f32 / progress_state.total as f32
            } else {
                0.0
            };

            ui.add(
                egui::ProgressBar::new(progress_fraction)
                    .show_percentage()
                    .animate(true),
            );

            ui.label(format!(
                "{} / {}",
                progress_state.current, progress_state.total
            ));

            if !progress_state.current_file.is_empty() {
                ui.label(&progress_state.current_file);
            }
        });
    }

    fn render_toolbar_controls(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.include_logs, t!("app.include_logs"));
            ui.add_space(20.0);
            ui.checkbox(&mut self.include_backups, t!("app.include_backups"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button_enabled = self.selected_world.is_some();
                if ui
                    .add_enabled(button_enabled, egui::Button::new(t!("app.compress_world")))
                    .clicked()
                {
                    self.start_backup(ctx);
                }
            });
        });
    }

    fn start_backup(&mut self, ctx: &egui::Context) {
        if let Some(index) = self.selected_world {
            let world = self.worlds[index].clone();
            let include_logs = self.include_logs;
            let include_backups = self.include_backups;

            let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
            let default_filename = format!("{}_{}.zip", world.name, timestamp);

            let file_dialog = rfd::FileDialog::new()
                .set_file_name(&default_filename)
                .add_filter("ZIP", &["zip"]);

            let file_dialog = if let Some(downloads) = dirs::download_dir() {
                file_dialog.set_directory(&downloads)
            } else {
                file_dialog
            };

            if let Some(save_path) = file_dialog.save_file() {
                let progress = Arc::clone(&self.progress);
                let ctx = ctx.clone();

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
                        &ctx,
                    );

                    let mut p = progress.lock().unwrap();
                    p.is_running = false;
                    p.result = Some(result);
                    ctx.request_repaint();
                });
            }
        }
    }

    fn render_world_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(t!("app.available_worlds"));
            if ui.button(t!("app.refresh")).clicked() {
                self.refresh_worlds();
            }
            if ui.button(t!("app.import_world")).clicked() {
                self.handle_import_button();
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
                        if ui
                            .selectable_label(is_selected, format!("🌍 {}", world.name))
                            .clicked()
                        {
                            self.selected_world = Some(index);
                        }
                    }
                });
        }
    }

    fn handle_import_button(&mut self) {
        let file_dialog = rfd::FileDialog::new().add_filter("ZIP", &["zip"]);

        if let Some(zip_path) = file_dialog.pick_file() {
            if zip_path.extension().map_or(false, |ext| ext == "zip") {
                if let Some(filename) = zip_path.file_stem() {
                    let filename_str = filename.to_string_lossy().to_string();
                    let world_name =
                        if filename_str.len() > 20 && filename_str.chars().rev().nth(19) == Some('_')
                        {
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

    fn render_world_details(&mut self, ui: &mut egui::Ui) {
        ui.label(t!("app.details"));
        ui.add_space(10.0);

        if let Some(index) = self.selected_world {
            if let Some(world) = self.worlds.get(index) {
                let available_width = ui.available_width();
                egui::Frame::group(ui.style())
                    .inner_margin(10.0)
                    .corner_radius(5.0)
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
                                ui.label(
                                    world
                                        .last_played
                                        .clone()
                                        .unwrap_or_else(|| t!("app.unknown").to_string()),
                                );
                                ui.end_row();

                                ui.label(t!("app.detail_path"));
                                ui.label(
                                    egui::RichText::new(world.path.to_string_lossy().to_string())
                                        .small()
                                        .weak(),
                                );
                                ui.end_row();
                            });
                    });

                ui.add_space(15.0);
                self.render_tabs(ui, &world.path.clone());
            }
        } else {
            ui.label(t!("app.select_world_hint"));
        }
    }

    fn render_tabs(&mut self, ui: &mut egui::Ui, world_path: &PathBuf) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.selected_tab == 0, t!("app.tab_backups"))
                .clicked()
            {
                self.selected_tab = 0;
            }
            ui.separator();
            if ui
                .selectable_label(self.selected_tab == 1, t!("app.tab_logs"))
                .clicked()
            {
                self.selected_tab = 1;
            }
        });

        ui.add_space(10.0);

        match self.selected_tab {
            0 => self.render_backups_tab(ui, world_path),
            1 => self.render_logs_tab(ui, world_path),
            _ => {}
        }
    }

    fn render_backups_tab(&mut self, ui: &mut egui::Ui, world_path: &PathBuf) {
        let backups = get_world_backups(world_path);

        if backups.is_empty() {
            ui.label(t!("app.no_backups_found"));
        } else {
            egui::ScrollArea::vertical()
                .id_salt("backups_list")
                .max_height(TAB_CONTENT_MAX_HEIGHT)
                .show(ui, |ui| {
                    for backup in &backups {
                        egui::Frame::group(ui.style()).inner_margin(5.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&backup.name).strong());
                                    ui.label(egui::RichText::new(format_size(backup.size)).weak());
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .button("🗑")
                                            .on_hover_text(t!("app.delete_backup"))
                                            .clicked()
                                        {
                                            self.pending_delete_backup = Some(backup.path.clone());
                                        }
                                        if ui
                                            .button("📂")
                                            .on_hover_text(t!("app.open_in_finder"))
                                            .clicked()
                                        {
                                            open_file_in_finder(&backup.path);
                                        }
                                    },
                                );
                            });
                        });
                        ui.add_space(5.0);
                    }
                });
        }
    }

    fn render_logs_tab(&self, ui: &mut egui::Ui, world_path: &PathBuf) {
        if let Some(log) = get_latest_log(world_path) {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&log.name).strong());
                if ui
                    .button("📂")
                    .on_hover_text(t!("app.open_in_finder"))
                    .clicked()
                {
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
    }
}

impl eframe::App for HytaleBackupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render dialogs
        self.render_delete_dialog(ctx);
        self.render_import_dialog(ctx);

        // Bottom toolbar
        egui::TopBottomPanel::bottom("toolbar").show(ctx, |ui| {
            self.render_toolbar(ctx, ui);
        });

        // Main content panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                ui.heading(t!("app.title"));
            });

            ui.add_space(20.0);

            self.render_world_list(ui);

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            self.render_world_details(ui);
        });
    }
}

