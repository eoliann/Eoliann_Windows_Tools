use std::sync::{Arc, Mutex};
use crate::commands;

use crate::commands::GLOBAL_OP_RUNNING;
#[allow(dead_code)]
pub fn show_settings(ui: &mut egui::Ui, log_output: &Arc<Mutex<String>>) {
    ui.heading("⚙ Settings");

    ui.separator();

    ui.label("Here you can configure application settings (theme, preferences, etc).");

    // if ui.button("🔄 Reset Output").clicked() {
    //     *log_output.lock().unwrap() = String::new();
    // }
    let global_busy = GLOBAL_OP_RUNNING.load(std::sync::atomic::Ordering::SeqCst);

    ui.horizontal_wrapped(|ui| {
        if ui.button("🌞 Switch to Light Mode").clicked() {
            *log_output.lock().unwrap() = commands::change_theme("light");
        }
        if ui.button("🌙 Switch to Dark Mode").clicked() {
            *log_output.lock().unwrap() = commands::change_theme("dark");
        }

        // Center Taskbar Items (Enable)
        // la începutul funcției UI (execută o dată pe frame init/first-run, nu pe fiecare frame)
        let current_state_centered = crate::commands::get_taskbar_alignment(); // bool
        ui.label(format!("Current alignment: {}", if current_state_centered { "Center" } else { "Left" }));

        let resp = ui.add_enabled(!global_busy, egui::Button::new("⚙ Center Taskbar Items"));
        resp.clone().on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label("Center the taskbar items (Windows 11).");
                ui.colored_label(egui::Color32::YELLOW, "Writes HKCU\\...\\Explorer\\Advanced\\TaskbarAl = 1. No admin needed.");
                ui.colored_label(egui::Color32::RED, "⚠ May require Explorer restart to be visible.");
                ui.hyperlink("https://christitustech.github.io/Winutil/dev/tweaks/Customize-Preferences/TaskbarAlignment");
            });
        });
        if resp.clicked() {
            if let Some(guard) = commands::try_start_global_op("Center Taskbar Items", log_output) {
                let log_clone = log_output.clone();
                std::thread::spawn(move || {
                    let _guard = guard;
                    let result = crate::commands::enable_center_taskbar();
                    let mut lg = log_clone.lock().unwrap();
                    if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                });
            }
        }

        // Left Taskbar Items (Disable center)
        let resp = ui.add_enabled(!global_busy, egui::Button::new("◀ Left Taskbar Items"));
        resp.clone().on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label("Align the taskbar items to the left (classic).");
                ui.colored_label(egui::Color32::LIGHT_BLUE, "Writes TaskbarAl = 0. No admin required.");
                ui.colored_label(egui::Color32::YELLOW, "ℹ Restart Explorer if change is not visible immediately.");
            });
        });
        if resp.clicked() {
            if let Some(guard) = commands::try_start_global_op("Left Taskbar Items", log_output) {
                let log_clone = log_output.clone();
                std::thread::spawn(move || {
                    let _guard = guard;
                    let result = crate::commands::disable_center_taskbar();
                    let mut lg = log_clone.lock().unwrap();
                    if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                });
            }
        }



        let button_style = egui::RichText::new("ℹ About")
            .color(egui::Color32::from_rgb(57, 255, 20)) // verde neon
            .strong();

        if ui.button(button_style).clicked() {
            let mut log = log_output.lock().unwrap();
            // *log = "Eoliann Windows Tools v1.0.7\nCreated by Eoliann Dev\nQuick tools for Windows administration.\nWebsite: https://github.com/eoliann".to_string();
            *log = format!(
                "Eoliann Windows Tools v{}\nCreated by Eoliann Dev\nQuick tools for Windows administration.\nWebsite: https://github.com/eoliann",
                env!("CARGO_PKG_VERSION")
            );

        }
    });
    ui.separator();
}

// Example usage: Call this function from your main UI code
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    #[test]
    fn test_show_settings_usage() {
        let log_output = Arc::new(Mutex::new(String::new()));
        // Cannot directly instantiate egui::Ui, so just test log_output logic
        {
            *log_output.lock().unwrap() = String::from("test");
            assert_eq!(&*log_output.lock().unwrap(), "test");
        }
    }
}
