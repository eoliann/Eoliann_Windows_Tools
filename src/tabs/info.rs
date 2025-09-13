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
        if ui.button("👤 whoami")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "whoami");
                    ui.label("• Displays the current logged-in username.");
                    ui.label("• Useful for confirming privileges or user context.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("whoami");
            *log_output.lock().unwrap() = format!("> whoami\n{}", out);
        }

        if ui.button("🌐 ipconfig")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "ipconfig /all");
                    ui.label("• Shows detailed network configuration.");
                    ui.label("• Includes IPs, DNS, DHCP, and MAC addresses.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("ipconfig /all");
            *log_output.lock().unwrap() = format!("> ipconfig /all\n{}", out);
        }

        if ui.button("💻 systeminfo")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "systeminfo");
                    ui.label("• Displays detailed system configuration.");
                    ui.label("• Useful for diagnosing Windows version, uptime, and hardware info.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("systeminfo");
            *log_output.lock().unwrap() = format!("> systeminfo\n{}", out);
        }

        if ui.button("📋 tasklist")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "tasklist");
                    ui.label("• Lists all running processes.");
                    ui.label("• Useful for checking active programs and resource usage.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("tasklist");
            *log_output.lock().unwrap() = format!("> tasklist\n{}", out);
        }
    });

    ui.separator();
    ui.label("📖 About:");
    // ui.label("Eoliann Windows Tools Version 1.0.7");
    ui.label(format!("Eoliann Windows Tools Version {}", env!("CARGO_PKG_VERSION")));
    ui.add_space(8.0);
    ui.label("Created by Eoliann");
    ui.label("Quick tools for Windows administration.");
    // if ui.button("Open GitHub Repo").clicked() {
    //     *log_output.lock().unwrap() = run_command("explorer https://github.com/eoliann/");
    // }
    let button_style = egui::RichText::new("🌐 Open GitHub Repo")
        .color(egui::Color32::from_rgb(57, 255, 20)) // verde neon
        .strong();

    if ui.button(button_style).clicked() {
        *log_output.lock().unwrap() = run_command("explorer https://github.com/eoliann/");
    }

}
