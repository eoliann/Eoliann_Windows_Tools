use eframe::egui;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use crate::debloat::{self, AppState, Status, APPS};
use crate::utils;

// -------------------- Persistent state --------------------

#[derive(Default)]
struct PageState {
    status: Option<Status>,
    loading: bool,
    busy: bool,
    loaded_once: bool,
    error: Option<String>,
    selected: HashSet<usize>,
    show_absent: bool,
    confirm: bool,
}

static STATE: OnceLock<Mutex<PageState>> = OnceLock::new();

fn state() -> &'static Mutex<PageState> {
    STATE.get_or_init(|| Mutex::new(PageState::default()))
}

const GROUPS_LEFT: &[&str] = &["Communication", "AI & Bing", "Productivity", "Promoted apps"];
const GROUPS_RIGHT: &[&str] = &["Media & Creativity", "Games & Xbox", "System & Misc"];

const NEON: egui::Color32 = egui::Color32::from_rgb(0, 255, 140);
const YELLOW: egui::Color32 = egui::Color32::from_rgb(255, 220, 80);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 160, 0);

fn append_line(log: &Arc<Mutex<String>>, line: impl AsRef<str>) {
    let line = line.as_ref();
    if let Ok(mut lg) = log.lock() {
        lg.push_str(line);
        if !line.ends_with('\n') {
            lg.push('\n');
        }
    }
}

fn refresh(log: Arc<Mutex<String>>) {
    {
        let mut st = state().lock().unwrap();
        if st.loading {
            return;
        }
        st.loading = true;
        st.loaded_once = true;
        st.error = None;
    }
    thread::spawn(move || {
        let result = debloat::read_status();
        let mut st = state().lock().unwrap();
        st.loading = false;
        match result {
            Ok(status) => st.status = Some(status),
            Err(error) => {
                append_line(&log, format!("❌ Could not read Windows apps: {error}"));
                st.error = Some(error);
            }
        }
    });
}

fn run_background(log: Arc<Mutex<String>>, title: String, job: impl FnOnce() -> Result<String, String> + Send + 'static) {
    {
        let mut st = state().lock().unwrap();
        if st.busy {
            return;
        }
        st.busy = true;
    }
    append_line(&log, &title);
    let log_clone = log.clone();
    thread::spawn(move || {
        match job() {
            Ok(output) => append_line(&log_clone, format!("✅ {output}")),
            Err(error) => append_line(&log_clone, format!("❌ {error}")),
        }
        {
            let mut st = state().lock().unwrap();
            st.busy = false;
            st.selected.clear();
        }
        refresh(log_clone);
    });
}

fn state_label(state: AppState, blocked: bool) -> (&'static str, egui::Color32) {
    match state {
        AppState::Installed if blocked => ("installed · blocked", ORANGE),
        AppState::Installed => ("installed", ORANGE),
        AppState::ProvisionedOnly => ("provisioned only", ORANGE),
        AppState::Blocked => ("blocked", NEON),
        AppState::Absent => ("not present", egui::Color32::GRAY),
    }
}

fn is_present(state: AppState) -> bool {
    matches!(state, AppState::Installed | AppState::ProvisionedOnly)
}

// -------------------- UI --------------------

pub fn show_winapp_removal(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    _show_popup: &mut bool,
    _popup_message: &mut String,
) {
    if !state().lock().unwrap().loaded_once {
        refresh(log.clone());
    }

    ui.heading("🗑 WinApp Removal — permanent debloat");
    ui.add_space(6.0);

    if !utils::is_elevated() {
        ui.horizontal(|ui| {
            ui.colored_label(YELLOW, "⚠ Not running as Administrator. Debloat requires elevation.");
            if ui.button("Relaunch as Administrator").clicked() {
                match utils::relaunch_as_admin() {
                    Ok(_) => std::process::exit(0),
                    Err(e) => append_line(log, format!("❌ Failed to relaunch as admin: {}", e)),
                }
            }
        });
        ui.add_space(6.0);
    }

    ui.label(
        "Uninstalling an app is not enough: Windows feature updates reinstall every app that is still provisioned. \
         Each selected app is first added to the AppxAllUserStore\\Deprovisioned list, then its provisioned package is removed \
         and it is uninstalled for all users — so Windows updates do not bring it back.",
    );
    ui.colored_label(egui::Color32::GRAY, "Create a restore point first. Apps can be allowed back from the Deprovisioned list below.");
    ui.add_space(8.0);

    let mut st = state().lock().unwrap();
    let working = st.loading || st.busy;

    // Toolbar
    let mut do_refresh = false;
    let mut select_recommended = false;
    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(!working, egui::Button::new("🔄 Refresh")).clicked() {
            do_refresh = true;
        }
        let recommended_count = st
            .status
            .as_ref()
            .map(|s| APPS.iter().zip(&s.apps).filter(|(a, s)| a.recommended && is_present(s.state)).count())
            .unwrap_or(0);
        if ui
            .add_enabled(!working && recommended_count > 0, egui::Button::new(format!("☑ Select rarely used ({recommended_count})")))
            .on_hover_text("Selects the bundled apps most people never use (Copilot, Bing, Clipchamp, Teams personal, new Outlook, Solitaire, Tips, promos...)")
            .clicked()
        {
            select_recommended = true;
        }
        if ui.add_enabled(!st.selected.is_empty() && !working, egui::Button::new("✖ Clear")).clicked() {
            st.selected.clear();
            st.confirm = false;
        }
        let label = egui::RichText::new(format!("🗑 Deprovision & remove SELECTED ({})", st.selected.len())).color(egui::Color32::from_rgb(255, 90, 90));
        if ui.add_enabled(!st.selected.is_empty() && !working, egui::Button::new(label)).clicked() {
            st.confirm = true;
        }
        ui.checkbox(&mut st.show_absent, "Show apps that are not present");
    });

    if select_recommended {
        if let Some(status) = &st.status {
            let picks: HashSet<usize> = APPS
                .iter()
                .zip(&status.apps)
                .enumerate()
                .filter(|(_, (a, s))| a.recommended && is_present(s.state))
                .map(|(i, _)| i)
                .collect();
            st.selected = picks;
        }
    }

    if st.loading {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Reading installed and provisioned packages...");
        });
    }
    if st.busy {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Working — see the log below...");
        });
    }
    if let Some(error) = &st.error {
        ui.colored_label(egui::Color32::from_rgb(255, 90, 90), format!("Could not read Windows apps: {error}"));
    }

    // Confirmation
    let mut start_removal: Option<Vec<(String, String)>> = None;
    if st.confirm {
        let names: Vec<&str> = st.selected.iter().map(|&i| APPS[i].label).collect();
        egui::Window::new("Confirm debloat")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |uiw| {
                uiw.label(format!("Deprovision and remove {} app(s):", names.len()));
                uiw.label(egui::RichText::new(names.join(", ")).color(YELLOW));
                uiw.add_space(6.0);
                uiw.colored_label(ORANGE, "They will be removed for every user and blocked from reinstalling during Windows updates.");
                uiw.add_space(6.0);
                uiw.horizontal(|uiw| {
                    if uiw.button("Confirm — Deprovision & remove").clicked() {
                        let status = st.status.clone().unwrap_or_default();
                        let mut indices: Vec<usize> = st.selected.iter().copied().collect();
                        indices.sort_unstable();
                        start_removal = Some(
                            indices
                                .into_iter()
                                .map(|i| {
                                    let family = status.apps.get(i).map(|s| s.family.clone()).unwrap_or_else(|| debloat::default_family(&APPS[i]));
                                    (APPS[i].package.to_string(), family)
                                })
                                .collect(),
                        );
                    }
                    if uiw.button("Cancel").clicked() {
                        st.confirm = false;
                    }
                });
            });
    }
    if start_removal.is_some() {
        st.confirm = false;
    }

    // App list
    let status = st.status.clone();
    let show_absent = st.show_absent;
    let mut allow_reinstall: Option<String> = None;
    ui.add_space(8.0);

    egui::ScrollArea::vertical().id_salt("winapp_scroll_v5").auto_shrink([false; 2]).show(ui, |ui| {
        if let Some(status) = &status {
            let present = status.apps.iter().filter(|s| is_present(s.state)).count();
            let blocked = status.apps.iter().filter(|s| s.blocked).count();
            ui.label(format!("{present} present · {blocked} blocked · {} selected", st.selected.len()));
            ui.add_space(4.0);

            ui.columns(2, |cols| {
                for (col, groups) in [(0usize, GROUPS_LEFT), (1usize, GROUPS_RIGHT)] {
                    for &group in groups {
                        let rows: Vec<usize> = APPS
                            .iter()
                            .enumerate()
                            .filter(|(i, a)| a.group == group && (show_absent || is_present(status.apps[*i].state)))
                            .map(|(i, _)| i)
                            .collect();
                        if rows.is_empty() {
                            continue;
                        }
                        cols[col].group(|ui| {
                            ui.label(egui::RichText::new(group).size(17.0).color(YELLOW).strong());
                            for i in rows {
                                let app = &APPS[i];
                                let app_status = &status.apps[i];
                                let (text, color) = state_label(app_status.state, app_status.blocked);
                                ui.horizontal(|ui| {
                                    let mut checked = st.selected.contains(&i);
                                    let label = if app.recommended { format!("{} ★", app.label) } else { app.label.to_string() };
                                    if ui
                                        .add_enabled(!working, egui::Checkbox::new(&mut checked, label))
                                        .on_hover_text(&app_status.family)
                                        .changed()
                                    {
                                        if checked {
                                            st.selected.insert(i);
                                        } else {
                                            st.selected.remove(&i);
                                        }
                                    }
                                    ui.colored_label(color, text);
                                });
                            }
                        });
                        cols[col].add_space(6.0);
                    }
                }
            });
            ui.colored_label(egui::Color32::GRAY, "★ = part of the \"rarely used\" preset.");

            ui.add_space(12.0);
            ui.separator();
            ui.label(egui::RichText::new(format!("Deprovisioned list ({} entries)", status.deprovisioned.len())).size(17.0).color(YELLOW).strong());
            ui.colored_label(egui::Color32::GRAY, "Windows skips these packages when it re-provisions apps during feature updates.");
            if status.deprovisioned.is_empty() {
                ui.label("The Deprovisioned list is empty.");
            }
            for family in &status.deprovisioned {
                ui.horizontal(|ui| {
                    let label = APPS
                        .iter()
                        .zip(&status.apps)
                        .find(|(_, s)| s.family.eq_ignore_ascii_case(family))
                        .map(|(a, _)| a.label.to_string())
                        .unwrap_or_else(|| family.split('_').next().unwrap_or(family).to_string());
                    if ui.add_enabled(!working, egui::Button::new("↩ Allow reinstall")).clicked() {
                        allow_reinstall = Some(family.clone());
                    }
                    ui.label(egui::RichText::new(label).strong());
                    ui.colored_label(egui::Color32::GRAY, family);
                });
            }
        }
    });
    drop(st);

    if do_refresh {
        refresh(log.clone());
    }
    if let Some(targets) = start_removal {
        let names = targets.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", ");
        run_background(log.clone(), format!("📦 Deprovisioning and removing: {names}"), move || debloat::deprovision_and_remove(&targets));
    }
    if let Some(family) = allow_reinstall {
        run_background(log.clone(), format!("↩ Removing from Deprovisioned list: {family}"), move || debloat::allow_reinstall(&family));
    }

    ui.ctx().request_repaint_after(Duration::from_millis(250));
}
