use crate::utils::run_command;
use std::sync::{Arc, Mutex};


#[allow(dead_code)]
pub fn show_info(ui: &mut egui::Ui, log_output: &Arc<Mutex<String>>) {
    let ascii_logo = r#"
    ███████╗ ██████╗ ██╗     ██╗ █████╗ ███╗   ██╗███╗   ██╗
    ██╔════╝██╔═══██╗██║     ██║██╔══██╗████╗  ██║████╗  ██║
    █████╗  ██║   ██║██║     ██║███████║██╔██╗ ██║██╔██╗ ██║
    ██╔══╝  ██║   ██║██║     ██║██╔══██║██║╚██╗██║██║╚██╗██║
    ███████╗╚██████╔╝███████╗██║██║  ██║██║ ╚████║██║ ╚████║
    ╚══════╝ ╚═════╝ ╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝
    "#;

    ui.label(
        egui::RichText::new(ascii_logo)
            .monospace()
            .color(egui::Color32::from_rgb(57, 255, 20))
            .size(16.0),
    );

    ui.separator();

    ui.heading("Info");
    ui.add_space(8.0);

    ui.horizontal_wrapped(|ui| {
        if ui.button("whoami").clicked() {
            let out = crate::utils::run_command("whoami");
            *log_output.lock().unwrap() = format!("> whoami\n{}", out);
        }
        if ui.button("ipconfig").clicked() {
            let out = crate::utils::run_command("ipconfig /all"); // Changed from `log` to `log_output`
            *log_output.lock().unwrap() = format!("> ipconfig /all\n{}", out);
        }
        if ui.button("systeminfo").clicked() {
            let out = crate::utils::run_command("systeminfo");
            *log_output.lock().unwrap() = format!("> systeminfo\n{}", out);
        }
        if ui.button("tasklist").clicked() {
            let out = crate::utils::run_command("tasklist");
            *log_output.lock().unwrap() = format!("> tasklist\n{}", out);
        }
    });

    ui.separator();
    ui.label("📖 About:");
    ui.label("Eoliann Windows Tools Version 1.0.3");
    ui.label("Created by Eoliann");
    ui.label("Quick tools for Windows administration.");
    if ui.button("Open GitHub Repo").clicked() {
        *log_output.lock().unwrap() = run_command("explorer https://github.com/eoliann/");
    }
}
