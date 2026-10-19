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

fn lenient_number<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::Number(n)) => n.as_f64(),
        Some(serde_json::Value::String(text)) => text.trim().parse::<f64>().ok(),
        _ => None,
    })
}

fn null_as_empty<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct DiskHealthRoot {
    #[serde(default, deserialize_with = "null_as_empty")]
    smart_devices: Vec<SmartDevice>,
    #[serde(default, deserialize_with = "null_as_empty")]
    physical_disks: Vec<PhysicalDisk>,
}

#[derive(Debug, Deserialize)]
struct SmartDevice {
    #[serde(default)]
    predict_failure: bool,
    #[serde(default, deserialize_with = "null_as_empty")]
    attributes: Vec<SmartAttribute>,
}

#[derive(Debug, Deserialize)]
struct SmartAttribute {
    #[serde(default, deserialize_with = "lenient_number")]
    id: Option<f64>,
    #[serde(default, deserialize_with = "lenient_number")]
    raw: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PhysicalDisk {
    #[serde(default)]
    friendly_name: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    media_type: Option<String>,
    #[serde(default)]
    bus_type: Option<String>,
    #[serde(default, deserialize_with = "lenient_number")]
    size_bytes: Option<f64>,
    /// 0 = new, 100 = worn
    #[serde(default, deserialize_with = "lenient_number")]
    wear_percent_used: Option<f64>,
    /// 0 = worn, 100 = new
    #[serde(default, deserialize_with = "lenient_number")]
    health_percent: Option<f64>,
    #[serde(default, deserialize_with = "lenient_number")]
    power_on_hours: Option<f64>,
    #[serde(default, deserialize_with = "lenient_number")]
    temperature_c: Option<f64>,
    #[serde(default, deserialize_with = "lenient_number")]
    total_bytes_written: Option<f64>,
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

    if data.physical_disks.is_empty() {
        ui.label("No disks were returned.");
        return;
    }

    // WMI SMART rows cannot be matched to a physical disk reliably; use them only when there is
    // exactly one of each (NVMe drives usually have none).
    let single_smart = if data.physical_disks.len() == 1 && data.smart_devices.len() == 1 {
        data.smart_devices.first()
    } else {
        None
    };

    for phys in &data.physical_disks {
        show_disk(ui, phys, single_smart);
    }
}

fn show_disk(ui: &mut egui::Ui, phys: &PhysicalDisk, smart: Option<&SmartDevice>) {
    ui.separator();
    let name = phys.model.clone().or_else(|| phys.friendly_name.clone()).unwrap_or_else(|| "Disk".to_string());
    let size_gb = phys.size_bytes.map(|b| (b / 1_000_000_000.0).round() as u64).unwrap_or(0);
    ui.label(
        RichText::new(format!(
            "{}  •  {}  •  {}  •  {} GB",
            name,
            phys.media_type.clone().unwrap_or_default(),
            phys.bus_type.clone().unwrap_or_default(),
            size_gb
        ))
        .strong(),
    );

    if let Some(temp) = phys.temperature_c {
        ui.label(format!("Temperature: {:.0} °C", temp));
    }

    let wear = phys.wear_percent_used.map(|w| w.clamp(0.0, 100.0));
    let health = phys.health_percent.or_else(|| wear.map(|w| 100.0 - w)).map(|h| h.clamp(0.0, 100.0));
    ui.label(RichText::new("Health").strong());
    match health {
        Some(h) => {
            ui.add(egui::ProgressBar::new(h as f32 / 100.0).show_percentage());
        }
        None => {
            ui.label("N/A (the drive does not report wear data)");
        }
    }

    if let Some(smart) = smart {
        let status = if smart.predict_failure { "⚠ SMART predicts failure — back up your data" } else { "SMART status: OK" };
        ui.label(status);
    }

    let power_on = phys.power_on_hours.map(|h| h as u64).or_else(|| smart.and_then(|s| smart_raw(s, 9)));
    match power_on {
        Some(hours) => {
            let (days, rem_hours) = hours_to_days(hours);
            ui.label(format!("Power on time: {} hours ({} days {} hours)", hours, days, rem_hours));
        }
        None => {
            ui.label("Power on time: N/A");
        }
    }

    match estimate_remaining_days(power_on, wear) {
        Some(days) => ui.label(format!("Estimated remaining lifetime: {days} days")),
        None => ui.label("Estimated remaining lifetime: N/A"),
    };

    let written = phys
        .total_bytes_written
        .filter(|v| *v > 0.0)
        .map(|v| v as u64)
        .or_else(|| smart.and_then(|s| smart_raw(s, 241)).map(|lba| lba * 512));
    match written {
        Some(bytes) => ui.label(format!("Lifetime writes: {:.2} GB", bytes as f64 / 1_000_000_000.0)),
        None => ui.label("Lifetime writes: N/A"),
    };
    ui.add_space(8.0);
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

fn smart_raw(smart: &SmartDevice, id: u32) -> Option<u64> {
    smart
        .attributes
        .iter()
        .find(|a| a.id.map(|v| v as u32) == Some(id))
        .and_then(|a| a.raw)
        .map(|v| v as u64)
}

fn hours_to_days(hours: u64) -> (u64, u64) {
    (hours / 24, hours % 24)
}

fn estimate_remaining_days(power_on: Option<u64>, wear_used: Option<f64>) -> Option<u64> {
    let used = wear_used?.min(100.0);
    if used <= 0.0 {
        return None;
    }
    let hours = power_on? as f64;
    let total_expected = hours * (100.0 / used);
    Some(((total_expected - hours) / 24.0).floor().max(0.0) as u64)
}
