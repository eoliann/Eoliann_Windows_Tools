use std::sync::{Arc, Mutex};

use eframe::egui;
use egui::{Color32, RichText, Vec2};

use crate::tabs::{info, tools, install, winapp_removal, customize_preferences, settings};

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
    CustomizePreferences,
    Settings,
}

pub struct App {
    pub latest_release: Option<crate::utils::GithubRelease>,
    pub update_available: bool,      // rămâne sursa de adevăr pentru notificări
    pub show_update_window: bool,    // control vizibilitate popup update (separat)
    page: Page,
    log: Arc<Mutex<String>>,
    show_popup: bool,
    popup_message: String,

    // stare pentru Customize Preferences (persistată în App)
    pub start_with_windows: bool,
    pub enable_tooltips: bool,
    pub auto_check_updates: bool,

    // stare pentru Customize Preferences (persistată în App)
    pub mouse_accel_enabled: bool,
    pub mouse_prefs_loaded: bool,
    pub numlock_enabled: bool,
    pub numlock_prefs_loaded: bool,
    pub taskbar_search_enabled: bool,
    pub taskbar_search_prefs_loaded: bool,
    pub snap_enabled: bool,
    pub snap_prefs_loaded: bool,
    pub sticky_enabled: bool,
    pub sticky_prefs_loaded: bool,
    pub taskview_enabled: bool,
    pub taskview_prefs_loaded: bool,
    // taskbar widgets
    pub taskbar_widgets_enabled: bool,
    pub taskbar_widgets_prefs_loaded: bool,

    // verbose logon (system / HKLM)
    pub verbose_logon_enabled: bool,
    pub verbose_logon_prefs_loaded: bool,

    // bitlocker
    pub bitlocker_protection_on: bool,
    pub bitlocker_prefs_loaded: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            latest_release: None,
            update_available: false,
            show_update_window: false,
            page: Page::Info,
            log: Arc::new(Mutex::new(String::new())),
            show_popup: false,
            popup_message: String::new(),

            // inițializări noi:
            start_with_windows: false,
            enable_tooltips: true,
            auto_check_updates: true,
            mouse_accel_enabled: false,
            mouse_prefs_loaded: false,
            numlock_enabled: false,
            numlock_prefs_loaded: false,
            taskbar_search_enabled: false,
            taskbar_search_prefs_loaded: false,
            snap_enabled: false,
            snap_prefs_loaded: false,
            sticky_enabled: false,
            sticky_prefs_loaded: false,
            taskview_enabled: false,
            taskview_prefs_loaded: false,
            taskbar_widgets_enabled: false,
            taskbar_widgets_prefs_loaded: false,

            verbose_logon_enabled: false,
            verbose_logon_prefs_loaded: false,

            bitlocker_protection_on: false,
            bitlocker_prefs_loaded: false,

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
            show_update_window: update_available, // afișăm popup inițial doar dacă există update
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
        ui.heading(RichText::new("Eoliann Windows Tools").color(Color32::from_rgb(0, 255, 140)));
        ui.add_space(10.0);
        ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));

        // show update info if available (uses latest_release and update_available)
        if self.update_available {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(0, 255, 140), "⬆ Update available");
                if ui.small_button("Open").clicked() {
                    // păstrăm comportamentul de a deschide GitHub; dacă vrei popup în loc, schimbă aici
                    if let Some(release) = &self.latest_release {
                        let _ = webbrowser::open(release.html_url.as_str());
                    } else {
                        let _ = webbrowser::open("https://github.com/eoliann/");
                    }
                }
                // opțiune: buton pentru a deschide popup explicit
                if ui.small_button("Details").clicked() {
                    self.show_update_window = true;
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
        if btn(ui, "Customize Preferences", self.page == Page::CustomizePreferences).clicked() {
            self.page = Page::CustomizePreferences;
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

        // --- Update popup (folosește flag separat show_update_window) ---
        if self.show_update_window {
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
                        // FIX: închidem doar popup-ul, nu ascundem notificările globale
                        self.show_update_window = false;
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
                            tools::show_tools(ui, &self.log, &mut self.show_popup, &mut self.popup_message, &mut tools::ToolsState {
                                show_hidden_state: false,
                                show_file_ext_state: false,
                                pending_reset_rx: None,
                                reset_in_progress: false, // This field exists in ToolsState
                                reset_aggressive: false,
                                last_message: String::new(),
                            });
                        }
                        Page::Install => { install::show_install(ui, &self.log); }
                        Page::WinAppRemoval => {
                            winapp_removal::show_winapp_removal(ui, &self.log, &mut self.show_popup, &mut self.popup_message);
                        }
                        Page::CustomizePreferences => {
                            customize_preferences::show_customize_preferences(
                                ui,
                                &self.log,
                                &mut self.show_popup,
                                &mut self.popup_message,
                                &mut self.start_with_windows,
                                &mut self.enable_tooltips,
                                &mut self.auto_check_updates,
                                &mut self.mouse_accel_enabled,
                                &mut self.mouse_prefs_loaded,
                                &mut self.numlock_enabled,
                                &mut self.numlock_prefs_loaded,
                                &mut self.taskbar_search_enabled,
                                &mut self.taskbar_search_prefs_loaded,
                                &mut self.taskbar_widgets_enabled,
                                &mut self.taskbar_widgets_prefs_loaded,
                                &mut self.snap_enabled,
                                &mut self.snap_prefs_loaded,
                                &mut self.sticky_enabled,
                                &mut self.sticky_prefs_loaded,
                                &mut self.taskview_enabled,
                                &mut self.taskview_prefs_loaded,
                                &mut self.verbose_logon_enabled,
                                &mut self.verbose_logon_prefs_loaded,
                                &mut self.bitlocker_protection_on,
                                &mut self.bitlocker_prefs_loaded,
                            );
                        }
                        Page::Info => {
                            info::show_info(ui, &self.log, self.update_available, self.latest_release.as_ref());
                        }
                        Page::Settings => { settings::show_settings(ui, &self.log); }
                    }

                    ui.add_space(6.0);
                });
        });

        // popup / update windows etc.
        if self.show_popup {
            egui::Window::new("Confirm / Log")
                .collapsible(false)
                .resizable(true)
                .default_size([800.0, 600.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(&self.popup_message);
                        
                        // ScrollArea cu înălțime limitată - lasă spațiu pentru butoane
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 50.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let text = { self.log.lock().unwrap().clone() };
                                ui.label(egui::RichText::new(text).monospace());
                            });
                        
                        ui.add_space(10.0);
                        
                        // Butoanele rămân întotdeauna vizibile jos
                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() { self.show_popup = false; }
                            if ui.button("Clear Log").clicked() { self.clear_log(); }
                        });
                    });
                });
        }
    }
}
