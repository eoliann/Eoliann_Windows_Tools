#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod tabs;
mod commands;
mod utils;

use eframe::egui; // pentru ViewportBuilder
use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])   // dimensiune inițială mai mare
            .with_min_inner_size([800.0, 600.0])
            .with_position(egui::Pos2::new(50.0, 50.0)), // pornește centrat pe ecran
        ..Default::default()
    };

    eframe::run_native(
        "Eoliann Windows Tools",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
