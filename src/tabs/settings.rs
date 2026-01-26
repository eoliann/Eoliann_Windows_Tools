use std::sync::{Arc, Mutex};
use std::process::Command;

use crate::commands;
use crate::commands::GLOBAL_OP_RUNNING;

#[allow(dead_code)]
pub fn show_settings(ui: &mut egui::Ui, log_output: &Arc<Mutex<String>>) {
    ui.heading("⚙ Settings");
    ui.separator();

    ui.label("Here you can configure application settings (theme, preferences, etc).");

    let global_busy = GLOBAL_OP_RUNNING.load(std::sync::atomic::Ordering::SeqCst);

    // Action flags (avoid mutable borrow conflicts in egui)
    let mut do_light = false;
    let mut do_dark = false;
    let mut do_center = false;
    let mut do_left = false;
    let mut do_create_local_account = false;
    let mut do_about = false;

    // Read current taskbar alignment once
    let current_state_centered = crate::commands::get_taskbar_alignment();

    ui.horizontal_wrapped(|ui| {
        // Theme buttons
        if ui.button("🌞 Switch to Light Mode").clicked() {
            do_light = true;
        }

        if ui.button("🌙 Switch to Dark Mode").clicked() {
            do_dark = true;
        }

        // Alignment status
        ui.label(format!(
            "Current alignment: {}",
            if current_state_centered { "Center" } else { "Left" }
        ));

        // Center taskbar
        let resp_center = ui.add_enabled(!global_busy, egui::Button::new("⚙ Center Taskbar Items"));
        let resp_center = resp_center.on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label("Center the taskbar items (Windows 11).");
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "Writes HKCU\\...\\Explorer\\Advanced\\TaskbarAl = 1. No admin needed."
                );
                ui.colored_label(
                    egui::Color32::RED,
                    "⚠ May require Explorer restart to be visible."
                );
                ui.hyperlink(
                    "https://christitustech.github.io/Winutil/dev/tweaks/Customize-Preferences/TaskbarAlignment"
                );
            });
        });

        if resp_center.clicked() {
            do_center = true;
        }

        // Left taskbar
        let resp_left = ui.add_enabled(!global_busy, egui::Button::new("◀ Left Taskbar Items"));
        let resp_left = resp_left.on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label("Align the taskbar items to the left (classic).");
                ui.colored_label(
                    egui::Color32::LIGHT_BLUE,
                    "Writes TaskbarAl = 0. No admin required."
                );
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "ℹ Restart Explorer if change is not visible immediately."
                );
            });
        });

        if resp_left.clicked() {
            do_left = true;
        }

        // Create local account (NEW FEATURE)
        let resp_local = ui.button("➕ Create new local account");
        let resp_local = resp_local.on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label("Open Windows 11 local account creation UI.");
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "Uses PowerShell: start ms-cxh:localonly"
                );
                ui.colored_label(
                    egui::Color32::RED,
                    "⚠ Availability depends on Windows 11 context."
                );
            });
        });

        if resp_local.clicked() {
            do_create_local_account = true;
        }

        // About button
        let button_style = egui::RichText::new("ℹ About")
            .color(egui::Color32::from_rgb(57, 255, 20))
            .strong();

        if ui.button(button_style).clicked() {
            do_about = true;
        }
    });

    // ---- Action handlers (after UI borrow ends) ----

    if do_light {
        let out = commands::change_theme("light");
        if let Ok(mut lg) = log_output.lock() {
            *lg = out;
        }
    }

    if do_dark {
        let out = commands::change_theme("dark");
        if let Ok(mut lg) = log_output.lock() {
            *lg = out;
        }
    }

    if do_center {
        if let Some(guard) =
            commands::try_start_global_op("Center Taskbar Items", &log_output.clone())
        {
            let log_clone = log_output.clone();
            std::thread::spawn(move || {
                let _guard = guard;
                let result = crate::commands::enable_center_taskbar();
                let mut lg = log_clone.lock().unwrap();
                if lg.is_empty() {
                    *lg = result;
                } else {
                    *lg = format!("{}\n{}", lg, result);
                }
            });
        }
    }

    if do_left {
        if let Some(guard) =
            commands::try_start_global_op("Left Taskbar Items", &log_output.clone())
        {
            let log_clone = log_output.clone();
            std::thread::spawn(move || {
                let _guard = guard;
                let result = crate::commands::disable_center_taskbar();
                let mut lg = log_clone.lock().unwrap();
                if lg.is_empty() {
                    *lg = result;
                } else {
                    *lg = format!("{}\n{}", lg, result);
                }
            });
        }
    }

    if do_create_local_account {
        #[cfg(target_os = "windows")]
        {
            let result = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "start ms-cxh:localonly"
                ])
                .spawn();

            let mut lg = log_output.lock().unwrap();
            match result {
                Ok(_) => {
                    *lg = "Opened Windows local account creation UI.".to_string();
                }
                Err(e) => {
                    *lg = format!(
                        "Failed to open local account creation UI.\nError: {}",
                        e
                    );
                }
            }
        }
    }

    if do_about {
        let mut lg = log_output.lock().unwrap();
        *lg = format!(
            "Eoliann Windows Tools v{}\nCreated by Eoliann Dev\nQuick tools for Windows administration.\nWebsite: https://github.com/eoliann",
            env!("CARGO_PKG_VERSION")
        );
    }

    ui.separator();
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_show_settings_usage() {
        let log_output = Arc::new(Mutex::new(String::new()));
        *log_output.lock().unwrap() = String::from("test");
        assert_eq!(&*log_output.lock().unwrap(), "test");
    }
}
