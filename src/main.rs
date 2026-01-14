mod app;
mod backup;
mod log_filter;
mod models;
mod utils;

use eframe::egui;
use rust_i18n::t;

use app::HytaleBackupApp;
use utils::detect_system_locale;

rust_i18n::i18n!("locales", fallback = "en");

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

