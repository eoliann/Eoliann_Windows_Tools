use crate::tabs::{info::show_info, tools::show_tools, settings::show_settings};
use egui::{FontDefinitions, FontData, FontFamily, FontId, TextStyle};

pub struct App {
    active_tab: Tab,
    log_output: String,
    show_popup: bool,
    popup_message: String,
}

#[derive(PartialEq)]
enum Tab {
    Info,
    Tools,
    Settings,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_tab: Tab::Info,
            log_output: String::new(),
            show_popup: false,
            popup_message: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===== Fonts =====
        let mut fonts = FontDefinitions::default();

        // Adaugăm font custom JetBrainsMono
        fonts.font_data.insert(
            "JetBrainsMono".to_owned(),
            FontData::from_owned(include_bytes!("../assets/JetBrainsMono-Regular.ttf").to_vec()),
        );

        // Setăm Monospace să folosească JetBrainsMono
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "JetBrainsMono".to_owned());

        ctx.set_fonts(fonts);

        // ===== Text Styles =====
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(10.0, FontFamily::Proportional)),
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

            // ui.horizontal(|ui| {
                if ui.button("📊 INFO").clicked() {
                    self.active_tab = Tab::Info;
                    self.log_output = "📊 INFO Selected".to_string();
                }
                if ui.button("🛠 TOOLS").clicked() {
                    self.active_tab = Tab::Tools;
                    self.log_output = "🛠 TOOLS Selected".to_string();
                }
                if ui.button("⚙ SETTINGS").clicked() {
                    self.active_tab = Tab::Settings;
                    self.log_output = "⚙ SETTINGS Selected".to_string();
                }
            // });
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

        // ===== Popup Notification =====
        if self.show_popup {
            egui::Window::new("Notification")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(57, 255, 20),
                        &self.popup_message,
                    );
                    if ui.button("OK").clicked() {
                        self.show_popup = false;
                    }
                });
        }
    }
}
