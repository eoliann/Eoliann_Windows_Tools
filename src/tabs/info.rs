// src/tabs/info.rs
use crate::utils::run_command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::process::Command;
use std::collections::HashMap;

use egui::{Color32, RichText, Ui};


#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// State + data
pub struct InfoState {
    pub data: Option<InfoData>,
    pub loading: bool,
}

impl InfoState {
    pub fn new() -> Self {
        Self {
            data: None,
            loading: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InfoData {
    pub pc_name: String,
    pub windows_edition: String,
    pub processor: String,
    pub ram_total: u64,
    pub storage_total: u64,
    pub gpu_name: String,
    pub gpu_ram: u64,
    pub installed_apps: usize,
    pub processes_count: usize,
    pub services_running: usize,
    // NEW network fields
    pub network_iface: String,
    pub network_ip: String,
}

fn run_pwsh(cmd: &str) -> String {
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-NonInteractive", "-Command", cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        match output {
            Ok(o) => {
                if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).trim().to_string()
                } else {
                    let s = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    if s.is_empty() { String::new() } else { s }
                }
            }
            Err(_) => String::new(),
        }
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("sh").arg("-c").arg(cmd).output();
        match output {
            Ok(o) => {
                if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).trim().to_string()
                } else {
                    String::from_utf8_lossy(&o.stderr).trim().to_string()
                }
            }
            Err(_) => String::new(),
        }
    }
}

fn format_bytes(b: u64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    format!("{:.2} GB", (b as f64) / GB)
}

fn parse_number(s: &str) -> Option<u64> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() { None } else { digits.parse::<u64>().ok() }
}

/// Blocking collection logic (runs inside a thread)
fn collect_info_blocking() -> InfoData {
    // hostname via run_command (keeps existing behaviour for simple commands)
    let pc_name = {
        let out = run_command("hostname").trim().to_string();
        if out.is_empty() { "Unknown".to_string() } else { out }
    };

    // Windows edition via proper PowerShell invocation
    let windows_edition = {
        let out = run_pwsh("(Get-CimInstance -ClassName Win32_OperatingSystem).Caption");
        if out.is_empty() { "Unknown".to_string() } else { out }
    };

    // Processor name
    let processor = {
        let out = run_pwsh("(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name).Trim()");
        if out.is_empty() { "Unknown".to_string() } else { out }
    };

    // RAM total (bytes)
    let ram_total = {
        let out = run_pwsh("(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory");
        parse_number(&out).unwrap_or(0)
    };

    // Storage total (sum of fixed disks)
    let storage_total = {
        let out = run_pwsh("Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3' | Measure-Object -Property Size -Sum | Select-Object -ExpandProperty Sum");
        parse_number(&out).unwrap_or(0)
    };

    // GPU name and memory
    let (gpu_name, gpu_ram) = {
        let out = run_pwsh(r#"Get-CimInstance Win32_VideoController | Select-Object -First 1 -Property Name,AdapterRAM | ForEach-Object { "$($_.Name)|$($_.AdapterRAM)" }"#);
        if out.contains('|') {
            let mut parts = out.splitn(2, '|');
            let name = parts.next().unwrap_or("Unknown").trim().to_string();
            let ram = parts.next().and_then(|s| parse_number(s)).unwrap_or(0);
            (name, ram)
        } else {
            ("Unknown".to_string(), 0u64)
        }
    };

    // Installed applications count (registry)
    let installed_apps = {
        let out = run_pwsh(r#"Get-ItemProperty HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\* , HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\* | Where-Object { $_.DisplayName } | Select-Object -ExpandProperty DisplayName | Measure-Object | Select-Object -ExpandProperty Count"#);
        parse_number(&out).map(|v| v as usize).unwrap_or(0)
    };

    // Processes count
    let processes_count = {
        let out = run_pwsh("Get-Process | Measure-Object | Select-Object -ExpandProperty Count");
        parse_number(&out).map(|v| v as usize).unwrap_or(0)
    };

    // Services running count
    let services_running = {
        let out = run_pwsh("Get-Service | Where-Object { $_.Status -eq 'Running' } | Measure-Object | Select-Object -ExpandProperty Count");
        parse_number(&out).map(|v| v as usize).unwrap_or(0)
    };

    // Network: interface alias + IPv4 address (pick first non-link-local/non-loopback)
    let (network_iface, network_ip) = {
        let out = run_pwsh(r#"Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike '169.*' -and $_.IPAddress -notlike '127.*' } | Select-Object -First 1 -Property IPAddress,InterfaceAlias | ForEach-Object { "$($_.InterfaceAlias)|$($_.IPAddress)" }"#);
        if out.contains('|') {
            let mut parts = out.splitn(2, '|');
            let iface = parts.next().unwrap_or("Unknown").trim().to_string();
            let ip = parts.next().unwrap_or("Unknown").trim().to_string();
            (iface, ip)
        } else {
            ("Unknown".to_string(), "Unknown".to_string())
        }
    };

    InfoData {
        pc_name,
        windows_edition,
        processor,
        ram_total,
        storage_total,
        gpu_name,
        gpu_ram,
        installed_apps,
        processes_count,
        services_running,
        network_iface,
        network_ip,
    }
}

/// Start background collection only when needed (force = true to re-run)
pub fn start_collect_if_needed(state: Arc<Mutex<InfoState>>, log_output: Option<Arc<Mutex<String>>>, force: bool) {
    let should_spawn = {
        let mut st = state.lock().unwrap();
        if st.loading {
            false
        } else if st.data.is_some() && !force {
            false
        } else {
            st.loading = true;
            if force { st.data = None; }
            true
        }
    };

    if !should_spawn { return; }

    let thread_state = state.clone();
    let thread_log = log_output.clone();

    thread::spawn(move || {
        if let Some(lg) = &thread_log {
            let mut l = lg.lock().unwrap();
            l.push_str("Started collecting system info...\n");
        }

        thread::sleep(Duration::from_millis(50));
        let info = collect_info_blocking();

        {
            let mut st = thread_state.lock().unwrap();
            st.data = Some(info);
            st.loading = false;
        }

        if let Some(lg) = &thread_log {
            let mut l = lg.lock().unwrap();
            l.push_str("Finished collecting system info\n");
        }
    });
}

/// UI function — accept &Arc<Mutex<InfoState>> state and display
pub fn show_info(
    ui: &mut Ui,
    log_output: &Arc<Mutex<String>>,
    update_available: bool,
    latest_release: Option<&crate::utils::GithubRelease>,
    state: &Arc<Mutex<InfoState>>,
    icons: &HashMap<String, egui::TextureHandle>,
) {
    // header + logo (unchanged)
    let ascii_logo = r#"
    ███████╗ ██████╗ ██╗     ██╗ █████╗ ███╗   ██╗███╗   ██╗
    ██╔════╝██╔═══██╗██║     ██║██╔══██╗████╗  ██║████╗  ██║
    █████╗  ██║   ██║██║     ██║███████║██╔██╗ ██║██╔██╗ ██║
    ██╔══╝  ██║   ██║██║     ██║██╔══██║██║╚██╗██║██║╚██╗██║
    ███████╗╚██████╔╝███████╗██║██║  ██║██║ ╚████║██║ ╚████║
    ╚══════╝ ╚═════╝ ╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝
    "#;

    ui.label(RichText::new(ascii_logo).monospace().color(Color32::from_rgb(57, 255, 20)).size(16.0));
    ui.separator();
    ui.heading("Info");
    ui.add_space(8.0);

    // update banner (same)
    if update_available {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⬆ Update available").strong().color(Color32::from_rgb(0, 255, 140)));
                ui.add_space(8.0);
                if let Some(rel) = latest_release {
                    ui.label(RichText::new(format!("Latest: {}", rel.tag_name)).strong());
                    ui.add_space(8.0);
                    ui.hyperlink_to("Release notes", rel.html_url.as_str());
                    ui.add_space(8.0);
                    if ui.button("Open on GitHub").clicked() {
                        let _ = webbrowser::open(rel.html_url.as_str());
                        let mut lg = log_output.lock().unwrap();
                        lg.push_str(&format!("Opened release page: {}\n", rel.html_url));
                    }
                } else {
                    ui.label("A new version is available. Visit the project on GitHub.");
                    if ui.button("Open GitHub").clicked() {
                        let _ = webbrowser::open("https://github.com/eoliann/");
                        let mut lg = log_output.lock().unwrap();
                        lg.push_str("Opened GitHub repo\n");
                    }
                }
            });
        });
        ui.add_space(10.0);
    }

    // quick buttons (same)
    ui.horizontal_wrapped(|ui| {
        if ui.button("👤 whoami").on_hover_text("Displays the current logged-in username.").clicked() {
            let out = crate::utils::run_command("whoami");
            *log_output.lock().unwrap() = format!("> whoami\n{}", out);
        }
        if ui.button("🌐 ipconfig").on_hover_text("Shows detailed network configuration.").clicked() {
            let out = crate::utils::run_command("ipconfig /all");
            *log_output.lock().unwrap() = format!("> ipconfig /all\n{}", out);
        }
        if ui.button("💻 systeminfo").on_hover_text("Displays detailed system configuration.").clicked() {
            let out = crate::utils::run_command("systeminfo");
            *log_output.lock().unwrap() = format!("> systeminfo\n{}", out);
        }
        if ui.button("📋 tasklist").on_hover_text("Lists all running processes.").clicked() {
            let out = crate::utils::run_command("tasklist");
            *log_output.lock().unwrap() = format!("> tasklist\n{}", out);
        }
    });

    ui.separator();

    // System information section
    ui.add_space(6.0);
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("System information");
        ui.add_space(6.0);

        // Start collection once (non-blocking)
        start_collect_if_needed(state.clone(), Some(log_output.clone()), false);

        // grab a snapshot of state quickly
        let (maybe_data, is_loading) = {
            let st = state.lock().unwrap();
            (st.data.clone(), st.loading)
        };

        if is_loading && maybe_data.is_none() {
            ui.label(RichText::new("Loading system information...").italics());
        }

        if let Some(data) = maybe_data {
            // cards: (icon_key, title, value)
            let cards: Vec<(&'static str, &'static str, String)> = vec![
                ("system", "PC Name", data.pc_name.clone()),
                ("windows", "Windows Edition", data.windows_edition.clone()),
                ("processor", "Processor", data.processor.clone()),
                ("memory", "RAM", format_bytes(data.ram_total)),
                ("storage", "Storage", format_bytes(data.storage_total)),
                ("graphics", "Graphics Card", format!("{} ({})", data.gpu_name, format_bytes(data.gpu_ram))),
                ("apps", "Installed apps", format!("{}", data.installed_apps)),
                ("processes", "Open processes", format!("{}", data.processes_count)),
                ("services", "Services running", format!("{}", data.services_running)),
                ("network", "Network", format!("{} — {}", data.network_iface, data.network_ip)),
            ];

            // two-column responsive layout: each column gets available width,
            // inside each card we allocate a fixed area for the text equal to (col_width - icon - paddings)
            ui.columns(2, |cols| {
                for (i, (icon_key, title, value)) in cards.into_iter().enumerate() {
                    let idx = i % 2;
                    let col = &mut cols[idx];

                    col.add_space(6.0);

                    egui::Frame::group(col.style()).show(col, |ui| {
                        // capture column available width BEFORE we add icon & spacing
                        let col_available = ui.available_width();

                        ui.horizontal(|ui| {
                            // fixed-size icon
                            crate::ui_helpers::draw_icon(ui, icons, icon_key, 28.0);
                            ui.add_space(8.0);

                            // compute width for the label area (leave some padding)
                            let reserved_for_icon = 28.0;
                            let padding = 8.0 + 12.0; // space + extra margin
                            let label_w = (col_available - reserved_for_icon - padding).max(80.0);

                            // allocate a sub-UI with a fixed width so labels wrap to this width
                            ui.allocate_ui_with_layout(
                                egui::vec2(label_w, ui.available_height()),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.add(egui::Label::new(RichText::new(title).size(14.0)).wrap());
                                    ui.add_space(4.0);
                                    ui.add(egui::Label::new(RichText::new(value).size(18.0).strong()).wrap());
                                },
                            );
                        });
                    });

                    col.add_space(6.0);
                }
            });
        }

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.small_button("Refresh system info").clicked() {
                start_collect_if_needed(state.clone(), Some(log_output.clone()), true);
                let mut l = log_output.lock().unwrap();
                l.push_str("Refresh requested for system information\n");
            }
            if ui.small_button("Show last log").clicked() {
                let mut l = log_output.lock().unwrap();
                l.push_str("User requested to view last log\n");
            }
        });
    });

    ui.add_space(12.0);

    ui.label("📖 About:");
    ui.label(format!("Eoliann Windows Tools Version {}", env!("CARGO_PKG_VERSION")));
    ui.add_space(8.0);
    ui.label("Created by Eoliann");
    ui.label("Quick tools for Windows administration.");

    if ui.button(RichText::new("🌐 Open GitHub Repo").color(Color32::from_rgb(57, 255, 20)).strong()).clicked() {
        *log_output.lock().unwrap() = run_command("explorer https://github.com/eoliann/");
    }
}
