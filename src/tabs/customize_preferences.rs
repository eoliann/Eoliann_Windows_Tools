// src/tabs/customize_preferences.rs
//
// Customize preferences UI + Windows tweaks.
// - Removed all "Start with Windows" / Startup shortcut functionality.
// - Rest of behavior unchanged: registry helpers, persistence, Windows Update auto-check, tooltips flag, features.

use std::sync::{Arc, Mutex};
use eframe::egui::{self, RichText};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs;
use std::{fs, io, env, time::{SystemTime, UNIX_EPOCH}};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Output;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn run_hidden(cmd: &str, args: &[&str]) -> Result<Output, String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(cmd)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("failed to spawn {}: {}", cmd, e))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(format!("{} is not supported on this platform", cmd))
    }
}

fn reg_query_value(path: &str, value_name: &str) -> Option<String> {
    let args = ["query", path, "/v", value_name];
    let out = run_hidden("reg", &args).ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    for line in s.lines() {
        if line.trim_start().starts_with(value_name) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(v) = parts.last() { return Some(v.to_string()); }
        }
    }
    None
}

//////////////////////////////////////////////////////////////////
// Preferences persistence
//////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefsFile {
    pub enable_tooltips: bool,
    pub auto_check_updates: bool,
    pub mouse_accel_enabled: bool,
    pub numlock_enabled: bool,
    pub taskbar_search_enabled: bool,
    pub taskbar_widgets_enabled: bool,
    pub snap_enabled: bool,
    pub sticky_enabled: bool,
    pub taskview_enabled: bool,
    pub verbose_logon_enabled: bool,
    pub bitlocker_protection_on: bool,
}

impl Default for PrefsFile {
    fn default() -> Self {
        Self {
            enable_tooltips: true,
            auto_check_updates: true,
            mouse_accel_enabled: false,
            numlock_enabled: false,
            taskbar_search_enabled: false,
            taskbar_widgets_enabled: false,
            snap_enabled: false,
            sticky_enabled: false,
            taskview_enabled: false,
            verbose_logon_enabled: false,
            bitlocker_protection_on: false,
        }
    }
}

fn prefs_path() -> PathBuf {
    if let Some(mut p) = dirs::config_dir() {
        p.push("Eoliann_Windows_Tools");
        let _ = fs::create_dir_all(&p);
        p.push("preferences.json");
        p
    } else {
        let mut p = PathBuf::from(".");
        p.push("preferences.json");
        p
    }
}

fn load_prefs() -> PrefsFile {
    let path = prefs_path();
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str::<PrefsFile>(&s).unwrap_or_default(),
        Err(_) => PrefsFile::default(),
    }
}

fn save_prefs(p: &PrefsFile) -> io::Result<()> {
    let path = prefs_path();
    if let Some(dir) = path.parent() { fs::create_dir_all(dir)?; }
    let s = serde_json::to_string_pretty(p).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, s)
}

//////////////////////////////////////////////////////////////////
// Registry helpers (unchanged behavior)
//////////////////////////////////////////////////////////////////

fn read_mouse_accel_from_registry() -> Option<bool> {
    reg_query_value(r#"HKCU\Control Panel\Mouse"#, "MouseSpeed")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n > 0))
}

fn set_mouse_accel_in_registry(enabled: bool) -> Result<String, String> {
    let (ms, t1, t2) = if enabled { ("1", "6", "10") } else { ("0", "0", "0") };
    let mut combined = String::new();
    let props = [("MouseSpeed", ms), ("MouseThreshold1", t1), ("MouseThreshold2", t2)];
    for (name, val) in props.iter() {
        let args = ["add", r#"HKCU\Control Panel\Mouse"#, "/v", name, "/t", "REG_SZ", "/d", val, "/f"];
        let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
        if !out.status.success() {
            combined.push_str(&format!("{}: err: {}\n", name, String::from_utf8_lossy(&out.stderr)));
        } else {
            combined.push_str(&format!("{}: ok\n", name));
        }
    }
    Ok(combined)
}

fn read_numlock_from_registry() -> Option<bool> {
    if let Some(v) = reg_query_value(r#"HKCU\Control Panel\Keyboard"#, "InitialKeyboardIndicators") {
        if let Ok(n) = v.parse::<i32>() { return Some(n != 0); }
        return Some(v != "0".to_string());
    }
    if let Some(v) = reg_query_value(r#"HKU\.DEFAULT\Control Panel\Keyboard"#, "InitialKeyboardIndicators") {
        if let Ok(n) = v.parse::<i32>() { return Some(n != 0); }
        return Some(v != "0".to_string());
    }
    None
}

fn set_numlock_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "2" } else { "0" };
    let mut combined = String::new();
    let targets = [
        (r#"HKCU\Control Panel\Keyboard"#, "InitialKeyboardIndicators"),
        (r#"HKU\.DEFAULT\Control Panel\Keyboard"#, "InitialKeyboardIndicators"),
    ];
    for (path, name) in targets.iter() {
        let args = ["add", path, "/v", name, "/t", "REG_SZ", "/d", value, "/f"];
        let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
        if !out.status.success() {
            combined.push_str(&format!("{}: err: {}\n", path, String::from_utf8_lossy(&out.stderr)));
        } else {
            combined.push_str(&format!("{}: ok\n", path));
        }
    }
    Ok(combined)
}

fn read_taskbar_search_from_registry() -> Option<bool> {
    reg_query_value(r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Search\"#, "SearchboxTaskbarMode")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n != 0))
}

fn set_taskbar_search_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "1" } else { "0" };
    let args = [
        "add",
        r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Search\"#,
        "/v",
        "SearchboxTaskbarMode",
        "/t",
        "REG_DWORD",
        "/d",
        value,
        "/f",
    ];
    let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
    if !out.status.success() { Err(format!("{}", String::from_utf8_lossy(&out.stderr))) }
    else { Ok(String::from_utf8_lossy(&out.stdout).to_string()) }
}

fn read_taskbar_widgets_from_registry() -> Option<bool> {
    reg_query_value(r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"#, "TaskbarDa")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n != 0))
}

fn set_taskbar_widgets_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "1" } else { "0" };
    let args = ["add", r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"#, "/v", "TaskbarDa", "/t", "REG_DWORD", "/d", value, "/f"];
    let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
    if out.status.success() { return Ok(String::from_utf8_lossy(&out.stdout).to_string()); }

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !stderr.to_lowercase().contains("access is denied") { return Err(stderr); }

    let dw: u32 = if enabled { 1 } else { 0 };
    let hex = format!("{:08x}", dw);
    let reg_text = format!(
        "Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced]\r\n\"TaskbarDa\"=dword:{}\r\n",
        hex
    );

    let mut tmp = env::temp_dir();
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    tmp.push(format!("ewt_taskbar_widgets_{}.reg", ts));

    if let Err(e) = fs::write(&tmp, reg_text) { return Err(format!("Failed to write temporary .reg file: {}", e)); }

    let ps_cmd = format!("Start-Process regedit.exe -ArgumentList '/s','{}' -Verb runas -Wait", tmp.display());
    let ps_args = ["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &ps_cmd];
    let ps_out = run_hidden("powershell", &ps_args).map_err(|e| format!("failed to spawn PowerShell for regedit fallback: {}", e))?;
    if !ps_out.status.success() {
        let ps_stderr = String::from_utf8_lossy(&ps_out.stderr).to_string();
        let _ = fs::remove_file(&tmp);
        return Err(format!("Failed to import .reg via elevated regedit: {}\nps_err: {}", stderr, ps_stderr));
    }

    let _ = run_hidden("rundll32.exe", &["user32.dll,UpdatePerUserSystemParameters"]);
    let _ = fs::remove_file(&tmp);
    Ok(format!("Imported .reg via regedit (fallback)."))
}

fn read_snap_from_registry() -> Option<bool> {
    reg_query_value(r#"HKCU\Control Panel\Desktop"#, "WindowArrangementActive")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n != 0))
}

fn set_snap_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "1" } else { "0" };
    let args = ["add", r#"HKCU\Control Panel\Desktop"#, "/v", "WindowArrangementActive", "/t", "REG_SZ", "/d", value, "/f"];
    let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
    if !out.status.success() { Err(format!("{}", String::from_utf8_lossy(&out.stderr))) }
    else { Ok(String::from_utf8_lossy(&out.stdout).to_string()) }
}

fn read_sticky_from_registry() -> Option<bool> {
    reg_query_value(r#"HKCU\Control Panel\Accessibility\StickyKeys"#, "Flags")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n == 510))
}

fn set_sticky_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "510" } else { "58" };
    let args = ["add", r#"HKCU\Control Panel\Accessibility\StickyKeys"#, "/v", "Flags", "/t", "REG_SZ", "/d", value, "/f"];
    let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
    if !out.status.success() { Err(format!("{}", String::from_utf8_lossy(&out.stderr))) }
    else { Ok(String::from_utf8_lossy(&out.stdout).to_string()) }
}

fn read_taskview_from_registry() -> Option<bool> {
    reg_query_value(r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"#, "ShowTaskViewButton")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n != 0))
}

fn set_taskview_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "1" } else { "0" };
    let args = ["add", r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"#, "/v", "ShowTaskViewButton", "/t", "REG_DWORD", "/d", value, "/f"];
    let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
    if !out.status.success() { Err(format!("{}", String::from_utf8_lossy(&out.stderr))) }
    else { Ok(String::from_utf8_lossy(&out.stdout).to_string()) }
}

fn read_verbose_logon_from_registry() -> Option<bool> {
    reg_query_value(r#"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System"#, "VerboseStatus")
        .and_then(|v| v.parse::<i32>().ok().map(|n| n != 0))
}

fn set_verbose_logon_in_registry(enabled: bool) -> Result<String, String> {
    let value = if enabled { "1" } else { "0" };
    let args = ["add", r#"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System"#, "/v", "VerboseStatus", "/t", "REG_DWORD", "/d", value, "/f"];
    let out = run_hidden("reg", &args).map_err(|e| format!("failed to spawn reg: {}", e))?;
    if !out.status.success() { Err(format!("{}", String::from_utf8_lossy(&out.stderr))) }
    else { Ok(String::from_utf8_lossy(&out.stdout).to_string()) }
}

fn read_bitlocker_protection_status() -> Option<bool> {
    let out = run_hidden("manage-bde", &["-status", "C:"]).ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    for line in s.lines() {
        let l = line.trim();
        if l.starts_with("Protection Status:") {
            if l.contains("On") { return Some(true); }
            if l.contains("Off") { return Some(false); }
        }
    }
    None
}

fn set_bitlocker_protection(enable: bool) -> Result<String, String> {
    let cmd = if enable { vec!["-on", "C:"] } else { vec!["-off", "C:"] };
    let out = run_hidden("manage-bde", &cmd.iter().map(|s| *s).collect::<Vec<&str>>()).map_err(|e| format!("failed to spawn manage-bde: {}", e))?;
    if !out.status.success() { Err(format!("{}", String::from_utf8_lossy(&out.stderr))) }
    else { Ok(String::from_utf8_lossy(&out.stdout).to_string()) }
}

//////////////////////////////////////////////////////////////////
// Windows Update auto-check + tooltips flag
//////////////////////////////////////////////////////////////////

fn set_auto_check_windows_updates(enabled: bool) -> Result<String, String> {
    let task_name = "Eoliann_EWT_CheckWindowsUpdate";
    if enabled {
        let args = [
            "/Create", "/SC", "DAILY", "/TN", task_name,
            "/TR", "cmd /c UsoClient StartScan", "/RL", "HIGHEST", "/RU", "SYSTEM", "/F",
        ];
        let out = run_hidden("schtasks", &args).map_err(|e| format!("failed to spawn schtasks: {}", e))?;
        if !out.status.success() { return Err(format!("schtasks create failed: {}", String::from_utf8_lossy(&out.stderr))); }
        Ok(format!("Scheduled task created: {}", task_name))
    } else {
        let args = ["/Delete", "/TN", task_name, "/F"];
        let out = run_hidden("schtasks", &args).map_err(|e| format!("failed to spawn schtasks: {}", e))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if stderr.to_lowercase().contains("the system cannot find") { return Ok("Scheduled task not present (nothing to delete)".to_string()); }
            return Err(format!("schtasks delete failed: {}", stderr));
        }
        Ok(format!("Scheduled task removed: {}", task_name))
    }
}

fn apply_enable_tooltips_flag(enabled: bool) -> Result<String, String> {
    if let Some(mut cfg) = dirs::config_dir() {
        cfg.push("Eoliann_Windows_Tools");
        fs::create_dir_all(&cfg).map_err(|e| format!("Failed to create config dir: {}", e))?;
        cfg.push("tooltips_enabled");
        if enabled {
            fs::write(&cfg, "1").map_err(|e| format!("Failed to write tooltip flag: {}", e))?;
            Ok(format!("Tooltip flag written: {}", cfg.display()))
        } else {
            if cfg.exists() { fs::remove_file(&cfg).map_err(|e| format!("Failed to remove tooltip flag: {}", e))?; }
            Ok(format!("Tooltip flag removed: {}", cfg.display()))
        }
    } else {
        Err("Could not resolve config directory".to_string())
    }
}

//////////////////////////////////////////////////////////////////
// UI: Customize Preferences (General + Features)
//////////////////////////////////////////////////////////////////
pub fn show_customize_preferences(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    show_popup: &mut bool,
    popup_message: &mut String,
    enable_tooltips: &mut bool,
    auto_check_updates: &mut bool,
    general_prefs_loaded: &mut bool,
    mouse_accel_enabled: &mut bool,
    mouse_prefs_loaded: &mut bool,
    numlock_enabled: &mut bool,
    numlock_prefs_loaded: &mut bool,
    taskbar_search_enabled: &mut bool,
    taskbar_search_prefs_loaded: &mut bool,
    taskbar_widgets_enabled: &mut bool,
    taskbar_widgets_prefs_loaded: &mut bool,
    snap_enabled: &mut bool,
    snap_prefs_loaded: &mut bool,
    sticky_enabled: &mut bool,
    sticky_prefs_loaded: &mut bool,
    taskview_enabled: &mut bool,
    taskview_prefs_loaded: &mut bool,
    verbose_logon_enabled: &mut bool,
    verbose_logon_prefs_loaded: &mut bool,
    bitlocker_protection_on: &mut bool,
    bitlocker_prefs_loaded: &mut bool,
) {
    if !*general_prefs_loaded {
        let p = load_prefs();
        *enable_tooltips = p.enable_tooltips;
        *auto_check_updates = p.auto_check_updates;

        *mouse_accel_enabled = p.mouse_accel_enabled;
        *numlock_enabled = p.numlock_enabled;
        *taskbar_search_enabled = p.taskbar_search_enabled;
        *taskbar_widgets_enabled = p.taskbar_widgets_enabled;
        *snap_enabled = p.snap_enabled;
        *sticky_enabled = p.sticky_enabled;
        *taskview_enabled = p.taskview_enabled;
        *verbose_logon_enabled = p.verbose_logon_enabled;
        *bitlocker_protection_on = p.bitlocker_protection_on;

        if let Ok(mut lg) = log.lock() { lg.push_str("Loaded preferences from disk.\n"); }
        *general_prefs_loaded = true;
    }

    egui::CollapsingHeader::new("General Preferences")
        .default_open(true)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("General Preferences").heading());
                ui.add_space(6.0);

                ui.checkbox(enable_tooltips, "Enable advanced tooltips");
                ui.checkbox(auto_check_updates, "Auto-check for updates (Windows Update)");

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save General Preferences").clicked() {
                        let prefs = PrefsFile {
                            enable_tooltips: *enable_tooltips,
                            auto_check_updates: *auto_check_updates,
                            mouse_accel_enabled: *mouse_accel_enabled,
                            numlock_enabled: *numlock_enabled,
                            taskbar_search_enabled: *taskbar_search_enabled,
                            taskbar_widgets_enabled: *taskbar_widgets_enabled,
                            snap_enabled: *snap_enabled,
                            sticky_enabled: *sticky_enabled,
                            taskview_enabled: *taskview_enabled,
                            verbose_logon_enabled: *verbose_logon_enabled,
                            bitlocker_protection_on: *bitlocker_protection_on,
                        };

                        if let Err(e) = save_prefs(&prefs) {
                            if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to save prefs: {}\n", e)); }
                            *show_popup = true;
                            *popup_message = format!("Failed to save preferences: {}", e);
                        } else {
                            if let Ok(mut lg) = log.lock() { lg.push_str("General preferences saved to disk.\n"); }

                            // apply tooltips flag (best-effort)
                            if let Err(err) = apply_enable_tooltips_flag(*enable_tooltips) {
                                if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to persist tooltips flag: {}\n", err)); }
                            }

                            // configure Windows Update auto-check (best-effort)
                            match set_auto_check_windows_updates(*auto_check_updates) {
                                Ok(msg) => { if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Auto-check for Windows Updates configured: {}\n", msg)); } }
                                Err(err) => {
                                    if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to configure Windows Update auto-check: {}\n", err)); }
                                    *show_popup = true;
                                    *popup_message = format!("Saved but failed to configure Windows Update auto-check: {}", err);
                                    return;
                                }
                            }
                        }
                    }

                    if ui.button("Reset General Defaults").clicked() {
                        *enable_tooltips = true;
                        *auto_check_updates = true;
                        if let Ok(mut lg) = log.lock() { lg.push_str("General preferences reset to defaults (not yet saved).\n"); }
                    }
                });
            });
        });

    ui.add_space(6.0);

    // Features section (unchanged behavior; persists on Apply)
    egui::CollapsingHeader::new("Features")
        .default_open(true)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Features / Tweaks").heading());
                ui.add_space(6.0);

                if !*mouse_prefs_loaded {
                    if let Some(val) = read_mouse_accel_from_registry() {
                        *mouse_accel_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded mouse accel from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read mouse accel (fallback persisted value used)\n"); }
                    *mouse_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Mouse Acceleration"); ui.checkbox(mouse_accel_enabled, ""); });
                ui.add_space(8.0);

                if !*numlock_prefs_loaded {
                    if let Some(val) = read_numlock_from_registry() {
                        *numlock_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded NumLock initial state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read NumLock state (fallback persisted value used)\n"); }
                    *numlock_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("NumLock on startup"); ui.checkbox(numlock_enabled, ""); });
                ui.add_space(8.0);

                if !*taskbar_search_prefs_loaded {
                    if let Some(val) = read_taskbar_search_from_registry() {
                        *taskbar_search_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded Taskbar Search state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read Taskbar Search state (fallback persisted value used)\n"); }
                    *taskbar_search_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Taskbar Search Button"); ui.checkbox(taskbar_search_enabled, ""); });
                ui.add_space(8.0);

                if !*taskbar_widgets_prefs_loaded {
                    if let Some(val) = read_taskbar_widgets_from_registry() {
                        *taskbar_widgets_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded Taskbar Widgets state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read Taskbar Widgets (fallback persisted value used)\n"); }
                    *taskbar_widgets_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Taskbar Widgets"); ui.checkbox(taskbar_widgets_enabled, ""); });
                ui.add_space(8.0);

                if !*snap_prefs_loaded {
                    if let Some(val) = read_snap_from_registry() {
                        *snap_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded Snap Windows state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read Snap Windows state (fallback persisted value used)\n"); }
                    *snap_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Snap Windows on startup"); ui.checkbox(snap_enabled, ""); });
                ui.add_space(8.0);

                if !*sticky_prefs_loaded {
                    if let Some(val) = read_sticky_from_registry() {
                        *sticky_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded Sticky Keys state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read Sticky Keys state (fallback persisted value used)\n"); }
                    *sticky_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Sticky Keys on startup"); ui.checkbox(sticky_enabled, ""); });
                ui.add_space(8.0);

                if !*taskview_prefs_loaded {
                    if let Some(val) = read_taskview_from_registry() {
                        *taskview_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded Task View state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read Task View state (fallback persisted value used)\n"); }
                    *taskview_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Task View button"); ui.checkbox(taskview_enabled, ""); });
                ui.add_space(8.0);

                if !*verbose_logon_prefs_loaded {
                    if let Some(val) = read_verbose_logon_from_registry() {
                        *verbose_logon_enabled = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded VerboseLogon state from registry: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read VerboseLogon state (fallback persisted value used)\n"); }
                    *verbose_logon_prefs_loaded = true;
                }
                ui.horizontal(|ui| { ui.label("Verbose Logon Messages (system)"); ui.checkbox(verbose_logon_enabled, ""); });
                ui.add_space(8.0);

                if !*bitlocker_prefs_loaded {
                    if let Some(val) = read_bitlocker_protection_status() {
                        *bitlocker_protection_on = val;
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Loaded BitLocker protection (C:) from system: {}\n", val)); }
                    } else if let Ok(mut lg) = log.lock() { lg.push_str("Could not read BitLocker status (manage-bde missing or error); fallback persisted value used\n"); }
                    *bitlocker_prefs_loaded = true;
                }
                ui.horizontal(|ui| {
                    ui.label(format!("BitLocker protection (C:): {}", if *bitlocker_protection_on { "ON" } else { "OFF" }));
                    ui.separator();
                    ui.checkbox(bitlocker_protection_on, "Protection ON");
                });

                ui.add_space(12.0);
                if ui.button("Save / Apply Features").clicked() {
                    if let Err(err) = set_mouse_accel_in_registry(*mouse_accel_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply mouse accel: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply mouse accel: {}", err); return;
                    }
                    if let Err(err) = set_numlock_in_registry(*numlock_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply NumLock: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply NumLock: {}", err); return;
                    }
                    if let Err(err) = set_taskbar_search_in_registry(*taskbar_search_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply Taskbar Search: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply Taskbar Search: {}", err); return;
                    }
                    if let Err(err) = set_taskbar_widgets_in_registry(*taskbar_widgets_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply Taskbar Widgets: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply Taskbar Widgets: {}", err); return;
                    }
                    if let Err(err) = set_snap_in_registry(*snap_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply Snap Windows: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply Snap Windows: {}", err); return;
                    }
                    if let Err(err) = set_sticky_in_registry(*sticky_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply Sticky Keys: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply Sticky Keys: {}", err); return;
                    }
                    if let Err(err) = set_taskview_in_registry(*taskview_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply Task View: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply Task View: {}", err); return;
                    }
                    if let Err(err) = set_verbose_logon_in_registry(*verbose_logon_enabled) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to apply Verbose Logon: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to apply Verbose Logon: {}", err); return;
                    }
                    if let Err(err) = set_bitlocker_protection(*bitlocker_protection_on) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to change BitLocker: {}\n", err)); }
                        *show_popup = true; *popup_message = format!("Failed to change BitLocker: {}", err); return;
                    }

                    let prefs = PrefsFile {
                        enable_tooltips: *enable_tooltips,
                        auto_check_updates: *auto_check_updates,
                        mouse_accel_enabled: *mouse_accel_enabled,
                        numlock_enabled: *numlock_enabled,
                        taskbar_search_enabled: *taskbar_search_enabled,
                        taskbar_widgets_enabled: *taskbar_widgets_enabled,
                        snap_enabled: *snap_enabled,
                        sticky_enabled: *sticky_enabled,
                        taskview_enabled: *taskview_enabled,
                        verbose_logon_enabled: *verbose_logon_enabled,
                        bitlocker_protection_on: *bitlocker_protection_on,
                    };
                    if let Err(e) = save_prefs(&prefs) {
                        if let Ok(mut lg) = log.lock() { lg.push_str(&format!("Failed to persist prefs after apply: {}\n", e)); }
                        *show_popup = true; *popup_message = format!("Applied but failed to persist prefs: {}", e);
                        return;
                    }

                    if let Ok(mut lg) = log.lock() { lg.push_str("All features applied successfully and persisted.\n"); }
                    *show_popup = true;
                    *popup_message = "Features applied.".to_string();
                }
            });
        });
}
