use eframe::egui;
use egui::{Color32, RichText};
use std::sync::{Arc, Mutex};

use crate::commands::{self, StartupAppEntry};

#[derive(Default)]
pub struct PerformanceState {
    pub initialized: bool,

    pub ultimate_performance_enabled: bool,
    pub hags_enabled: bool,
    pub vbs_enabled: bool,
    pub restart_apps_enabled: bool,
    pub background_apps_enabled: bool,
    pub activity_history_enabled: bool,

    // pub visual_settings_open: bool,
    pub visual_effects_performance_enabled: bool,
    pub transparency_enabled: bool,

    // pub gaming_settings_open: bool,
    pub game_mode_enabled: bool,
    pub windowed_optimizations_enabled: bool,
    pub background_recording_enabled: bool,
    pub superfetch_enabled: bool,

    pub search_indexing_enabled: bool,
    pub delivery_optimization_enabled: bool,
    pub network_task_offload_enabled: bool,

    pub show_startup_apps_modal: bool,
    pub startup_apps: Vec<StartupAppEntry>,
}

fn append_log(log: &Arc<Mutex<String>>, text: impl AsRef<str>) {
    if let Ok(mut lg) = log.lock() {
        if !lg.is_empty() {
            lg.push('\n');
        }
        lg.push_str(text.as_ref());
        lg.push('\n');
    }
}

fn refresh_startup_apps(state: &mut PerformanceState, log: &Arc<Mutex<String>>) {
    match commands::list_startup_apps() {
        Ok(items) => state.startup_apps = items,
        Err(err) => {
            state.startup_apps.clear();
            append_log(log, format!("Failed to read startup apps: {err}"));
        }
    }
}

fn refresh_state(state: &mut PerformanceState, log: &Arc<Mutex<String>>) {
    macro_rules! load_bool {
        ($field:ident, $expr:expr) => {
            match $expr {
                Ok(v) => state.$field = v,
                Err(err) => append_log(log, format!("Failed to load {}: {}", stringify!($field), err)),
            }
        };
    }

    load_bool!(ultimate_performance_enabled, commands::get_ultimate_performance_enabled());
    load_bool!(hags_enabled, commands::get_hags_enabled());
    load_bool!(vbs_enabled, commands::get_vbs_enabled());
    load_bool!(restart_apps_enabled, commands::get_restart_apps_enabled());
    load_bool!(background_apps_enabled, commands::get_background_apps_enabled());
    load_bool!(activity_history_enabled, commands::get_activity_history_enabled());
    load_bool!(visual_effects_performance_enabled, commands::get_visual_effects_for_performance_enabled());
    load_bool!(transparency_enabled, commands::get_transparency_enabled());
    load_bool!(game_mode_enabled, commands::get_game_mode_enabled());
    load_bool!(windowed_optimizations_enabled, commands::get_windowed_optimizations_enabled());
    load_bool!(background_recording_enabled, commands::get_background_recording_enabled());
    load_bool!(superfetch_enabled, commands::get_superfetch_enabled());
    load_bool!(search_indexing_enabled, commands::get_search_indexing_enabled());
    load_bool!(delivery_optimization_enabled, commands::get_delivery_optimization_enabled());
    load_bool!(network_task_offload_enabled, commands::get_network_task_offload_enabled());

    refresh_startup_apps(state, log);
    state.initialized = true;
}

fn toggle_card<F>(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    state_value: &mut bool,
    read_more_url: Option<&str>,
    on_toggle: F,
) -> Option<String>
where
    F: FnOnce(bool) -> String,
{
    let mut changed_to: Option<bool> = None;

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).strong());
                ui.small(description);

                if let Some(url) = read_more_url {
                    if ui.small_button("Read more").clicked() {
                        let _ = webbrowser::open(url);
                    }
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut tmp = *state_value;
                if ui.checkbox(&mut tmp, "").changed() {
                    changed_to = Some(tmp);
                }
            });
        });
    });

    if let Some(new_value) = changed_to {
        *state_value = new_value;
        return Some(on_toggle(new_value));
    }

    None
}

pub fn show_performance(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    state: &mut PerformanceState,
) {
    if !state.initialized {
        refresh_state(state, log);
    }

    let yellow = Color32::from_rgb(255, 215, 0);

    ui.heading(RichText::new("⚡ Performance").color(yellow));
    ui.label("Power, graphics and startup settings.");
    ui.add_space(10.0);

    if ui.button("Refresh all states").clicked() {
        refresh_state(state, log);
        append_log(log, "Performance state refreshed.");
    }

    ui.add_space(8.0);

    if let Some(msg) = toggle_card(
        ui,
        "Ultimate performance power plan",
        "Switch between the app-managed Ultimate plan and Balanced.",
        &mut state.ultimate_performance_enabled,
        None,
        commands::set_ultimate_performance_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "HAGS (hardware-accelerated GPU scheduling)",
        "May reduce latency and CPU overhead. Restart usually required.",
        &mut state.hags_enabled,
        None,
        commands::set_hags_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "VBS (virtualization-based security)",
        "Turns VBS / memory-integrity-related registry flags on or off. Restart required.",
        &mut state.vbs_enabled,
        None,
        commands::set_vbs_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Startup apps").strong());
                ui.small("Manage classic Run startup entries.");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Configure").clicked() {
                    refresh_startup_apps(state, log);
                    state.show_startup_apps_modal = true;
                }
            });
        });
    });

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "Relaunch apps",
        "Automatically save restartable apps and relaunch them after sign-in.",
        &mut state.restart_apps_enabled,
        None,
        commands::set_restart_apps_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "Background apps",
        "Global background-app permission switch for the current user.",
        &mut state.background_apps_enabled,
        None,
        commands::set_background_apps_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "Activity history",
        "Uses the existing Activity History policies already present in the app.",
        &mut state.activity_history_enabled,
        None,
        |enabled| {
            if enabled {
                commands::enable_activity_history()
            } else {
                commands::disable_activity_history()
            }
        },
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::CollapsingHeader::new(RichText::new("Visual settings").strong())
            .default_open(false)
            .show(ui, |ui| {
                if let Some(msg) = toggle_card(
                    ui,
                    "Optimize visual effects for performance",
                    "Uses the existing performance visual profile from the app.",
                    &mut state.visual_effects_performance_enabled,
                    None,
                    commands::set_visual_effects_for_performance_enabled,
                ) {
                    append_log(log, msg);
                }

                ui.add_space(6.0);

                if let Some(msg) = toggle_card(
                    ui,
                    "Transparency",
                    "Windows transparency effects for the current user.",
                    &mut state.transparency_enabled,
                    None,
                    commands::set_transparency_enabled,
                ) {
                    append_log(log, msg);
                }
            });
    });

    ui.add_space(6.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::CollapsingHeader::new(RichText::new("Gaming settings").strong())
            .default_open(false)
            .show(ui, |ui| {
                if let Some(msg) = toggle_card(
                    ui,
                    "Game mode",
                    "Windows Game Mode.",
                    &mut state.game_mode_enabled,
                    None,
                    commands::set_game_mode_enabled,
                ) {
                    append_log(log, msg);
                }

                ui.add_space(6.0);

                if let Some(msg) = toggle_card(
                    ui,
                    "Windowed mode optimizations",
                    "Optimizations for windowed games. Restart the game after changes.",
                    &mut state.windowed_optimizations_enabled,
                    Some("https://support.microsoft.com/en-us/windows/optimizations-for-windowed-games-in-windows-11-3f006843-2c7e-4ed0-9a5e-f9389e535952"),
                    commands::set_windowed_optimizations_enabled,
                ) {
                    append_log(log, msg);
                }

                ui.add_space(6.0);

                if let Some(msg) = toggle_card(
                    ui,
                    "Background recording",
                    "Xbox Game Bar / Game DVR background capture.",
                    &mut state.background_recording_enabled,
                    None,
                    commands::set_background_recording_enabled,
                ) {
                    append_log(log, msg);
                }

                ui.add_space(6.0);

                if let Some(msg) = toggle_card(
                    ui,
                    "Superfetch",
                    "Controls the SysMain service.",
                    &mut state.superfetch_enabled,
                    None,
                    commands::set_superfetch_enabled,
                ) {
                    append_log(log, msg);
                }
            });
    });

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "Search indexing",
        "Controls the Windows Search indexing service.",
        &mut state.search_indexing_enabled,
        None,
        commands::set_search_indexing_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "Delivery optimization",
        "Controls Delivery Optimization sharing behavior.",
        &mut state.delivery_optimization_enabled,
        None,
        commands::set_delivery_optimization_enabled,
    ) {
        append_log(log, msg);
    }

    ui.add_space(6.0);

    if let Some(msg) = toggle_card(
        ui,
        "Network adapter onboard processor",
        "Controls TCP task offload.",
        &mut state.network_task_offload_enabled,
        None,
        commands::set_network_task_offload_enabled,
    ) {
        append_log(log, msg);
    }

    if state.show_startup_apps_modal {
        let mut open = true;
        let mut pending_toggle: Option<(String, String, bool)> = None;
        let neon = Color32::from_rgb(0, 255, 140);

        egui::Window::new("Startup apps")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(true)
            .movable(false)
            .default_size([560.0, 480.0])
            .frame(
                egui::Frame::window(ui.style())
                    .stroke(egui::Stroke::new(1.5, neon)),
            )
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.label("Classic Run startup entries");
                ui.small("This modal manages registry Run entries only.");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("Refresh").clicked() {
                        refresh_startup_apps(state, log);
                    }

                    if ui.button("Close").clicked() {
                        state.show_startup_apps_modal = false;
                    }
                });

                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if state.startup_apps.is_empty() {
                        ui.colored_label(Color32::GRAY, "No startup entries found.");
                    } else {
                        for item in &state.startup_apps {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(&item.name).strong());
                                        ui.small(format!("Scope: {}", item.scope));
                                        ui.small(&item.command);
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let mut enabled = item.enabled;
                                            if ui.checkbox(&mut enabled, "").changed() {
                                                pending_toggle = Some((
                                                    item.scope.clone(),
                                                    item.name.clone(),
                                                    enabled,
                                                ));
                                            }
                                        },
                                    );
                                });
                            });

                            ui.add_space(6.0);
                        }
                    }
                });
            });

        if !open {
            state.show_startup_apps_modal = false;
        }

        if let Some((scope, name, enabled)) = pending_toggle {
            match commands::set_startup_app_enabled(&scope, &name, enabled) {
                Ok(msg) => append_log(log, msg),
                Err(err) => append_log(log, format!("Failed to update startup app '{}': {}", name, err)),
            }
            refresh_startup_apps(state, log);
        }
    }
}