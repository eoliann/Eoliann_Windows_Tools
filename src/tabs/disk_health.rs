use std::sync::{Arc, Mutex, mpsc};
use egui::{self, RichText};
use serde::Deserialize;

use crate::commands;

/* =======================
   STATE (stored in App)
   ======================= */

#[derive(Default)]
pub struct DiskHealthState {
    loading: bool,
    rx: Option<mpsc::Receiver<Result<DiskHealthRoot, String>>>,
    data: Option<DiskHealthRoot>,
    error: Option<String>,
}

/* =======================
   JSON STRUCTURES
   ======================= */

#[derive(Debug, Deserialize)]
struct DiskHealthRoot {
    smart_devices: Vec<SmartDevice>,
    physical_disks: Vec<PhysicalDisk>,
}

#[derive(Debug, Deserialize)]
struct SmartDevice {
    #[serde(rename = "instance_name")]
    _instance_name: String,
    predict_failure: bool,
    attributes: Vec<SmartAttribute>,
}

#[derive(Debug, Deserialize)]
struct SmartAttribute {
    id: u32,
    #[serde(rename = "name")]
    _name: Option<String>,
    raw: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PhysicalDisk {
    device_id: String,
    friendly_name: String,
    model: String,
    media_type: String,
    bus_type: String,
    size_bytes: u64,

    wear_percent_used: u32,   // 0 = new, 100 = worn
    health_percent: u32,      // 0 = worn, 100 = new

    power_on_hours: Option<u64>, // often null → ignore
    temperature_c: Option<i32>,
    total_bytes_written: Option<u64>,
}

/* =======================
   PUBLIC UI ENTRY POINT
   ======================= */

pub fn show_disk_health(
    ui: &mut egui::Ui,
    log_output: &Arc<Mutex<String>>,
    state: &mut DiskHealthState,
) {
    poll_worker(state);

    // auto-refresh once
    if state.data.is_none() && !state.loading && state.error.is_none() {
        start_refresh(state, log_output.clone());
    }

    ui.heading("💽 Disk Health");
    ui.separator();

    ui.horizontal(|ui| {
        if ui.add_enabled(!state.loading, egui::Button::new("🔄 Refresh")).clicked() {
            start_refresh(state, log_output.clone());
        }

        if state.loading {
            ui.add(egui::Spinner::new());
            ui.label("Collecting disk health information…");
        }
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
        return;
    }

    let Some(data) = &state.data else {
        ui.label("No disk data available.");
        return;
    };

    if data.physical_disks.is_empty() || data.smart_devices.is_empty() {
        ui.label("No disks were returned.");
        return;
    }

    // One-to-one mapping (works for your system; extend later if needed)
    let phys = &data.physical_disks[0];
    let smart = &data.smart_devices[0];

    ui.separator();
    ui.label(
        RichText::new(format!(
            "{}  •  {}  •  {} GB",
            phys.model,
            phys.bus_type,
            phys.size_bytes / 1_000_000_000
        ))
        .strong(),
    );

    if let Some(temp) = phys.temperature_c {
        ui.label(format!("Temperature: {} °C", temp));
    }

    ui.separator();

    /* =======================
       HEALTH
       ======================= */

    let health = phys.health_percent.min(100);
    ui.label(RichText::new("Health").strong());
    ui.add(egui::ProgressBar::new(health as f32 / 100.0).show_percentage());

    ui.add_space(10.0);

    /* =======================
       PERFORMANCE
       ======================= */

    let performance = if smart.predict_failure { 0 } else { 100 };
    ui.label(RichText::new("Performance").strong());
    ui.add(egui::ProgressBar::new(performance as f32 / 100.0).show_percentage());

    ui.add_space(14.0);

    /* =======================
       POWER ON TIME (SMART ID 9)
       ======================= */

    if let Some(hours) = extract_power_on_hours(smart) {
        let (days, rem_hours) = hours_to_days(hours);
        ui.label(RichText::new("Power on time").strong());
        ui.label(format!(
            "{} hours  ({} days {} hours)",
            hours, days, rem_hours
        ));
    } else {
        ui.label("Power on time: N/A");
    }

    ui.add_space(14.0);

    /* =======================
       ESTIMATED REMAINING LIFE
       ======================= */

    ui.label(RichText::new("Estimated remaining lifetime").strong());
    match estimate_remaining_days(
        extract_power_on_hours(smart),
        phys.wear_percent_used,
    ) {
        Some(days) => ui.label(format!("{days} days")),
        None => ui.label("N/A"),
    };

    ui.add_space(14.0);

    /* =======================
       LIFETIME WRITES
       ======================= */

    ui.label(RichText::new("Lifetime writes").strong());
    match extract_lifetime_writes(phys, smart) {
        Some(bytes) => ui.label(format!("{tb:.2} GB ({bytes} bytes)", tb = bytes as f64 / 1_000_000_000.0, bytes = bytes)),
        None => ui.label("N/A"),
    };
}

/* =======================
   BACKGROUND WORKER
   ======================= */

fn start_refresh(state: &mut DiskHealthState, log: Arc<Mutex<String>>) {
    if state.loading {
        return;
    }

    state.loading = true;
    state.error = None;

    let (tx, rx) = mpsc::channel();
    state.rx = Some(rx);

    std::thread::spawn(move || {
        let raw = commands::disk_health_report_json();

        let json = match extract_json_object(&raw) {
            Some(v) => v,
            None => {
                let _ = tx.send(Err("Failed to extract JSON object.".into()));
                return;
            }
        };

        match serde_json::from_str::<DiskHealthRoot>(&json) {
            Ok(data) => {
                let _ = tx.send(Ok(data));
            }
            Err(e) => {
                let mut lg = log.lock().unwrap();
                *lg = format!("{}\n\nRaw JSON:\n{}", e, json);
                let _ = tx.send(Err(format!(
                    "Disk health: failed to parse JSON: {e}"
                )));
            }
        }
    });
}

fn poll_worker(state: &mut DiskHealthState) {
    if let Some(rx) = &state.rx {
        if let Ok(result) = rx.try_recv() {
            state.loading = false;
            state.rx = None;

            match result {
                Ok(data) => state.data = Some(data),
                Err(e) => state.error = Some(e),
            }
        }
    }
}

/* =======================
   HELPERS
   ======================= */

fn extract_json_object(s: &str) -> Option<String> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end > start {
        Some(s[start..=end].to_string())
    } else {
        None
    }
}

fn extract_power_on_hours(smart: &SmartDevice) -> Option<u64> {
    smart
        .attributes
        .iter()
        .find(|a| a.id == 9)
        .map(|a| a.raw)
}

fn hours_to_days(hours: u64) -> (u64, u64) {
    (hours / 24, hours % 24)
}

fn estimate_remaining_days(power_on: Option<u64>, wear_used: u32) -> Option<u64> {
    let used = wear_used.min(100);
    if used == 0 {
        return None;
    }

    let hours = power_on?;
    let total_expected = hours as f64 * (100.0 / used as f64);
    let remaining = total_expected - hours as f64;
    Some((remaining / 24.0).floor() as u64)
}

fn extract_lifetime_writes(
    phys: &PhysicalDisk,
    smart: &SmartDevice,
) -> Option<u64> {
    if let Some(v) = phys.total_bytes_written {
        if v > 0 {
            return Some(v);
        }
    }

    // SMART fallback: ID 241 (Total LBAs Written)
    smart
        .attributes
        .iter()
        .find(|a| a.id == 241)
        .map(|a| a.raw * 512)
}
