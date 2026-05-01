#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod commands;
mod tabs;
mod ui_helpers;
mod utils;
mod vcredist;

use app::App;
use eframe::egui;

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "windows")]
    vcredist::ensure_vc_runtime_x64();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 768.0])
            .with_min_inner_size([800.0, 600.0])
            .with_position(egui::Pos2::new(50.0, 50.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Eoliann Windows Tools",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}