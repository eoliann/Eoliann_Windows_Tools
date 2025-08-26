use crate::utils::run_command;

pub fn show_info(ui: &mut egui::Ui, log_output: &mut String) {
    // ASCII Logo retro hacker
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

    ui.horizontal_wrapped(|ui| {
        if ui.button("whoami").clicked() {
            *log_output = run_command("whoami");
        }
        if ui.button("ipconfig").clicked() {
            *log_output = run_command("ipconfig");
        }
        if ui.button("systeminfo").clicked() {
            *log_output = run_command("systeminfo");
        }
        if ui.button("tasklist").clicked() {
            *log_output = run_command("tasklist");
        }
    });

    ui.separator();
    ui.label("📖 About:");
    ui.label("Eoliann Windows Tools v1.0.1");
    ui.label("Created by Eoliann");
    ui.label("Quick tools for Windows administration.");
    if ui.button("Open GitHub Repo").clicked() {
        *log_output = run_command("explorer https://github.com/eoliann/");
    }
}
