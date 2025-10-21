// src/tabs/info.rs
use crate::utils::run_command;
use std::sync::{Arc, Mutex};

use egui::{Color32, RichText, Ui};

pub fn show_info(
    ui: &mut Ui,
    log_output: &Arc<Mutex<String>>,
    update_available: bool,
    latest_release: Option<&crate::utils::GithubRelease>,
) {
    let ascii_logo = r#"
    ███████╗ ██████╗ ██╗     ██╗ █████╗ ███╗   ██╗███╗   ██╗
    ██╔════╝██╔═══██╗██║     ██║██╔══██╗████╗  ██║████╗  ██║
    █████╗  ██║   ██║██║     ██║███████║██╔██╗ ██║██╔██╗ ██║
    ██╔══╝  ██║   ██║██║     ██║██╔══██║██║╚██╗██║██║╚██╗██║
    ███████╗╚██████╔╝███████╗██║██║  ██║██║ ╚████║██║ ╚████║
    ╚══════╝ ╚═════╝ ╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝
    "#;

    ui.label(
        RichText::new(ascii_logo)
            .monospace()
            .color(Color32::from_rgb(57, 255, 20))
            .size(16.0),
    );

    ui.separator();

    ui.heading("Info");
    ui.add_space(8.0);

    // --- UPDATE BANNER (simplu, robust) ---
    if update_available {
        // folosim Frame::group pentru compatibilitate cu versiuni egui
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⬆ Update available")
                        .strong()
                        .color(Color32::from_rgb(0, 255, 140)),
                );
                ui.add_space(8.0);

                if let Some(rel) = latest_release {
                    ui.label(RichText::new(format!("Latest: {}", rel.tag_name)).strong());
                    ui.add_space(8.0);
                    ui.hyperlink_to("Release notes", rel.html_url.as_str());
                    ui.add_space(8.0);
                    if ui.button("Open on GitHub").clicked() {
                        let _ = webbrowser::open(rel.html_url.as_str());
                        let mut lg = log_output.lock().unwrap();
                        lg.push_str(&format!("Opened release page: {}\n", rel.html_url));
                    }
                } else {
                    ui.label("A new version is available. Visit the project on GitHub.");
                    if ui.button("Open GitHub").clicked() {
                        let _ = webbrowser::open("https://github.com/eoliann/");
                        let mut lg = log_output.lock().unwrap();
                        lg.push_str("Opened GitHub repo\n");
                    }
                }
            });
        });

        ui.add_space(10.0);
    }

    // --- restul butoanelor existente ---
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("👤 whoami")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(Color32::from_rgb(57, 255, 20), "whoami");
                    ui.label("• Displays the current logged-in username.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("whoami");
            *log_output.lock().unwrap() = format!("> whoami\n{}", out);
        }

        if ui
            .button("🌐 ipconfig")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(Color32::from_rgb(57, 255, 20), "ipconfig /all");
                    ui.label("• Shows detailed network configuration.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("ipconfig /all");
            *log_output.lock().unwrap() = format!("> ipconfig /all\n{}", out);
        }

        if ui
            .button("💻 systeminfo")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(Color32::from_rgb(57, 255, 20), "systeminfo");
                    ui.label("• Displays detailed system configuration.");
                });
            })
            .clicked()
        {
            let out = crate::utils::run_command("systeminfo");
            *log_output.lock().unwrap() = format!("> systeminfo\n{}", out);
        }

        if ui
            .button("📋 tasklist")
            .on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(Color32::from_rgb(57, 255, 20), "tasklist");
                    ui.label("• Lists all running processes.");
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
    ui.label(format!("Eoliann Windows Tools Version {}", env!("CARGO_PKG_VERSION")));
    ui.add_space(8.0);
    ui.label("Created by Eoliann");
    ui.label("Quick tools for Windows administration.");

    let button_style = RichText::new("🌐 Open GitHub Repo")
        .color(Color32::from_rgb(57, 255, 20))
        .strong();

    if ui.button(button_style).clicked() {
        *log_output.lock().unwrap() = run_command("explorer https://github.com/eoliann/");
    }

    ui.add_space(12.0);

    // notă: dacă structa ta GithubRelease are și alte câmpuri, le poți afișa aici
}
