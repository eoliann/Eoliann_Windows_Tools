use eframe::egui;
use egui::{Color32, RichText};
use std::sync::{Arc, Mutex};

use crate::commands::{self, RestorePointInfo};

#[derive(Default)]
pub struct HealthState {
    pub initialized: bool,
    pub hibernation_enabled: bool,
    pub restore_points: Vec<RestorePointInfo>,
    pub selected_restore_point: Option<u32>,
    pub system_restore_used_gb: f64,
    pub show_restore_confirm: bool,
    pub last_battery_report_path: Option<String>,
    pub show_battery_report_prompt: bool,
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

fn refresh_restore_points(state: &mut HealthState, log: &Arc<Mutex<String>>) {
    match commands::list_restore_points() {
        Ok(points) => {
            state.restore_points = points;

            if let Some(selected) = state.selected_restore_point {
                if !state
                    .restore_points
                    .iter()
                    .any(|rp| rp.sequence_number == selected)
                {
                    state.selected_restore_point =
                        state.restore_points.first().map(|rp| rp.sequence_number);
                }
            } else {
                state.selected_restore_point =
                    state.restore_points.first().map(|rp| rp.sequence_number);
            }
        }
        Err(err) => {
            append_log(log, format!("Failed to load restore points: {err}"));
            state.restore_points.clear();
            state.selected_restore_point = None;
        }
    }

    state.system_restore_used_gb = commands::get_system_restore_used_space_gb().unwrap_or(0.0);
}

fn refresh_all(state: &mut HealthState, log: &Arc<Mutex<String>>) {
    match commands::is_hibernation_enabled() {
        Ok(enabled) => state.hibernation_enabled = enabled,
        Err(err) => append_log(log, format!("Failed to detect hibernation state: {err}")),
    }

    refresh_restore_points(state, log);
    state.initialized = true;
}

pub fn show_health(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    state: &mut HealthState,
) {
    if !state.initialized {
        refresh_all(state, log);
    }

    let neon = Color32::from_rgb(0, 255, 140);
    let yellow = Color32::from_rgb(255, 215, 0);

    ui.heading(RichText::new("❤ Health").color(yellow));
    ui.label("Power, cleanup and diagnostics.");
    ui.add_space(10.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Hibernation").color(yellow).size(18.0));
        ui.label("Enable or disable Windows hibernation.");
        ui.add_space(6.0);

        let status = if state.hibernation_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        ui.colored_label(neon, format!("Current status: {status}"));

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let button_label = if state.hibernation_enabled {
                "Disable"
            } else {
                "Enable"
            };

            if ui.button(button_label).clicked() {
                let result = if state.hibernation_enabled {
                    commands::disable_hibernation()
                } else {
                    commands::enable_hibernation()
                };

                append_log(log, &result);

                if let Ok(enabled) = commands::is_hibernation_enabled() {
                    state.hibernation_enabled = enabled;
                }
            }

            if ui.button("Refresh").clicked() {
                match commands::is_hibernation_enabled() {
                    Ok(enabled) => state.hibernation_enabled = enabled,
                    Err(err) => append_log(log, format!("Failed to refresh hibernation state: {err}")),
                }
            }
        });
    });

    ui.add_space(8.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Cleanup").color(yellow).size(18.0));
        ui.label("Storage settings and System Restore cleanup.");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Storage");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Open").clicked() {
                    let result = commands::open_storage_settings();
                    append_log(log, &result);
                }
            });
        });

        ui.separator();

        ui.label(RichText::new("System restore").color(yellow).size(18.0));
        ui.small(format!(
            "Total used space: {:.2} GB",
            state.system_restore_used_gb
        ));
        ui.small(format!(
            "Available restore points: {}",
            state.restore_points.len()
        ));

        ui.add_space(6.0);

        if state.restore_points.is_empty() {
            ui.colored_label(Color32::GRAY, "No restore points found.");
        } else {
            egui::ComboBox::from_id_salt("restore_points_combo")
                .width(520.0)
                .selected_text(
                    state
                        .selected_restore_point
                        .and_then(|seq| {
                            state.restore_points.iter().find(|rp| rp.sequence_number == seq)
                        })
                        .map(|rp| {
                            format!(
                                "#{} | {} | {}",
                                rp.sequence_number, rp.created, rp.description
                            )
                        })
                        .unwrap_or_else(|| "Select a restore point".to_string()),
                )
                .show_ui(ui, |ui| {
                    for rp in &state.restore_points {
                        ui.selectable_value(
                            &mut state.selected_restore_point,
                            Some(rp.sequence_number),
                            format!(
                                "#{} | {} | {}",
                                rp.sequence_number, rp.created, rp.description
                            ),
                        );
                    }
                });

            ui.add_space(6.0);

            if let Some(selected) = state
                .selected_restore_point
                .and_then(|seq| state.restore_points.iter().find(|rp| rp.sequence_number == seq))
            {
                ui.group(|ui| {
                    ui.label(format!("Sequence number: {}", selected.sequence_number));
                    ui.label(format!("Created: {}", selected.created));
                    ui.label(format!("Description: {}", selected.description));
                    ui.label(format!("Restore point type: {}", selected.restore_point_type));
                    ui.label(format!("Event type: {}", selected.event_type));
                });
            }
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                refresh_restore_points(state, log);
            }

            let can_delete = state.selected_restore_point.is_some();
            if ui
                .add_enabled(can_delete, egui::Button::new("Delete selected"))
                .clicked()
            {
                state.show_restore_confirm = true;
            }
        });
    });

    ui.add_space(8.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Battery report").color(yellow).size(18.0));
        ui.label("Generate the Windows battery report and save it to Documents.");
        ui.add_space(6.0);

        if ui.button("Generate").clicked() {
            match commands::generate_battery_report() {
                Ok(path) => {
                    append_log(log, format!("Battery report created:\n{}", path));
                    state.last_battery_report_path = Some(path);
                    state.show_battery_report_prompt = true;
                }
                Err(err) => {
                    append_log(log, format!("Failed to generate battery report: {}", err));
                }
            }
        }

        if let Some(path) = &state.last_battery_report_path {
            ui.small(format!("Last report: {}", path));
        }
    });

    ui.add_space(8.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Memory diagnostic").color(yellow).size(18.0));
        ui.label("Launch Windows Memory Diagnostic.");
        ui.add_space(6.0);

        if ui.button("Check").clicked() {
            let result = commands::launch_memory_diagnostic();
            append_log(log, &result);
        }
    });

    if state.show_restore_confirm {
        let selected = state
            .selected_restore_point
            .and_then(|seq| state.restore_points.iter().find(|rp| rp.sequence_number == seq))
            .cloned();

        let mut open = true;
        let mut confirmed_delete = false;

        egui::Window::new("Delete restore point")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .default_size([420.0, 240.0])
            .frame(
                egui::Frame::window(ui.style())
                    .stroke(egui::Stroke::new(1.5, Color32::from_rgb(0, 255, 140))),
            )
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.add_space(8.0);
                ui.colored_label(
                    yellow,
                    "Only the selected restore point will be removed.",
                );
                ui.add_space(10.0);

                if let Some(rp) = &selected {
                    ui.label(format!("Sequence number: {}", rp.sequence_number));
                    ui.label(format!("Created: {}", rp.created));
                    ui.label(format!("Description: {}", rp.description));
                } else {
                    ui.colored_label(Color32::RED, "No restore point is selected.");
                }

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(selected.is_some(), egui::Button::new("Delete"))
                        .clicked()
                    {
                        confirmed_delete = true;
                    }

                    if ui.button("Close").clicked() {
                        state.show_restore_confirm = false;
                    }
                });
            });

        if !open {
            state.show_restore_confirm = false;
        }

        if confirmed_delete {
            state.show_restore_confirm = false;

            if let Some(rp) = selected {
                match commands::delete_restore_point(rp.sequence_number) {
                    Ok(msg) => append_log(log, msg),
                    Err(err) => append_log(
                        log,
                        format!(
                            "Failed to delete restore point #{}: {}",
                            rp.sequence_number, err
                        ),
                    ),
                }

                refresh_restore_points(state, log);
            }
        }
    }

    if state.show_battery_report_prompt {
        let mut open = true;
        let mut open_clicked = false;
        let path = state.last_battery_report_path.clone();

        egui::Window::new("Battery report")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .default_size([430.0, 210.0])
            .frame(
                egui::Frame::window(ui.style())
                    .stroke(egui::Stroke::new(1.5, Color32::from_rgb(0, 255, 140))),
            )
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.add_space(8.0);
                ui.colored_label(yellow, "Battery report created successfully.");
                ui.add_space(10.0);

                if let Some(path) = &path {
                    ui.label("Saved to:");
                    ui.small(path);
                }

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(path.is_some(), egui::Button::new("Open"))
                        .clicked()
                    {
                        open_clicked = true;
                    }

                    if ui.button("Close").clicked() {
                        state.show_battery_report_prompt = false;
                    }
                });
            });

        if !open {
            state.show_battery_report_prompt = false;
        }

        if open_clicked {
            state.show_battery_report_prompt = false;

            if let Some(path) = &state.last_battery_report_path {
                match commands::open_battery_report(path) {
                    Ok(msg) => append_log(log, msg),
                    Err(err) => append_log(log, format!("Failed to open battery report: {}", err)),
                }
            }
        }
    }
}