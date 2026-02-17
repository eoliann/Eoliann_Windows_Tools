use std::sync::{Arc, Mutex};
use crate::commands;
use crate::commands::GLOBAL_OP_RUNNING;

pub fn show_quick_keys(ui: &mut egui::Ui, _log: &Arc<Mutex<String>>) {
    ui.heading("⌨ Quick Keys");
    ui.separator();

    ui.label("Execute Windows built-in keyboard shortcuts.");

    let global_busy =
        GLOBAL_OP_RUNNING.load(std::sync::atomic::Ordering::SeqCst);

    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(!global_busy, egui::Button::new("Win + X\nQuick Link Menu")).clicked() {
            commands::send_win_x();
        }

        if ui.add_enabled(!global_busy, egui::Button::new("Win + D\nShow Desktop")).clicked() {
            commands::send_win_d();
        }

        if ui.add_enabled(!global_busy, egui::Button::new("Win + L\nLock PC")).clicked() {
            commands::send_win_l();
        }

        if ui.add_enabled(!global_busy, egui::Button::new("Win + R\nRun Dialog")).clicked() {
            commands::send_win_r();
        }

        if ui.add_enabled(!global_busy, egui::Button::new("Win + E\nFile Explorer")).clicked() {
            commands::send_win_e();
        }

        if ui.add_enabled(!global_busy, egui::Button::new("Win + I\nSettings")).clicked() {
            commands::send_win_i();
        }

        // ────────────────────────────────────────────────
        // NOU: Registry Editor (simulează Win+R → regedit → Enter)
        // ────────────────────────────────────────────────
        if ui.add_enabled(!global_busy, egui::Button::new("Regedit\nRegistry Editor")).clicked() {
            commands::open_registry_editor();
        }

        // ────────────────────────────────────────────────
        // NOU: Group Policy Editor (Win + R → gpedit.msc → Enter)
        // ────────────────────────────────────────────────
        let gp_button = ui.add_enabled(!global_busy, egui::Button::new("gpedit.msc\nGroup Policy"));

        gp_button.clone().on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label("Opens Group Policy Editor (gpedit.msc)");
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "Note: Only available on Windows Pro / Enterprise / Education editions"
                );
                ui.label("On Home edition → will show error unless manually enabled via script.");
            });
        });

        if gp_button.clicked() {
            commands::open_group_policy_editor();
        }
    });
}