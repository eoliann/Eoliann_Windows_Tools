// src/ui_helpers.rs
use std::collections::HashMap;
use eframe::egui;
use eframe::egui::RichText;

/// Draw an icon by key from the provided icons map.
/// If the texture is present, draws it at `size` px. Otherwise falls back to a safe text label.
pub fn draw_icon(
    ui: &mut egui::Ui,
    icons: &HashMap<String, egui::TextureHandle>,
    key: &str,
    size: f32,
) {
    if let Some(tx) = icons.get(key) {
        // Ui::image expects a single argument convertible to ImageSource.
        // Pass a tuple (TextureId, Vec2) as one argument.
        ui.image((tx.id(), egui::vec2(size, size)));
    } else {
        let fallback = match key {
            "windows" => "Windows",
            "system" => "System",
            "processor" => "CPU",
            "memory" => "RAM",
            "storage" => "Storage",
            "graphics" => "GPU",
            "apps" => "Apps",
            "processes" => "Procs",
            "services" => "Svc",
            "network" => "Net",
            "performance" => "Perf",
            _ => key,
        };
        ui.label(RichText::new(fallback).size(size));
    }
}
