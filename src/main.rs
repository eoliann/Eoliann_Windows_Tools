#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod tabs;
mod commands;
mod utils;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0]) // ✅ dimensiunea inițială
            .with_min_inner_size([800.0, 600.0]), // ✅ dimensiunea minimă
        ..Default::default()
    };

    eframe::run_native(
        "Eoliann Windows Tools",
        options,
        Box::new(|_cc| Box::new(App::default())),
    )
}
