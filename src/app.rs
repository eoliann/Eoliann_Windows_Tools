use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use eframe::egui;
use crate::tabs::info::InfoState;
use egui::{Color32, RichText, Vec2};

use crate::tabs::{info, tools, disk_health, install, winapp_removal, customize_preferences, settings};
use eframe::egui::TextureHandle;

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
    DiskHealth,
    Install,
    WinAppRemoval,
    CustomizePreferences,
    Settings,
}

pub struct App {
    pub latest_release: Option<crate::utils::GithubRelease>,
    pub update_available: bool,
    pub show_update_window: bool,
    page: Page,
    log: Arc<Mutex<String>>,
    show_popup: bool,
    popup_message: String,

    // prefs
    pub enable_tooltips: bool,
    pub auto_check_updates: bool,

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
    pub taskbar_widgets_enabled: bool,
    pub taskbar_widgets_prefs_loaded: bool,

    pub verbose_logon_enabled: bool,
    pub verbose_logon_prefs_loaded: bool,

    pub bitlocker_protection_on: bool,
    pub bitlocker_prefs_loaded: bool,

    // Info tab state
    pub info_state: Arc<Mutex<InfoState>>,

    // Disk Health tab state
    pub disk_health_state: disk_health::DiskHealthState,

    // Icons: textures loaded lazily at first `update` call
    pub icons: HashMap<String, TextureHandle>,

    pub general_prefs_loaded: bool,
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
            info_state: Arc::new(Mutex::new(InfoState::new())),
            disk_health_state: disk_health::DiskHealthState::default(),
            icons: HashMap::new(),
            general_prefs_loaded: false,
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
            show_update_window: update_available,
            info_state: Arc::new(Mutex::new(InfoState::new())),
            ..Self::default()
        }
    }

    fn clear_log(&self) {
        if let Ok(mut lg) = self.log.lock() {
            lg.clear();
        }
    }

    fn ensure_icons_loaded(&mut self, ctx: &egui::Context) {
        if !self.icons.is_empty() {
            return;
        }

        macro_rules! try_insert {
            ($key:expr, $bytes:expr) => {
                if let Some(tx) = load_png_from_bytes(ctx, $key, $bytes) {
                    self.icons.insert($key.to_string(), tx);
                } else {
                    eprintln!("Failed to load icon: {}", $key);
                }
            };
        }

        try_insert!("windows", include_bytes!("../assets/icons/windows.png") as &'static [u8]);
        try_insert!("system", include_bytes!("../assets/icons/system.png") as &'static [u8]);
        try_insert!("processor", include_bytes!("../assets/icons/processor.png") as &'static [u8]);
        try_insert!("memory", include_bytes!("../assets/icons/memory.png") as &'static [u8]);
        try_insert!("storage", include_bytes!("../assets/icons/storage.png") as &'static [u8]);
        try_insert!("graphics", include_bytes!("../assets/icons/graphics.png") as &'static [u8]);
        try_insert!("apps", include_bytes!("../assets/icons/apps.png") as &'static [u8]);
        try_insert!("processes", include_bytes!("../assets/icons/processes.png") as &'static [u8]);
        try_insert!("services", include_bytes!("../assets/icons/services.png") as &'static [u8]);
        try_insert!("network", include_bytes!("../assets/icons/network.png") as &'static [u8]);
        try_insert!("performance", include_bytes!("../assets/icons/performance.png") as &'static [u8]);
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.heading(RichText::new("Eoliann Windows Tools").color(Color32::from_rgb(0, 255, 140)));
        ui.add_space(10.0);
        ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));

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

        // if btn(ui, "Info", self.page == Page::Info).clicked() {
        //     self.page = Page::Info;
        // }
        // if btn(ui, "🛠 Tools", self.page == Page::Tools).clicked() {
        //     self.page = Page::Tools;
        // }
        // if btn(ui, "💽 Disk Health", self.page == Page::DiskHealth).clicked() {
        //     self.page = Page::DiskHealth;
        // }
        // if btn(ui, "Install", self.page == Page::Install).clicked() {
        //     self.page = Page::Install;
        // }
        // if btn(ui, "WinApp Removal", self.page == Page::WinAppRemoval).clicked() {
        //     self.page = Page::WinAppRemoval;
        // }
        // if btn(ui, "Customize Preferences", self.page == Page::CustomizePreferences).clicked() {
        //     self.page = Page::CustomizePreferences;
        // }
        // if btn(ui, "Settings", self.page == Page::Settings).clicked() {
        //     self.page = Page::Settings;
        // }

        if btn(ui, "❓ Info", self.page == Page::Info).clicked() {
            self.page = Page::Info;
        }

        if btn(ui, "🛠 Tools", self.page == Page::Tools).clicked() {
            self.page = Page::Tools;
        }

        if btn(ui, "💽 Disk Health", self.page == Page::DiskHealth).clicked() {
            self.page = Page::DiskHealth;
        }

        if btn(ui, "📦 Install", self.page == Page::Install).clicked() {
            self.page = Page::Install;
        }

        if btn(ui, "🗑 WinApp Removal", self.page == Page::WinAppRemoval).clicked() {
            self.page = Page::WinAppRemoval;
        }

        if btn(ui, "🔍 Customize Preferences", self.page == Page::CustomizePreferences).clicked() {
            self.page = Page::CustomizePreferences;
        }

        if btn(ui, "🔧 Settings", self.page == Page::Settings).clicked() {
            self.page = Page::Settings;
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.separator();

            let button_style = egui::RichText::new("💖 Donate")
                .color(egui::Color32::from_rgb(57, 255, 20))
                .strong();

            if ui.button(button_style).clicked() {
                let _ = webbrowser::open("https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G");
            }

            if ui.button("Clear Log").clicked() {
                self.clear_log();
            }
        });
    }

    fn log_view(&self, ui: &mut egui::Ui) {
        let text = { self.log.lock().unwrap().clone() };

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(text).monospace());
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_neon_theme(ctx);
        self.ensure_icons_loaded(ctx);

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                self.sidebar(ui);
            });
        });

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
                        self.show_update_window = false;
                    }
                });
        }

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
                    self.log_view(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let avail_h = ui.available_height();
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
                                reset_in_progress: false,
                                reset_aggressive: false,
                                last_message: String::new(),
                            });
                        }
                        Page::DiskHealth => {
                            disk_health::show_disk_health(ui, &self.log, &mut self.disk_health_state);
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
                                &mut self.enable_tooltips,
                                &mut self.auto_check_updates,
                                &mut self.general_prefs_loaded,
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
                            info::show_info(ui, &self.log, self.update_available, self.latest_release.as_ref(), &self.info_state, &self.icons);
                        }
                        Page::Settings => { settings::show_settings(ui, &self.log); }
                    }

                    ui.add_space(6.0);
                });
        });

        if self.show_popup {
            egui::Window::new("Confirm / Log")
                .collapsible(false)
                .resizable(true)
                .default_size([800.0, 600.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(&self.popup_message);

                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 50.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let text = { self.log.lock().unwrap().clone() };
                                ui.label(egui::RichText::new(text).monospace());
                            });

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() { self.show_popup = false; }
                            if ui.button("Clear Log").clicked() { self.clear_log(); }
                        });
                    });
                });
        }
    }
}

// top of file
use image;

fn load_png_from_bytes(
    ctx: &egui::Context,
    name: &str,
    bytes: &'static [u8],
) -> Option<egui::TextureHandle> {
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let img = img.to_rgba8();
            let (w, h) = (img.width() as usize, img.height() as usize);
            let pixels = img.into_vec(); // RGBA u8
            let color_image = egui::ColorImage::from_rgba_unmultiplied([w, h], &pixels);
            let options = egui::TextureOptions::LINEAR;
            Some(ctx.load_texture(name.to_owned(), color_image, options))
        }
        Err(err) => {
            eprintln!("load_png_from_bytes failed for {}: {}", name, err);
            None
        }
    }
}
