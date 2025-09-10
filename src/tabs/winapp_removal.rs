use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use crate::commands;
use crate::utils; // pentru run_command la verificarea instalării

// -------------------- Modelul de date --------------------

#[derive(Clone, Copy)]
struct Item {
    group: &'static str,
    label: &'static str,
    pkg:   &'static str,
}

// Lista fixă de aplicații (categorii + pachete), în ordinea cerută.
const ITEMS: &[Item] = &[
    // Communication
    Item { group: "Communication", label: "Outlook for Windows", pkg: "Microsoft.OutlookForWindows" },
    Item { group: "Communication", label: "Skype",                pkg: "Microsoft.SkypeApp" },
    Item { group: "Communication", label: "Teams",                pkg: "MSTeams" },
    Item { group: "Communication", label: "GroupMe",              pkg: "Microsoft.GroupMe10" },
    Item { group: "Communication", label: "To-Do",                pkg: "Microsoft.Todos" },
    Item { group: "Communication", label: "Your Phone",           pkg: "Microsoft.YourPhone" },
    Item { group: "Communication", label: "CommsPhone",           pkg: "Microsoft.CommsPhone" },
    Item { group: "Communication", label: "Messaging",            pkg: "Microsoft.Messaging" },
    Item { group: "Communication", label: "Mail & Calendar",      pkg: "microsoft.windowscommunicationsapps" },

    // Media & Creativity
    Item { group: "Media & Creativity", label: "Clipchamp",                 pkg: "Clipchamp.Clipchamp" },
    Item { group: "Media & Creativity", label: "Camera",                    pkg: "Microsoft.WindowsCamera" },
    Item { group: "Media & Creativity", label: "MS Paint",                  pkg: "Microsoft.MSPaint" },
    Item { group: "Media & Creativity", label: "3D Builder",                pkg: "Microsoft.3DBuilder" },
    Item { group: "Media & Creativity", label: "3D Viewer",                 pkg: "Microsoft.Microsoft3DViewer" },
    Item { group: "Media & Creativity", label: "Print 3D",                  pkg: "Microsoft.Print3D" },
    Item { group: "Media & Creativity", label: "Mixed Reality Portal",      pkg: "Microsoft.MixedReality.Portal" },

    // Microsoft Apps
    Item { group: "Microsoft Apps", label: "Office Hub",    pkg: "Microsoft.MicrosoftOfficeHub" },
    Item { group: "Microsoft Apps", label: "OneNote",       pkg: "Microsoft.Office.OneNote" },
    Item { group: "Microsoft Apps", label: "Sway",          pkg: "Microsoft.Office.Sway" },
    Item { group: "Microsoft Apps", label: "Sticky Notes",  pkg: "Microsoft.MicrosoftStickyNotes" },
    Item { group: "Microsoft Apps", label: "Family Safety", pkg: "MicrosoftCorporationII.MicrosoftFamily" },

    // Bing Apps
    Item { group: "Bing Apps", label: "Bing",          pkg: "Microsoft.549981C3F5F10" }, // Xbox App Stub / Bing shell
    Item { group: "Bing Apps", label: "Bing Weather",  pkg: "Microsoft.BingWeather" },
    Item { group: "Bing Apps", label: "Bing Sports",   pkg: "Microsoft.BingSports" },
    Item { group: "Bing Apps", label: "Bing Finance",  pkg: "Microsoft.BingFinance" },
    Item { group: "Bing Apps", label: "Bing News",     pkg: "Microsoft.BingNews" },

    // Games
    Item { group: "Games", label: "Solitaire Collection",   pkg: "Microsoft.MicrosoftSolitaireCollection" },
    Item { group: "Games", label: "Minecraft for Windows",  pkg: "Microsoft.MinecraftUWP" },

    // Systems & Misc
    Item { group: "Systems & Misc", label: "People",          pkg: "Microsoft.People" },
    Item { group: "Systems & Misc", label: "Maps",            pkg: "Microsoft.WindowsMaps" },
    Item { group: "Systems & Misc", label: "Wallet",          pkg: "Microsoft.Wallet" },
    Item { group: "Systems & Misc", label: "Get Started",     pkg: "Microsoft.Getstarted" },
    Item { group: "Systems & Misc", label: "Feedback Hub",    pkg: "Microsoft.WindowsFeedbackHub" },
    Item { group: "Systems & Misc", label: "Alarms & Clock",  pkg: "Microsoft.WindowsAlarms" },
    Item { group: "Systems & Misc", label: "OneConnect",      pkg: "Microsoft.OneConnect" },
    Item { group: "Systems & Misc", label: "Windows Phone",   pkg: "Microsoft.WindowsPhone" },
    Item { group: "Systems & Misc", label: "Voice Recorder",  pkg: "Microsoft.WindowsSoundRecorder" },
];

// Col0 vs Col1: ca în layout-ul original
const COL0_GROUPS: &[&str] = &["Communication", "Media & Creativity", "Microsoft Apps"];
const COL1_GROUPS: &[&str] = &["Bing Apps", "Games", "Systems & Misc"];

// -------------------- Stări persistente (fără a modifica App) --------------------

// Selecțiile checkbox-urilor
static SELECTION: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();

// Progres pentru bara de progres de sus
#[derive(Clone, Default)]
struct Progress {
    running: bool,
    current: usize,
    total: usize,
    label: String,
}
static PROGRESS: OnceLock<Mutex<Progress>> = OnceLock::new();

fn selection_state() -> &'static Mutex<Vec<bool>> {
    SELECTION.get_or_init(|| Mutex::new(vec![false; ITEMS.len()]))
}

fn progress_state() -> &'static Mutex<Progress> {
    PROGRESS.get_or_init(|| Mutex::new(Progress::default()))
}

// -------------------- Utils --------------------

fn append_line(log: &Arc<Mutex<String>>, line: impl AsRef<str>) {
    let line = line.as_ref();
    if let Ok(mut lg) = log.lock() {
        lg.push_str(line);
        if !line.ends_with('\n') {
            lg.push('\n');
        }
    }
}

// Verifică dacă o aplicație este instalată (user sau provisioned)
fn is_app_installed(pkg: &str) -> bool {
    // Folosim PowerShell; ascuns datorită utils::run_command (care folosește CREATE_NO_WINDOW pe Windows)
    let ps = format!(
        r#"powershell -ExecutionPolicy Unrestricted -NoProfile -Command "$u = Get-AppxPackage -Name '{0}' -ErrorAction SilentlyContinue | Measure-Object; $p = Get-AppxProvisionedPackage -Online | Where-Object DisplayName -like '{0}' | Measure-Object; if (($u.Count -gt 0) -or ($p.Count -gt 0)) {{ '1' }} else {{ '0' }}""#,
        pkg
    );
    let out = utils::run_command(&ps);
    out.trim().ends_with('1')
}

// -------------------- UI principal --------------------

pub fn show_winapp_removal(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    _show_popup: &mut bool,
    _popup_message: &mut String,
) {
    // asigură mărimea selecțiilor (în caz de schimbări de listă)
    {
        let mut sel = selection_state().lock().unwrap();
        if sel.len() != ITEMS.len() {
            sel.resize(ITEMS.len(), false);
        }
    }

    ui.heading("🗑 WinApp Removal");
    ui.add_space(4.0);

    // --- mic output cu progres (sus) ---
    {
        let p = progress_state().lock().unwrap().clone();
        if p.running {
            ui.horizontal(|ui| {
                ui.label(format!("Removing: {}", p.label));
                let frac = if p.total == 0 { 0.0 } else { p.current as f32 / p.total as f32 };
                ui.add(egui::ProgressBar::new(frac).show_percentage());
            });
            ui.add_space(6.0);
        }
    }

    // --- butoane de acțiune ---
    ui.horizontal(|ui| {
        if ui.button("Remove ALL listed apps").clicked() {
            spawn_bulk(false, log.clone());
        }
        if ui.button("Force remove ALL").clicked() {
            spawn_bulk(true, log.clone());
        }
        if ui.button("Remove SELECTED").clicked() {
            spawn_selected(false, log.clone());
        }
        if ui.button("Force remove SELECTED").clicked() {
            spawn_selected(true, log.clone());
        }
    });

    ui.add_space(6.0);

    // --- zonă scrollabilă cu toate categoriile și checkbox-uri ---
    let avail_h = ui.available_height();
    egui::ScrollArea::vertical()
        .id_salt("winapp_scroll_v3")
        .auto_shrink([false; 2])
        .max_height(avail_h)
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                render_column(&mut cols[0], COL0_GROUPS);
                render_column(&mut cols[1], COL1_GROUPS);
            });
        });
}

// -------------------- Render helpers --------------------

fn render_column(ui: &mut egui::Ui, groups: &[&str]) {
    for &g in groups {
        ui.group(|ui| {
            ui.label(egui::RichText::new(g).size(16.0));
            let mut sel = selection_state().lock().unwrap();
            for (i, it) in ITEMS.iter().enumerate().filter(|(_, it)| it.group == g) {
                let mut v = sel[i];
                // checkbox toggle
                if ui.checkbox(&mut v, it.label).clicked() {
                    sel[i] = v;
                }
            }
        });
        ui.add_space(6.0);
    }
}

// -------------------- Task runners --------------------

fn spawn_bulk(force: bool, log: Arc<Mutex<String>>) {
    let total = ITEMS.len();
    {
        let mut p = progress_state().lock().unwrap();
        p.running = true;
        p.current = 0;
        p.total = total;
        p.label.clear();
    }

    thread::spawn(move || {
        append_line(&log, if force { "📦 Force bulk removal started..." } else { "📦 Bulk removal started..." });

        for (idx, it) in ITEMS.iter().enumerate() {
            {
                let mut p = progress_state().lock().unwrap();
                p.current = idx;
                p.label = format!("{} ({}/{})", it.label, idx + 1, total);
            }
            append_line(&log, format!("→ {} {}", if force { "Force-removing" } else { "Removing" }, it.label));
            let res = if force {
                commands::remove_app_force(it.pkg)
            } else {
                commands::remove_app(it.pkg)
            };
            append_line(&log, res);
        }

        {
            let mut p = progress_state().lock().unwrap();
            p.current = total;
            p.label.clear();
            p.running = false;
        }
        append_line(&log, if force { "✅ Force bulk removal finished." } else { "✅ Bulk removal finished." });
    });
}

fn spawn_selected(force: bool, log: Arc<Mutex<String>>) {
    let indices: Vec<usize> = {
        let sel = selection_state().lock().unwrap();
        sel.iter().enumerate().filter_map(|(i, &v)| if v { Some(i) } else { None }).collect()
    };

    if indices.is_empty() {
        append_line(&log, "ℹ No items selected.");
        return;
    }

    let total = indices.len();
    {
        let mut p = progress_state().lock().unwrap();
        p.running = true;
        p.current = 0;
        p.total = total;
        p.label.clear();
    }

    thread::spawn(move || {
        append_line(&log, if force { "📦 Force removal (selected) started..." } else { "📦 Removal (selected) started..." });

        for (pos, idx) in indices.iter().enumerate() {
            let it = ITEMS[*idx];
            {
                let mut p = progress_state().lock().unwrap();
                p.current = pos;
                p.label = format!("{} ({}/{})", it.label, pos + 1, total);
            }

            // verifică dacă e instalat
            if !is_app_installed(it.pkg) {
                append_line(&log, format!("ℹ {} is not installed. Skipping.", it.label));
                continue;
            }

            append_line(&log, format!("→ {} {}", if force { "Force-removing" } else { "Removing" }, it.label));
            let res = if force {
                commands::remove_app_force(it.pkg)
            } else {
                commands::remove_app(it.pkg)
            };
            append_line(&log, res);
        }

        // finalizează progresul
        {
            let mut p = progress_state().lock().unwrap();
            p.current = total;
            p.label.clear();
            p.running = false;
        }

        // debifează elementele selectate după terminare
        {
            let mut sel = selection_state().lock().unwrap();
            for idx in indices {
                if idx < sel.len() {
                    sel[idx] = false;
                }
            }
        }

        append_line(&log, if force { "✅ Force removal (selected) finished." } else { "✅ Removal (selected) finished." });
    });
}
