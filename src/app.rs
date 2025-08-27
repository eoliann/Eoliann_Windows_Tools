use crate::tabs::{
    info::show_info,
    tools::show_tools,
    settings::show_settings,
    winapp_removal::show_winapp_removal,
};

use egui::{FontDefinitions, FontData, FontFamily, FontId, TextStyle};

pub struct App {
    active_tab: Tab,
    log_output: String,
    show_popup: bool,
    popup_message: String,
    show_bulk_report: bool,
    bulk_report: String,
}


#[derive(PartialEq)]
enum Tab {
    Info,
    Tools,
    WinAppRemoval,
    Settings,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_tab: Tab::Info,
            log_output: "📊 INFO Selected".to_string(),
            show_popup: false,
            popup_message: String::new(),
            show_bulk_report: false,
            bulk_report: String::new(),
        }
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===== Fonts =====
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "JetBrainsMono".to_owned(),
            FontData::from_owned(include_bytes!("../assets/JetBrainsMono-Regular.ttf").to_vec()),
        );
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "JetBrainsMono".to_owned());
        ctx.set_fonts(fonts);

        // ===== Text Styles =====
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
        ]
        .into();

        // ===== Custom Retro Hacker Theme =====
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(57, 255, 20)); // verde neon
        style.visuals.window_fill = egui::Color32::from_rgb(0, 0, 0); // fundal negru
        style.visuals.panel_fill = egui::Color32::from_rgb(0, 0, 0);
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(57, 255, 20);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0, 20, 0);
        style.visuals.widgets.inactive.bg_stroke.width = 1.5;
        style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(0, 255, 127);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 40, 0);
        style.visuals.widgets.hovered.bg_stroke.width = 2.0;
        style.visuals.widgets.active.fg_stroke.color = egui::Color32::BLACK;
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(57, 255, 20);
        style.visuals.widgets.active.bg_stroke.width = 2.5;

        ctx.set_style(style);

        // ===== Sidebar =====
        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            ui.heading("📂 Menu");
            if ui.button("📊 INFO").clicked() {
                self.active_tab = Tab::Info;
                self.log_output = "📊 INFO Selected".to_string();
            }
            if ui.button("🛠 TOOLS").clicked() {
                self.active_tab = Tab::Tools;
                self.log_output = "🛠 TOOLS Selected".to_string();
            }
            if ui.button("🗑 WinApp Removal").clicked() {
                self.active_tab = Tab::WinAppRemoval;
                self.log_output = "🗑 WinApp Removal Selected".to_string();
            }
            if ui.button("⚙ SETTINGS").clicked() {
                self.active_tab = Tab::Settings;
                self.log_output = "⚙ SETTINGS Selected".to_string();
            }
        });

        // ===== Central Panel =====
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Info => show_info(ui, &mut self.log_output),
                Tab::Tools => show_tools(
                    ui,
                    &mut self.log_output,
                    &mut self.show_popup,
                    &mut self.popup_message,
                ),
                Tab::WinAppRemoval => show_winapp_removal(
                    ui,
                    &mut self.log_output,
                    &mut self.show_popup,
                    &mut self.popup_message,
                ),
                Tab::Settings => show_settings(ui, &mut self.log_output),
            }

            ui.separator();
            ui.label("📝 Output:");
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log_output)
                            .desired_rows(15)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY),
                    );
                });
        });

        // ===== Popup Notification (Modal) =====
        if self.show_popup {
            let avail = ctx.available_rect(); // zona vizibilă
            let max_size = egui::Vec2::new(avail.width() * 0.6, avail.height() * 0.4);

            egui::Window::new("⚠ Notification")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .max_size(max_size) // 🔥 se încadrează
                .default_size(max_size * 0.9)
                .frame(egui::Frame {
                    fill: egui::Color32::from_rgb(40, 40, 40),
                    stroke: egui::Stroke::new(
                        2.0,
                        if self.popup_message.contains("FORCE") {
                            egui::Color32::RED
                        } else {
                            egui::Color32::from_rgb(57, 255, 20)
                        },
                    ),
                    inner_margin: egui::Margin::same(12.0),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(
                                    egui::RichText::new(&self.popup_message)
                                        .color(egui::Color32::WHITE)
                                        .size(18.0)
                                        .strong(),
                                );

                                ui.add_space(10.0);

                                if self.popup_message.contains("FORCE") {
                                    ui.horizontal(|ui| {
                                        if ui.button("✅ YES (Force)").clicked() {
                                            let result = crate::commands::remove_all_apps(true);
                                            self.bulk_report = result;
                                            self.show_bulk_report = true;
                                            self.show_popup = false;
                                        }
                                        if ui.button("❌ NO").clicked() {
                                            self.show_popup = false;
                                        }
                                    });
                                } else if self.popup_message.contains("Confirm bulk removal") {
                                    ui.horizontal(|ui| {
                                        if ui.button("✅ YES").clicked() {
                                            let result = crate::commands::remove_all_apps(false);
                                            self.bulk_report = result;
                                            self.show_bulk_report = true;
                                            self.show_popup = false;
                                        }
                                        if ui.button("❌ NO").clicked() {
                                            self.show_popup = false;
                                        }
                                    });
                                } else {
                                    if ui.button("OK").clicked() {
                                        self.show_popup = false;
                                    }
                                }

                                ui.add_space(8.0);
                                if ui.button("❎ Close").clicked() {
                                    self.show_popup = false;
                                }
                            });
                        });
                });
        }


        // ===== Bulk Removal Report Popup =====
        if self.show_bulk_report {
            let avail = ctx.available_rect();
            let max_size = egui::Vec2::new(avail.width() * 0.9, avail.height() * 0.8);

            egui::Window::new("📋 Bulk Removal Report")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .max_size(max_size) // 🔥 încape în aplicație
                .default_size(max_size * 0.9)
                .frame(egui::Frame {
                    fill: egui::Color32::from_rgb(25, 25, 60),
                    stroke: egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE),
                    inner_margin: egui::Margin::same(12.0),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new("Results of bulk app removal")
                                .color(egui::Color32::LIGHT_BLUE)
                                .size(18.0)
                                .strong(),
                        );

                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true) // 🔥 mereu vezi ultima linie
                            .show(ui, |ui| {
                                for line in self.bulk_report.lines() {
                                    if line.contains("✅") {
                                        ui.colored_label(egui::Color32::from_rgb(57, 255, 20), line);
                                    } else if line.contains("❌") {
                                        ui.colored_label(egui::Color32::RED, line);
                                    } else if line.contains("⚠") {
                                        ui.colored_label(egui::Color32::YELLOW, line);
                                    } else if line.contains("⏱") {
                                        ui.colored_label(egui::Color32::LIGHT_BLUE, line);
                                    } else {
                                        ui.label(line);
                                    }
                                }
                                // 🔥 scroll forțat la ultima linie
                                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                            });

                        ui.add_space(10.0);
                        if ui.button("❎ Close").clicked() {
                            self.show_bulk_report = false;
                        }
                    });
                });
        }


    }
}
