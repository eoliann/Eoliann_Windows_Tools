use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::commands;

pub fn show_tools(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    _show_popup: &mut bool,
    _popup_message: &mut String,
) {
    ui.heading("🛠 Windows Tools");
    ui.add_space(6.0);

    // ---- Context menu ----
    ui.group(|ui| {
        ui.label("Context menu");
        if ui.button("🖱 Toggle context menu (Win11 / Classic)")
            .on_hover_ui(|ui| {
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
            })
            .clicked()
        {
            let out = commands::toggle_context_menu();
            *log.lock().unwrap() = out;
        }
    });

    ui.add_space(6.0);

    // ---- Maintenance ----
    ui.group(|ui| {
        ui.label("Maintenance");
        ui.horizontal_wrapped(|ui| {
            if ui.button("🗑 Disk Cleanup")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Runs Windows Disk Cleanup");
                        ui.label("• Cleans system junk files");
                        ui.label("• Can free up disk space");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ May take several minutes");
                    });
                })
                .clicked()
            {
                let log_clone = log.clone();
                thread::spawn(move || {
                    let msg = commands::disk_cleanup();
                    *log_clone.lock().unwrap() = msg;
                });
            }

            if ui.button("🗑 Empty Recycle Bin")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Permanently deletes all files in the Recycle Bin");
                        ui.label("• Frees up disk space immediately");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ Files cannot be recovered after this");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::empty_recycle_bin();
            }

            if ui.button("🗑 Clean Temporary Files")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Deletes temporary system and app files");
                        ui.label("• Cleans %TEMP% folder");
                        ui.label("• Cleans Windows\\Temp");
                        ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Helps speed up Windows and free space");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::clean_temporary_files();
            }

            if ui.button("📶 Network Reset")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Resets Windows network configuration");
                        ui.label("• Flushes DNS");
                        ui.label("• Resets Winsock & TCP/IP stack");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ Will temporarily disconnect network");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::network_reset();
            }

            if ui.button("🛠 Verify System Integrity (SFC + DISM)")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Runs Windows integrity check");
                        ui.label("• Runs SFC (System File Checker)");
                        ui.label("• Runs DISM (Repair Windows Image)");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ May take 10–30 minutes, do not close app");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() =
                    "⏳ Starting system integrity check... (SFC + DISM)".to_string();
                commands::verify_system_integrity_live(log.clone());
            }
        });
    });

    ui.add_space(6.0);

    // ---- Essential Tweaks ----
    ui.group(|ui| {
        ui.label("Essential Tweaks");
        ui.horizontal_wrapped(|ui| {
            if ui.button("📡 Disable Telemetry")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Disables Microsoft telemetry services");
                        ui.label("• Disables scheduled tasks");
                        ui.label("• Disables related registry keys");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ May break Edge personalization features");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::disable_telemetry();
            }

            if ui.button("📍 Disable Location Tracking")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Disables system-wide location tracking");
                        ui.label("• Modifies registry to deny location usage");
                        ui.label("• Disables location service");
                        ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Apps won't be able to access location");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::disable_location_tracking();
            }

            if ui.button("📶 Disable Wifi-Sense")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Disables Wifi-Sense (network data sharing)");
                        ui.label("• Blocks hotspot reporting");
                        ui.label("• Prevents auto-connect to WifiSense hotspots");
                        ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Improves privacy, no effect on normal Wi-Fi");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::disable_wifi_sense();
            }

            if ui.button("🖱 Enable End Task With Right Click")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Adds 'End Task' option in taskbar context menu");
                        ui.label("• Right-click taskbar apps → End Task");
                        ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ Makes closing apps faster");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::enable_end_task_right_click();
            }

            if ui.button("↩ Undo End Task Right Click")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::YELLOW, "Removes 'End Task' from taskbar context menu");
                        ui.label("• Restores default taskbar behavior");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::disable_end_task_right_click();
            }

            if ui.button("🚫 Disable Recall")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::RED, "Disables Windows Recall feature");
                        ui.label("• Turns off AI data analysis");
                        ui.label("• Removes Recall system feature via DISM");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ Requires system restart");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::disable_recall();
            }

            if ui.button("✅ Enable Recall")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(57, 255, 20), "Re-enables Windows Recall feature");
                        ui.label("• Restores AI data analysis services");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ Requires system restart");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::enable_recall();
            }

            if ui.button("🗑 Debloat Edge")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Removes Edge bloatware and telemetry");
                        ui.label("• Disables recommendations & ads");
                        ui.label("• Hides first run experience");
                        ui.colored_label(egui::Color32::YELLOW, "⚠ May disable Edge personalization features");
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::debloat_edge();
            }
        });
    });

    ui.add_space(6.0);

    // ---- Advanced Tweaks ----
    // ui.group(|ui| {
    //     ui.label("Advanced Tweaks");
    //     ui.horizontal_wrapped(|ui| {
    //         if ui.button("🚫 Adobe Network Block")
    //             .on_hover_ui(|ui| {
    //                 ui.vertical(|ui| {
    //                     ui.colored_label(
    //                         egui::Color32::from_rgb(255, 80, 80),
    //                         "Blocks Adobe activation & telemetry servers"
    //                     );
    //                     ui.label("• Edits the HOSTS file with blocklist");
    //                     ui.colored_label(
    //                         egui::Color32::YELLOW,
    //                         "⚠ Requires admin rights"
    //                     );
    //                     ui.colored_label(
    //                         egui::Color32::LIGHT_BLUE,
    //                         "ℹ DNS cache will be flushed"
    //                     );
    //                 });
    //             })
    //             .clicked()
    //         {
    //             *log.lock().unwrap() = commands::adobe_network_block();
    //         }

    //         if ui.button("📉 Debloat Adobe")
    //             .on_hover_ui(|ui| {
    //                 ui.vertical(|ui| {
    //                     ui.colored_label(
    //                         egui::Color32::from_rgb(57, 255, 20),
    //                         "Disables Adobe background services & updates"
    //                     );
    //                     ui.label("• Stops Adobe Desktop Service");
    //                     ui.label("• Disables Acrobat auto updates");
    //                     ui.colored_label(
    //                         egui::Color32::YELLOW,
    //                         "⚠ May break Adobe CC auto updates"
    //                     );
    //                 });
    //             })
    //             .clicked()
    //         {
    //             *log.lock().unwrap() = commands::adobe_debloat();
    //         }

    //         if ui.button("🚫 Disable Microsoft Copilot")
    //             .on_hover_ui(|ui| {
    //                 ui.vertical(|ui| {
    //                     ui.colored_label(
    //                         egui::Color32::from_rgb(255, 100, 100),
    //                         "Removes Microsoft Copilot integration"
    //                     );
    //                     ui.label("• Disables registry & Copilot button");
    //                     ui.colored_label(
    //                         egui::Color32::YELLOW,
    //                         "⚠ Requires Windows 23H2+"
    //                     );
    //                     ui.colored_label(
    //                         egui::Color32::LIGHT_BLUE,
    //                         "ℹ Restart required to apply"
    //                     );
    //                 });
    //             })
    //             .clicked()
    //         {
    //             *log.lock().unwrap() = crate::commands::disable_copilot();
    //         }
    //         if ui.button("🖥 Set Display for Performance")
    //             .on_hover_ui(|ui| {
    //                 ui.vertical(|ui| {
    //                     ui.colored_label(
    //                         egui::Color32::from_rgb(57, 255, 20), // verde neon
    //                         "✔ Optimizes system for best performance"
    //                     );
    //                     ui.label("• Disables animations and visual effects");
    //                     ui.colored_label(
    //                         egui::Color32::YELLOW,
    //                         "⚠ May make UI less smooth but faster"
    //                     );
    //                     ui.colored_label(
    //                         egui::Color32::LIGHT_BLUE,
    //                         "ℹ Requires logoff/restart to fully apply"
    //                     );
    //                 });
    //             })
    //             .clicked()
    //         {
    //             *log.lock().unwrap() = crate::commands::set_display_for_performance();
    //         }

    //     });
    // });

    // ---- Advanced Tweaks ----
    ui.group(|ui| {
        ui.label("Advanced Tweaks");
        ui.horizontal_wrapped(|ui| {
            if ui.button("🚫 Adobe Network Block")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 80, 80),
                            "Blocks Adobe activation & telemetry servers"
                        );
                        ui.label("• Edits the HOSTS file with blocklist");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ Requires admin rights"
                        );
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            "ℹ DNS cache will be flushed"
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::adobe_network_block();
            }

            if ui.button("📉 Debloat Adobe")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "Disables Adobe background services & updates"
                        );
                        ui.label("• Stops Adobe Desktop Service");
                        ui.label("• Disables Acrobat auto updates");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ May break Adobe CC auto updates"
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::adobe_debloat();
            }

            if ui.button("🚫 Disable Microsoft Copilot")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 100, 100),
                            "Removes Microsoft Copilot integration"
                        );
                        ui.label("• Disables registry & Copilot button");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ Requires Windows 23H2+"
                        );
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            "ℹ Restart required to apply"
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::disable_copilot();
            }

            if ui.button("🖥 Set Display for Performance")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20), // verde neon
                            "✔ Optimizes system for best performance"
                        );
                        ui.label("• Disables animations and visual effects");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ May make UI less smooth but faster"
                        );
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            "ℹ Requires logoff/restart to fully apply"
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = crate::commands::set_display_for_performance();
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

            if ui.button("▶ Run").clicked() {
                let provider = selected.to_string();
                *log.lock().unwrap() = format!("⏳ Setting DNS to {provider}...");
                let log_clone = log.clone();

                thread::spawn(move || {
                    let result = commands::set_dns(&provider);
                    *log_clone.lock().unwrap() = result;
                });
            }
        });
    });


    ui.add_space(6.0);

    // ---- Power Plans ----
    ui.group(|ui| {
        ui.label("Power Plans");
        ui.horizontal_wrapped(|ui| {
            if ui.button("⚡ High Performance")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "High Performance Plan"
                        );
                        ui.label("• Maximizes performance at the cost of higher power usage.");
                        ui.label("• Keeps CPU and GPU at higher frequencies.");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ Recommended for desktops or when on AC power."
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() =
                    commands::power_plan_switcher("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c");
            }

            if ui.button("🔌 Balanced")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "Balanced Plan"
                        );
                        ui.label("• Default Windows plan (best for most users).");
                        ui.label("• Dynamically balances performance and energy usage.");
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            "ℹ Recommended for laptops and general use."
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() =
                    commands::power_plan_switcher("381b4222-f694-41f0-9685-ff5bb260df2e");
            }

            if ui.button("🔋 Power Saver")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "Power Saver Plan"
                        );
                        ui.label("• Reduces system performance to save battery life.");
                        ui.label("• Lowers CPU frequencies and dims display faster.");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ Recommended only when running low on battery."
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() =
                    commands::power_plan_switcher("a1841308-3541-4fab-bc81-f71556f20b4a");
            }
        });
    });

    ui.add_space(6.0);

    // ---- Power Tweaks ----
    ui.group(|ui| {
        ui.label("Power Tweaks");
        ui.horizontal_wrapped(|ui| {
            if ui.button("💤 Disable Sleep")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "Disable Sleep Mode"
                        );
                        ui.label("• Prevents Windows from going into sleep mode.");
                        ui.label("• Useful for servers, media PCs, or long-running tasks.");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ May increase power consumption."
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::disable_sleep();
            }

            if ui.button("💽 Disable HDD/SSD Timeout")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "Disable Disk Timeout"
                        );
                        ui.label("• Prevents hard drives and SSDs from powering down after inactivity.");
                        ui.label("• Can improve responsiveness on systems with multiple drives.");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ Continuous spinning may reduce HDD lifespan slightly."
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::disable_hdd();
            }

            if ui.button("🖥️ Disable Monitor Timeout")
                .on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(57, 255, 20),
                            "Disable Display Timeout"
                        );
                        ui.label("• Prevents the monitor from turning off automatically.");
                        ui.label("• Useful for presentations, kiosks, or media setups.");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ Monitor stays on constantly → higher energy use."
                        );
                    });
                })
                .clicked()
            {
                *log.lock().unwrap() = commands::disable_monitor();
            }
        });
    });
}
