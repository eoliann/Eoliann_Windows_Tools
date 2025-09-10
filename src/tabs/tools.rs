use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::commands;

pub fn show_tools(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    _show_popup: &mut bool,
    _popup_message: &mut String,
) {
    ui.heading("🛠 Windows Tools");
    ui.add_space(6.0);

    // ---- Context menu ----
    ui.group(|ui| {
        ui.label("Context menu");
        if ui.button("Toggle context menu (Win11/Classic)").clicked() {
            let out = commands::toggle_context_menu();
            *log.lock().unwrap() = out;
        }
    });

    ui.add_space(6.0);

    // ---- Maintenance ----
    ui.group(|ui| {
        ui.label("Maintenance");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Disk Cleanup").clicked() {
                let log_clone = log.clone();
                thread::spawn(move || {
                    let msg = commands::disk_cleanup();
                    *log_clone.lock().unwrap() = msg;
                });
            }
            if ui.button("Empty Recycle Bin").clicked() {
                *log.lock().unwrap() = commands::empty_recycle_bin();
            }
            if ui.button("Clean Temporary Files").clicked() {
                *log.lock().unwrap() = commands::clean_temporary_files();
            }
            if ui.button("Network Reset").clicked() {
                *log.lock().unwrap() = commands::network_reset();
            }
            if ui.button("Verify System Integrity (SFC + DISM)").clicked() {
                *log.lock().unwrap() =
                    "⏳ Starting system integrity check... (SFC + DISM)".to_string();
                commands::verify_system_integrity_live(log.clone());
            }
            // NOTĂ: "Open Display Settings" este mutat în Settings (cerința ta)
        });
    });

    ui.add_space(6.0);

    // ---- Power Plans ----
    ui.group(|ui| {
        ui.label("Power Plans");
        ui.horizontal_wrapped(|ui| {
            if ui.button("High Performance").clicked() {
                *log.lock().unwrap() =
                    commands::power_plan_switcher("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c");
            }
            if ui.button("Balanced").clicked() {
                *log.lock().unwrap() =
                    commands::power_plan_switcher("381b4222-f694-41f0-9685-ff5bb260df2e");
            }
            if ui.button("Power Saver").clicked() {
                *log.lock().unwrap() =
                    commands::power_plan_switcher("a1841308-3541-4fab-bc81-f71556f20b4a");
            }
        });
    });

    ui.add_space(6.0);

    // ---- Power Tweaks ----
    ui.group(|ui| {
        ui.label("Power Tweaks");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Disable Sleep").clicked() {
                *log.lock().unwrap() = commands::disable_sleep();
            }
            if ui.button("Disable HDD/SSD timeout").clicked() {
                *log.lock().unwrap() = commands::disable_hdd();
            }
            if ui.button("Disable Monitor timeout").clicked() {
                *log.lock().unwrap() = commands::disable_monitor();
            }
        });
    });
}
