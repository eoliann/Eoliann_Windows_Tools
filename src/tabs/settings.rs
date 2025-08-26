use crate::commands::change_theme;

pub fn show_settings(ui: &mut egui::Ui, log_output: &mut String) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("Switch to Dark Mode").clicked() {
            *log_output = change_theme("dark");
        }
        if ui.button("Switch to Light Mode").clicked() {
            *log_output = change_theme("light");
        }
    });
}
