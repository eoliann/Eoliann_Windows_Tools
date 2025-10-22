// tools.rs (adaptat să folosească GLOBAL_OP_RUNNING / try_start_global_op)
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::Ordering;

use crate::commands;

/// Helper: așteaptă în log până apare un mesaj final (SUCCESS: sau ERROR:) sau expiră.
/// Returnează true dacă a găsit un final, false dacă a expirat.
fn wait_for_live_completion(log: &Arc<Mutex<String>>, timeout_secs: u64) -> bool {
    let start = Instant::now();
    loop {
        {
            let lg = log.lock().unwrap();
            let s = lg.to_uppercase();
            if s.contains("SUCCESS:") || s.contains("ERROR:") {
                return true;
            }
        }
        if start.elapsed() > Duration::from_secs(timeout_secs) {
            return false;
        }
        thread::sleep(Duration::from_millis(400));
    }
}

pub fn show_tools(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    _show_popup: &mut bool,
    _popup_message: &mut String,
) {
    ui.heading("🛠 Windows Tools");
    ui.add_space(6.0);

    // consultăm flag-ul global pentru a dezactiva butoanele dacă e cazul
    let global_busy = crate::commands::GLOBAL_OP_RUNNING.load(Ordering::SeqCst);

    // ---- Context menu ----
    ui.group(|ui| {
        ui.label("Context menu");

        let resp = ui.add_enabled(!global_busy, egui::Button::new("🖱 Toggle context menu (Win11 / Classic)"));
        resp.clone().on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(57, 255, 20),
                    "Switches between Windows 11 and Classic context menu"
                );
                ui.label("• Windows 11: modern, simplified context menu");
                ui.label("• Classic: full right-click menu from Windows 10");
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Requires logoff/restart of Explorer to apply"
                );
            });
        });
        if resp.clicked() { // `resp` is not moved here, it's a copy of the Response struct
            if let Some(guard) = commands::try_start_global_op("Toggle context menu", log) {
                let log_clone = log.clone();
                thread::spawn(move || {
                    let _guard = guard;
                    let out = commands::toggle_context_menu();
                    let mut lg = log_clone.lock().unwrap();
                    if lg.is_empty() { *lg = out; } else { *lg = format!("{}\n{}", lg, out); }
                });
            }
        }
    });

    ui.add_space(6.0);

    // ---- Maintenance ----
    ui.group(|ui| {
        ui.label("Maintenance");
        ui.horizontal_wrapped(|ui| {
            // Disk Cleanup
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🗑 Disk Cleanup"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Runs Windows Disk Cleanup");
                    ui.label("• Cleans system junk files");
                    ui.label("• Can free up disk space");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May take several minutes");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disk Cleanup", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::disk_cleanup();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Empty Recycle Bin
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🗑 Empty Recycle Bin"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Permanently deletes all files in the Recycle Bin");
                    ui.label("• Frees up disk space immediately");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Files cannot be recovered after this");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Empty Recycle Bin", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::empty_recycle_bin();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Clean Temporary Files
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🗑 Clean Temporary Files"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Deletes temporary system and app files");
                    ui.label("• Cleans %TEMP% folder");
                    ui.label("• Cleans Windows\\Temp");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Helps speed up Windows and free space");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Clean Temporary Files", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::clean_temporary_files();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Network Reset
            let resp = ui.add_enabled(!global_busy, egui::Button::new("📶 Network Reset"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Resets Windows network configuration");
                    ui.label("• Flushes DNS");
                    ui.label("• Resets Winsock & TCP/IP stack");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Will temporarily disconnect network");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Network Reset", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::network_reset();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Verify System Integrity (live)
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🛠 Verify System Integrity (SFC + DISM)"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Runs Windows integrity check");
                    ui.label("• Runs SFC (System File Checker)");
                    ui.label("• Runs DISM (Repair Windows Image)");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May take 10–30 minutes, do not close app");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("System Integrity Check (SFC+DISM)", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        // păstrăm guard și apelăm funcția live care streamează în log
                        let _guard = guard;
                        commands::verify_system_integrity_live(log_clone.clone());
                        // așteptăm finalul (cautăm SUCCESS: / ERROR:)
                        let completed = wait_for_live_completion(&log_clone, 3600); // timeout 1h
                        if !completed {
                            let mut lg = log_clone.lock().unwrap();
                            *lg = format!("{}\nERROR: System integrity check did not finish within timeout.", lg);
                        }
                        // când thread-ul se termină, guard e droppuit și GLOBAL_OP_RUNNING = false
                    });
                }
            }
        });
    });

    ui.add_space(6.0);

    // ---- Essential Tweaks ----
    ui.group(|ui| {
        ui.label("Essential Tweaks");
        ui.horizontal_wrapped(|ui| {
            // Disable Telemetry
            let resp = ui.add_enabled(!global_busy, egui::Button::new("📡 Disable Telemetry"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Disables Microsoft telemetry services");
                    ui.label("• Disables scheduled tasks");
                    ui.label("• Disables related registry keys");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May break Edge personalization features");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Telemetry", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::disable_telemetry();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Disable Location Tracking
            let resp = ui.add_enabled(!global_busy, egui::Button::new("📍 Disable Location Tracking"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Disables system-wide location tracking");
                    ui.label("• Modifies registry to deny location usage");
                    ui.label("• Disables location service");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Apps won't be able to access location");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Location Tracking", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::disable_location_tracking();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Disable Wifi-Sense
            let resp = ui.add_enabled(!global_busy, egui::Button::new("📶 Disable Wifi-Sense"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Disables Wifi-Sense (network data sharing)");
                    ui.label("• Blocks hotspot reporting");
                    ui.label("• Prevents auto-connect to WifiSense hotspots");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Improves privacy, no effect on normal Wi-Fi");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Wifi-Sense", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::disable_wifi_sense();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Enable End Task With Right Click
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🖱 Enable End Task With Right Click"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Adds 'End Task' option in taskbar context menu");
                    ui.label("• Right-click taskbar apps → End Task");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Makes closing apps faster");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Enable End Task Right Click", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::enable_end_task_right_click();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Undo End Task Right Click
            let resp = ui.add_enabled(!global_busy, egui::Button::new("↩ Undo End Task Right Click"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "Removes 'End Task' from taskbar context menu");
                    ui.label("• Restores default taskbar behavior");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Undo End Task Right Click", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::disable_end_task_right_click();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Disable Recall
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🚫 Disable Recall"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::RED, "Disables Windows Recall feature");
                    ui.label("• Turns off AI data analysis");
                    ui.label("• Removes Recall system feature via DISM");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Requires system restart");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Recall", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::disable_recall();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Enable Recall
            let resp = ui.add_enabled(!global_busy, egui::Button::new("✅ Enable Recall"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Re-enables Windows Recall feature");
                    ui.label("• Restores AI data analysis services");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Requires system restart");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Enable Recall", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::enable_recall();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Debloat Edge
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🗑 Debloat Edge"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Removes Edge bloatware and telemetry");
                    ui.label("• Disables recommendations & ads");
                    ui.label("• Hides first run experience");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May disable Edge personalization features");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Debloat Edge", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::debloat_edge();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Create Restore Point (live)
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🔁 Create Restore Point"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Creates a restore point at runtime in case a revert is needed from Winutil modifications");
                    ui.label("• Creates a System Restore Point");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Requires administrative privileges");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Uses PowerShell with ExecutionPolicy Bypass; may prompt UAC");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Create Restore Point", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        // apelăm varianta live care streamează în log
                        crate::commands::create_restore_point_live(log_clone.clone());
                        // așteptăm finalul
                        let completed = wait_for_live_completion(&log_clone, 1800); // timeout 30m
                        if !completed {
                            let mut lg = log_clone.lock().unwrap();
                            *lg = format!("{}\nERROR: Restore point creation did not finish within timeout.", lg);
                        }
                    });
                }
            }
        });
    });

    ui.add_space(6.0);

    // ---- Advanced Tweaks ----
    ui.group(|ui| {
        ui.label("Advanced Tweaks");
        ui.horizontal_wrapped(|ui| {
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🚫 Adobe Network Block"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 80, 80), "Blocks Adobe activation & telemetry servers");
                    ui.label("• Edits the HOSTS file with blocklist");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Requires admin rights");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ DNS cache will be flushed");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Adobe Network Block", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::adobe_network_block();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            let resp = ui.add_enabled(!global_busy, egui::Button::new("📉 Debloat Adobe"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Disables Adobe background services & updates");
                    ui.label("• Stops Adobe Desktop Service");
                    ui.label("• Disables Acrobat auto updates");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May break Adobe CC auto updates");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Debloat Adobe", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::adobe_debloat();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            let resp = ui.add_enabled(!global_busy, egui::Button::new("🚫 Disable Microsoft Copilot"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "Removes Microsoft Copilot integration");
                    ui.label("• Disables registry & Copilot button");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Requires Windows 23H2+");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Restart required to apply");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Microsoft Copilot", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::disable_copilot();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            let resp = ui.add_enabled(!global_busy, egui::Button::new("🖥 Set Display for Performance"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "✔ Optimizes system for best performance");
                    ui.label("• Disables animations and visual effects");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May make UI less smooth but faster");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Requires logoff/restart to fully apply");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Set Display for Performance", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = crate::commands::set_display_for_performance();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }
        });

        ui.add_space(10.0);

        // ---- DNS Selector ----
        ui.group(|ui| {
            ui.label("Set DNS");

            let dns_options = vec![
                "Google",
                "Cloudflare",
                "Cloudflare_Malware",
                "Cloudflare_Malware_Adult",
                "Open_DNS",
                "Quad9",
                "AdGuard_Ads_Trackers",
                "AdGuard_Ads_Trackers_Malware_Adult",
                "dns0.eu_Open",
                "dns0.eu_ZERO",
                "dns0.eu_KIDS",
            ];

            // dropdown state
            static mut SELECTED_DNS: &str = "Google";
            let mut selected = unsafe { SELECTED_DNS };

            egui::ComboBox::from_label("Choose DNS provider")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for option in &dns_options {
                        if ui.selectable_label(selected == *option, *option).clicked() {
                            selected = option;
                        }
                    }
                });

            unsafe { SELECTED_DNS = selected; }

            ui.add_space(6.0);

            let resp = ui.add_enabled(!global_busy, egui::Button::new("▶ Run"));
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op(&format!("Set DNS to {}", selected), log) {
                    let provider = selected.to_string();
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::set_dns(&provider);
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }
        });
    });

    ui.add_space(6.0);

    // ---- Power Plans ----
    ui.group(|ui| {
        ui.label("Power Plans");
        ui.horizontal_wrapped(|ui| {
            // High Performance
            let resp = ui.add_enabled(!global_busy, egui::Button::new("⚡ High Performance"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "High Performance Plan");
                    ui.label("• Maximizes performance at the cost of higher power usage.");
                    ui.label("• Keeps CPU and GPU at higher frequencies.");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Recommended for desktops or when on AC power.");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Power Plan: High Performance", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::power_plan_switcher("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c");
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Balanced
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🔌 Balanced"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Balanced Plan");
                    ui.label("• Default Windows plan (best for most users).");
                    ui.label("• Dynamically balances performance and energy usage.");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Recommended for laptops and general use.");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Power Plan: Balanced", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::power_plan_switcher("381b4222-f694-41f0-9685-ff5bb260df2e");
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            // Power Saver
            let resp = ui.add_enabled(!global_busy, egui::Button::new("🔋 Power Saver"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Power Saver Plan");
                    ui.label("• Reduces system performance to save battery life.");
                    ui.label("• Lowers CPU frequencies and dims display faster.");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Recommended only when running low on battery.");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Power Plan: Power Saver", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::power_plan_switcher("a1841308-3541-4fab-bc81-f71556f20b4a");
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }
        });
    });

    ui.add_space(6.0);

    // ---- Power Tweaks ----
    ui.group(|ui| {
        ui.label("Power Tweaks");
        ui.horizontal_wrapped(|ui| {
            let resp = ui.add_enabled(!global_busy, egui::Button::new("💤 Disable Sleep"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Disable Sleep Mode");
                    ui.label("• Prevents Windows from going into sleep mode.");
                    ui.label("• Useful for servers, media PCs, or long-running tasks.");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ May increase power consumption.");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Sleep", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::disable_sleep();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            let resp = ui.add_enabled(!global_busy, egui::Button::new("💽 Disable HDD/SSD Timeout"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Disable Disk Timeout");
                    ui.label("• Prevents hard drives and SSDs from powering down after inactivity.");
                    ui.label("• Can improve responsiveness on systems with multiple drives.");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Continuous spinning may reduce HDD lifespan slightly.");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable HDD/SSD Timeout", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::disable_hdd();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }

            let resp = ui.add_enabled(!global_busy, egui::Button::new("🖥️ Disable Monitor Timeout"));
            resp.clone().on_hover_ui(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Disable Display Timeout");
                    ui.label("• Prevents the monitor from turning off automatically.");
                    ui.label("• Useful for presentations, kiosks, or media setups.");
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Monitor stays on constantly → higher energy use.");
                });
            });
            if resp.clicked() {
                if let Some(guard) = commands::try_start_global_op("Disable Monitor Timeout", log) {
                    let log_clone = log.clone();
                    thread::spawn(move || {
                        let _guard = guard;
                        let result = commands::disable_monitor();
                        let mut lg = log_clone.lock().unwrap();
                        if lg.is_empty() { *lg = result; } else { *lg = format!("{}\n{}", lg, result); }
                    });
                }
            }
        });
    });
}
