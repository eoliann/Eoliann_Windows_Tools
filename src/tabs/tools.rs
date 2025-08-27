use crate::commands::*;

pub fn show_tools(
    ui: &mut egui::Ui,
    log_output: &mut String,
    show_popup: &mut bool,
    popup_message: &mut String,
) {
    // ===== Context Menu =====
    ui.group(|ui| {
        ui.heading("🖱 Context Menu");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Toggle Context Menu").clicked() {
                let result = toggle_context_menu();
                *log_output = result.clone();
                *popup_message = result;
                *show_popup = true;
            }
        });
    });

    ui.separator();

    // ===== Maintenance =====
    ui.group(|ui| {
        ui.heading("🛠 Maintenance");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Disk Cleanup").clicked() {
                *log_output = disk_cleanup();
            }
            if ui.button("Network Reset").clicked() {
                *log_output = network_reset();
            }
            if ui.button("Open Display Settings").clicked() {
            *log_output = quick_access_settings("display");
        }
        });
    });

    ui.separator();

    // ===== Power Plans =====
    ui.group(|ui| {
        ui.heading("⚡ Power Plans");
        ui.horizontal_wrapped(|ui| {
            if ui.button("High Performance").clicked() {
                *log_output =
                    power_plan_switcher("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c");
            }
            if ui.button("Balanced").clicked() {
                *log_output =
                    power_plan_switcher("381b4222-f694-41f0-9685-ff5bb260df2e");
            }
            if ui.button("Power Saver").clicked() {
                *log_output =
                    power_plan_switcher("a1841308-3541-4fab-bc81-f71556f20b4a");
            }
        });
    });

    ui.separator();

    // ===== Power Tweaks =====
    ui.group(|ui| {
        ui.heading("🔋 Power Tweaks");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Disable Sleep").clicked() {
                *log_output = disable_sleep();
            }
            if ui.button("Disable HDD/SSD Timeout").clicked() {
                *log_output = disable_hdd();
            }
            if ui.button("Disable Monitor Timeout").clicked() {
                *log_output = disable_monitor();
            }
        });
    });
}
