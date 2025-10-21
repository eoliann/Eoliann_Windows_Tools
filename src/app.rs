use std::sync::{Arc, Mutex};

use eframe::egui;
use egui::{Color32, RichText, Vec2};

use crate::tabs::{info, tools, install, winapp_removal, settings};

// ---------------- THEME (verde fluorescent) ----------------
fn apply_neon_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.spacing.item_spacing = Vec2::new(6.0, 6.0);
    style.spacing.button_padding = Vec2::new(10.0, 8.0);

    let neon = Color32::from_rgb(0, 255, 140);
    let neon_hover = Color32::from_rgb(20, 240, 160);
    let neon_active = Color32::from_rgb(40, 220, 180);

    let mut visuals = egui::Visuals::dark();

    visuals.override_text_color = Some(Color32::WHITE);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(20, 20, 24);
    visuals.panel_fill = Color32::from_rgb(14, 14, 18);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 36);
    visuals.widgets.inactive.fg_stroke.color = neon;

    visuals.widgets.hovered.bg_fill = neon_hover.linear_multiply(0.12);
    visuals.widgets.hovered.fg_stroke.color = neon_hover;
    visuals.widgets.hovered.bg_stroke.color = neon_hover;

    visuals.widgets.active.bg_fill = neon_active.linear_multiply(0.18);
    visuals.widgets.active.fg_stroke.color = neon_active;
    visuals.widgets.active.bg_stroke.color = neon_active;

    visuals.selection.bg_fill = neon.linear_multiply(0.25);
    visuals.selection.stroke.color = neon;

    style.visuals = visuals;
    ctx.set_style(style);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Info,
    Tools,
    Install,
    WinAppRemoval,
    Settings,
}

pub struct App {
    pub latest_release: Option<crate::utils::GithubRelease>,
    pub update_available: bool,
    page: Page,
    log: Arc<Mutex<String>>,
    show_popup: bool,
    popup_message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            latest_release: None,
            update_available: false,
            page: Page::Info,
            log: Arc::new(Mutex::new(String::new())),
            show_popup: false,
            popup_message: String::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        let current_version = env!("CARGO_PKG_VERSION");

        let mut update_available = false;
        let mut latest_release = None;

        if let Some(release) = crate::utils::check_latest_version() {
            if crate::utils::is_update_available(current_version, &release.tag_name) {
                update_available = true;
                latest_release = Some(release);
            }
        }

        Self {
            latest_release,
            update_available,
            ..Self::default()
        }
    }

    fn clear_log(&self) {
        if let Ok(mut lg) = self.log.lock() {
            lg.clear();
        }
    }

    // -------------- SIDEBAR --------------
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.heading(RichText::new("Eoliann Win Tools").color(Color32::from_rgb(0, 255, 140)));
        ui.add_space(10.0);
        ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));

        // show update info if available (uses latest_release and update_available)
        // show update info if available (simple inline indicator + open button)
        if self.update_available {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(0, 255, 140), "⬆ Update available");
                if ui.small_button("Open").clicked() {
                    if let Some(release) = &self.latest_release {
                        let _ = webbrowser::open(release.html_url.as_str());
                    } else {
                        let _ = webbrowser::open("https://github.com/eoliann/");
                    }
                }
            });
            ui.add_space(6.0);
        }


        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        let btn = |ui: &mut egui::Ui, label: &str, selected: bool| {
            let text = if selected {
                RichText::new(label).strong().color(Color32::from_rgb(0, 255, 140))
            } else {
                RichText::new(label)
            };
            ui.selectable_label(selected, text)
        };

        if btn(ui, "Info", self.page == Page::Info).clicked() {
            self.page = Page::Info;
        }
        if btn(ui, "Tools", self.page == Page::Tools).clicked() {
            self.page = Page::Tools;
        }
        if btn(ui, "Install", self.page == Page::Install).clicked() {
            self.page = Page::Install;
        }
        if btn(ui, "WinApp Removal", self.page == Page::WinAppRemoval).clicked() {
            self.page = Page::WinAppRemoval;
        }
        if btn(ui, "Settings", self.page == Page::Settings).clicked() {
            self.page = Page::Settings;
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.separator();

            let button_style = egui::RichText::new("💖 Donate")
                .color(egui::Color32::from_rgb(57, 255, 20)) // verde neon
                .strong();

            if ui.button(button_style).clicked() {
                let _ = webbrowser::open("https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G");
            }

            if ui.button("Clear Log").clicked() {
                self.clear_log();
            }
        });
    }

    // -------------- LOG VIEW (cu scroll & auto-scroll) --------------
    fn log_view(&self, ui: &mut egui::Ui) {
        let text = { self.log.lock().unwrap().clone() };

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true) // 🔥 stă lipit jos
            .show(ui, |ui| {
                ui.label(egui::RichText::new(text).monospace());
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_neon_theme(ctx);

        // Sidebar (lăsăm neschimbat)
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                self.sidebar(ui);
            });
        });

                // --- Update popup (foloseste ctx) ---
        if self.update_available {
            egui::Window::new("Update Available")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("A new version is available!");
                    if let Some(release) = &self.latest_release {
                        ui.label(format!("Latest version: {}", release.tag_name));
                        if ui.button("Open release on GitHub").clicked() {
                            let _ = webbrowser::open(release.html_url.as_str());
                        }
                    } else {
                        if ui.button("Open GitHub").clicked() {
                            let _ = webbrowser::open("https://github.com/eoliann/");
                        }
                    }
                    if ui.button("Close").clicked() {
                        self.update_available = false;
                    }
                });
        }

        // === 1) Bottom panel FIRST (rezervă spațiul) ===
        egui::TopBottomPanel::bottom("log_bottom_panel")
            .resizable(true)
            .default_height(220.0)
            .min_height(80.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Output / Log:").strong());
                    if ui.button("Clear").clicked() {
                        self.clear_log();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.small_button("Pop out").clicked() {
                            self.show_popup = true;
                            self.popup_message = String::from("Log popped out.");
                        }
                    });
                });

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    // reutilizăm log_view care conține ScrollArea + stick_to_bottom
                    self.log_view(ui);
                });
            });

        // === 2) Central panel AFTER bottom (va primi doar spațiul rămas) ===
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail_h = ui.available_height(); // acum corect — după bottom panel
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(avail_h)
                .show(ui, |ui| {
                    match self.page {
                        Page::Tools => {
                            tools::show_tools(ui, &self.log, &mut self.show_popup, &mut self.popup_message);
                        }
                        Page::Install => { install::show_install(ui, &self.log); }
                        Page::WinAppRemoval => {
                            winapp_removal::show_winapp_removal(ui, &self.log, &mut self.show_popup, &mut self.popup_message);
                        }
                        // Page::Info => { info::show_info(ui, &self.log); }
                        Page::Info => {
                            info::show_info(ui, &self.log, self.update_available, self.latest_release.as_ref());
                        }
                        Page::Settings => { settings::show_settings(ui, &self.log); }
                    }

                    ui.add_space(6.0);
                });
        });

        // popup / update windows etc. (rămân neschimbate)
        if self.show_popup {
            egui::Window::new("Confirm / Log")
                .collapsible(false)
                .resizable(true)
                .default_size([600.0, 400.0])
                .show(ctx, |ui| {
                    ui.label(&self.popup_message);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let text = { self.log.lock().unwrap().clone() };
                        ui.label(egui::RichText::new(text).monospace());
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() { self.show_popup = false; }
                        if ui.button("Clear Log").clicked() { self.clear_log(); }
                    });
                });
        }

        // update window (unchanged)...
    }
}
