use crate::commands::*;

pub fn show_tools(
    ui: &mut egui::Ui,
    log_output: &mut String,
    show_popup: &mut bool,
    popup_message: &mut String,
) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("Toggle Context Menu").clicked() {
            let result = toggle_context_menu();
            *log_output = result.clone();
            *popup_message = result;
            *show_popup = true;
        }

        if ui.button("Disk Cleanup").clicked() {
            *log_output = disk_cleanup();
        }

        if ui.button("Open Display Settings").clicked() {
            *log_output = quick_access_settings("display");
        }

        if ui.button("Network Reset").clicked() {
            *log_output = network_reset();
        }

        if ui.button("Switch to High Performance").clicked() {
            *log_output =
                power_plan_switcher("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"); // GUID High Performance
        }

        if ui.button("Switch to Balanced").clicked() {
            *log_output =
                power_plan_switcher("381b4222-f694-41f0-9685-ff5bb260df2e"); // GUID Balanced
        }

        if ui.button("Switch to Power Saver").clicked() {
            *log_output =
                power_plan_switcher("a1841308-3541-4fab-bc81-f71556f20b4a"); // GUID Power Saver
        }
    });
}
