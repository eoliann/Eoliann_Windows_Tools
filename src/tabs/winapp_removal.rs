use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use crate::utils;
use crate::commands;
 // must export run_powershell_cmd(...) and is_elevated()

// -------------------- Data model --------------------

#[derive(Clone, Copy)]
struct Item {
    group: &'static str,
    label: &'static str,
    pattern: &'static str, // substring/pattern used with -like "*pattern*"
}

// Complete ITEMS list (in-box / common built-in apps)
//
// Patterns use simple aliases separated by '|' (split_alts/pattern parsing in your code should handle this).
// Keep this block as the single source-of-truth for the UI.
const ITEMS: &[Item] = &[
    // Communication
    Item { group: "Communication", label: "Cortana",            pattern: "Microsoft.549981C3F5F10|cortana" },
    Item { group: "Communication", label: "Outlook for Windows", pattern: "outlook|microsoft.outlookforwindows" },
    Item { group: "Communication", label: "Skype",             pattern: "skype|microsoft.skypeapp" },
    Item { group: "Communication", label: "Teams",             pattern: "teams|microsoft.teams|msteams" },
    Item { group: "Communication", label: "GroupMe",           pattern: "groupme|microsoft.groupme" },
    Item { group: "Communication", label: "To Do",             pattern: "todos|todo|microsoft.todos|microsoft.officetodo" },
    Item { group: "Communication", label: "Phone Link",        pattern: "yourphone|phonelink|microsoft.yourphone|microsoft.phonelink" },
    Item { group: "Communication", label: "Messaging",         pattern: "messaging|microsoft.messaging" },
    Item { group: "Communication", label: "Mail & Calendar",   pattern: "windowscommunicationsapps|mail|microsoft.windowscommunicationsapps" },

    // Media & Creativity
    Item { group: "Media & Creativity", label: "Clipchamp",         pattern: "clipchamp|clipchamp.clipchamp" },
    Item { group: "Media & Creativity", label: "Camera",            pattern: "windowscamera|camera|microsoft.windowscamera" },
    Item { group: "Media & Creativity", label: "Photos",            pattern: "photos|microsoft.windows.photos|microsoft.photos" },
    Item { group: "Media & Creativity", label: "Photos Editor (optional)", pattern: "photoeditor|microsoft.photoeditor" },
    Item { group: "Media & Creativity", label: "Movies & TV (Media Player)", pattern: "zunevideo|microsoft.zunevideo|mediaplayer|microsoft.windowsmediaplayer" },
    Item { group: "Media & Creativity", label: "Groove Music",      pattern: "zunemusic|microsoft.zunemusic|music" },
    Item { group: "Media & Creativity", label: "MS Paint",          pattern: "mspaint|paint|microsoft.mspaint" },
    Item { group: "Media & Creativity", label: "3D Viewer",         pattern: "3dviewer|microsoft.microsoft3dviewer" },
    Item { group: "Media & Creativity", label: "3D Builder",        pattern: "3dbuilder|microsoft.3dbuilder" },
    Item { group: "Media & Creativity", label: "Print 3D",          pattern: "print3d|microsoft.print3d" },
    Item { group: "Media & Creativity", label: "Media Player (modern)", pattern: "mediaplayer|microsoft.windowsmediaplayer|microsoft.windows.media.player" },

    // Microsoft Apps / Productivity
    Item { group: "Microsoft Apps", label: "Office Hub",        pattern: "microsoftofficehub|officehub|microsoft.microsoftofficehub" },
    Item { group: "Microsoft Apps", label: "OneNote",           pattern: "onenote|microsoft.onenote" },
    Item { group: "Microsoft Apps", label: "Sway",              pattern: "sway|microsoft.office.sway" },
    Item { group: "Microsoft Apps", label: "Sticky Notes",      pattern: "stickynotes|microsoft.stickynotes|microsoft.microsoftstickynotes" },
    Item { group: "Microsoft Apps", label: "Family Safety",     pattern: "family|microsoft.family|microsoftcorporationii.microsoftfamily" },
    Item { group: "Microsoft Apps", label: "Power Automate",    pattern: "powerautomate|microsoft.powerautomate" },
    Item { group: "Microsoft Apps", label: "Notepad",           pattern: "notepad|microsoft.notepad" },

    // Bing / Web experience
    Item { group: "Bing Apps", label: "Bing (Web Experience Pack / shell)", pattern: "bing|windowswebexperiencepack|microsoft.webexperience|microsoft.549981c3f5f10" },
    Item { group: "Bing Apps", label: "Bing Weather",       pattern: "bingweather|microsoft.bingweather" },
    Item { group: "Bing Apps", label: "Bing Sports",        pattern: "bingsports|microsoft.bingsports" },
    Item { group: "Bing Apps", label: "Bing Finance",       pattern: "bingfinance|microsoft.bingfinance" },
    Item { group: "Bing Apps", label: "Bing News",          pattern: "bingnews|microsoft.bingnews" },
    Item { group: "Bing Apps", label: "Web Experience Pack (WebP/Web media)", pattern: "webexperience|webp|webmedia|microsoft.webexperiencepack" },

    // Games / Xbox
    Item { group: "Games", label: "Microsoft Store / Xbox App", pattern: "xbox|microsoft.xboxapp|microsoft.gamingapp|xboxapp" },
    Item { group: "Games", label: "Xbox Game Bar / Overlay", pattern: "xboxgamebar|xboxgameoverlay|microsoft.xboxgameoverlay" },
    Item { group: "Games", label: "Solitaire Collection", pattern: "microsoftsolitairecollection|solitaire|microsoft.microsoftsolitairecollection" },
    Item { group: "Games", label: "Minecraft for Windows", pattern: "minecraft|minecraftuwp|microsoft.minecraftuwp" },

    // System & Utilities
    Item { group: "Systems & Misc", label: "Calculator",        pattern: "calculator|windowscalculator|microsoft.windowscalculator" },
    Item { group: "Systems & Misc", label: "Maps",              pattern: "windowsmaps|maps|microsoft.windowsmaps" },
    Item { group: "Systems & Misc", label: "People",            pattern: "people|peopleexperiencehost|microsoft.people" },
    Item { group: "Systems & Misc", label: "Alarms & Clock",    pattern: "windowsalarms|alarms|microsoft.windowsalarms" },
    Item { group: "Systems & Misc", label: "Voice Recorder",    pattern: "soundrecorder|voicerecorder|microsoft.windowssoundrecorder" },
    Item { group: "Systems & Misc", label: "Wallet",            pattern: "wallet|microsoft.wallet" },
    Item { group: "Systems & Misc", label: "Feedback Hub",      pattern: "windowsfeedbackhub|feedbackhub|microsoft.windowsfeedbackhub" },
    Item { group: "Systems & Misc", label: "Get Started",       pattern: "getstarted|microsoft.getstarted" },
    Item { group: "Systems & Misc", label: "Snipping Tool",     pattern: "snippingtool|microsoft.snippingtool" },
    Item { group: "Systems & Misc", label: "Quick Assist",      pattern: "quickassist|microsoft.quickassist" },
    Item { group: "Systems & Misc", label: "Windows Security",  pattern: "windowssecurity|securityapp|microsoft.windowssecurity" },
    Item { group: "Systems & Misc", label: "Windows Web Experience Pack", pattern: "windowswebexperiencepack|webexperience" },

    // Store / Extensions / Codec & image extensions
    Item { group: "Store & System", label: "Microsoft Store",  pattern: "windowsstore|microsoft.windowsstore|store" },
    Item { group: "Store & System", label: "App Installer",    pattern: "appinstaller|microsoft.appinstaller" },
    Item { group: "Store & System", label: "HEIF Image Extensions", pattern: "heif|heifimageextension" },
    Item { group: "Store & System", label: "HEVC Video Extensions", pattern: "hevc|hevcvideoextension" },
    Item { group: "Store & System", label: "AV1/VP9/Web media extensions", pattern: "av1|vp9|webmedia|webp|webpimageextension" },

    // Misc / legacy or optional
    Item { group: "Other", label: "Paint 3D (if present)", pattern: "mixedreality|mixedrealityportal|microsoft.mixedreality.portal|paint3d" },
    Item { group: "Other", label: "Office-related (Hub/OneNote/ToDo)", pattern: "office|onenote|microsoft.office.onenote|microsoft.office.sway|microsoft.todos" },
];

const COL0_GROUPS: &[&str] = &["Communication", "Media & Creativity", "Microsoft Apps", "Other"];
const COL1_GROUPS: &[&str] = &["Bing Apps", "Games", "Systems & Misc", "Store & System"];

// -------------------- Persistent states --------------------
static SELECTION: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();
static INSTALLED: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();

static CONFIRM_REMOVE_ALL: OnceLock<Mutex<bool>> = OnceLock::new();
static CONFIRM_FORCE_REMOVE_ALL: OnceLock<Mutex<bool>> = OnceLock::new();

// use std::time::Duration; // adaugă dacă nu există deja

// Refresh state (blocking the UI buttons while scanning)
static REFRESHING: OnceLock<Mutex<bool>> = OnceLock::new();
static REFRESH_PROGRESS: OnceLock<Mutex<RefreshProgress>> = OnceLock::new();

#[derive(Clone, Default)]
struct RefreshProgress {
    running: bool,
    current: usize,
    total: usize,
}

fn refreshing_state() -> &'static Mutex<bool> {
    REFRESHING.get_or_init(|| Mutex::new(false))
}
fn refresh_progress_state() -> &'static Mutex<RefreshProgress> {
    REFRESH_PROGRESS.get_or_init(|| Mutex::new(RefreshProgress::default()))
}

fn confirm_remove_all_state() -> &'static Mutex<bool> {
    CONFIRM_REMOVE_ALL.get_or_init(|| Mutex::new(false))
}
fn confirm_force_remove_all_state() -> &'static Mutex<bool> {
    CONFIRM_FORCE_REMOVE_ALL.get_or_init(|| Mutex::new(false))
}

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
fn installed_state() -> &'static Mutex<Vec<bool>> {
    INSTALLED.get_or_init(|| Mutex::new(vec![false; ITEMS.len()]))
}
fn progress_state() -> &'static Mutex<Progress> {
    PROGRESS.get_or_init(|| Mutex::new(Progress::default()))
}

// -------------------- Helpers --------------------

fn append_line(log: &Arc<Mutex<String>>, line: impl AsRef<str>) {
    let line = line.as_ref();
    if let Ok(mut lg) = log.lock() {
        lg.push_str(line);
        if !line.ends_with('\n') {
            lg.push('\n');
        }
    }
}

/// Detectează aplicaţia folosind mai multe canale (Appx packages, provisioned, StartApps, winget, registry).
/// Returnează (found, diagnostic_output) — diagnostic_output e complet și se afișează în log.
// fn detect_app(pattern: &str) -> (bool, String) {
//     let safe = pattern.replace('\'', "").replace('"', "");
//     let pat = format!("*{}*", safe);
//     let mut diag = String::new();
//     let run = |cmd: &str| -> String { utils::run_powershell_cmd(cmd) };


//     diag.push_str(&format!("--- Detect pattern: '{}' ---\n", pattern));

//     // 1) Get-AppxPackage (CurrentUser)
//     diag.push_str("=== Get-AppxPackage (CurrentUser) ===\n");
//     let ps1 = format!(
//         r#"Get-AppxPackage | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}') }} | Select-Object Name,PackageFullName,PackageFamilyName | Format-List -Force"#,
//         pat
//     );
//     let out1 = run(&ps1);
//     diag.push_str(&out1);
//     diag.push('\n');

//     // 2) Get-AppxPackage -AllUsers
//     diag.push_str("=== Get-AppxPackage -AllUsers ===\n");
//     let ps2 = format!(
//         r#"Get-AppxPackage -AllUsers | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}') }} | Select-Object Name,PackageFullName,PackageFamilyName | Format-List -Force"#,
//         pat
//     );
//     let out2 = run(&ps2);
//     diag.push_str(&out2);
//     diag.push('\n');

//     // 3) Get-AppxProvisionedPackage -Online
//     diag.push_str("=== Get-AppxProvisionedPackage -Online ===\n");
//     let ps3 = format!(
//         r#"Get-AppxProvisionedPackage -Online | Where-Object {{ ($_.DisplayName -like '{0}') -or ($_.PackageName -like '{0}') }} | Select-Object DisplayName,PackageName | Format-List -Force"#,
//         pat
//     );
//     let out3 = run(&ps3);
//     diag.push_str(&out3);
//     diag.push('\n');

//     // 4) Get-StartApps
//     diag.push_str("=== Get-StartApps (Start index) ===\n");
//     let ps4 = format!(
//         r#"Get-StartApps | Where-Object {{ $_.AppID -like '{0}' -or $_.Name -like '{0}' }} | Format-List -Force"#,
//         pat
//     );
//     let out4 = run(&ps4);
//     diag.push_str(&out4);
//     diag.push('\n');

//     // 5) winget list (fallback)
//     diag.push_str("=== winget list (if available) ===\n");
//     let ps_winget = format!(r#"try {{ winget list --source winget | Where-Object {{ $_ -match '{0}' }} | Out-String }} catch {{ 'winget not available or failed' }}"#, safe);
//     let out5 = run(&ps_winget);
//     diag.push_str(&out5);
//     diag.push('\n');

//     // 6) Registry uninstall keys (HKLM/HKCU)
//     diag.push_str("=== Registry: Uninstall keys (HKLM/HKCU) ===\n");
//     let ps_reg = format!(r#"
//         $pat='{0}';
//         $keys=@(
//         'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall',
//         'HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall',
//         'HKCU:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall'
//         );
//         $result = @();
//         foreach ($k in $keys) {{
//         try {{
//             Get-ChildItem $k -ErrorAction SilentlyContinue | ForEach-Object {{
//             $p = Get-ItemProperty -Path ($_.PSPath) -ErrorAction SilentlyContinue;
//             if ($p -and $p.DisplayName -and ($p.DisplayName -like ""*$pat*"")) {{
//                 $obj = [PSCustomObject]@{{ Key=$k; DisplayName=$p.DisplayName; UninstallString=$p.UninstallString }};
//                 $result += $obj;
//             }}
//             }}
//         }} catch {{ }}
//         }}
//         if ($result.Count -gt 0) {{ $result | Format-List -Force | Out-String }} else {{ Write-Output '' }}
//         "#, safe);
//     let out6 = run(&ps_reg);
//     diag.push_str(&out6);
//     diag.push('\n');

//     let found = [out1, out2, out3, out4, out5, out6].iter().any(|s| !s.trim().is_empty());
//     (found, diag)
// }

// Replace the old detect_app with this version.
// Uses utils::run_powershell_cmd(cmd) to execute commands.
fn detect_app(pattern: &str) -> (bool, String) {
    let safe = pattern.replace('\'', "").replace('"', "");
    let pat = format!("*{}*", safe);
    let mut diag = String::new();
    let run = |cmd: &str| -> String { utils::run_powershell_cmd(cmd) };

    // helper to run a command and append only if it produced content
    let mut any_found = false;
    let mut run_and_maybe_append = |title: &str, cmd: String| {
        let out = run(&cmd);
        if !out.trim().is_empty() {
            any_found = true;
            diag.push_str(&format!("=== {} ===\n", title));
            diag.push_str(out.trim_end());
            diag.push_str("\n\n");
        }
    };

    // 1) Get-AppxPackage (CurrentUser)
    let ps1 = format!(
        r#"Get-AppxPackage | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}') }} | Select-Object Name,PackageFullName,PackageFamilyName | Out-String"#,
        pat
    );
    run_and_maybe_append("Get-AppxPackage (CurrentUser)", ps1);

    // 2) Get-AppxPackage -AllUsers
    let ps2 = format!(
        r#"Get-AppxPackage -AllUsers | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}') }} | Select-Object Name,PackageFullName,PackageFamilyName | Out-String"#,
        pat
    );
    run_and_maybe_append("Get-AppxPackage -AllUsers", ps2);

    // 3) Get-AppxProvisionedPackage -Online
    let ps3 = format!(
        r#"Get-AppxProvisionedPackage -Online | Where-Object {{ ($_.DisplayName -like '{0}') -or ($_.PackageName -like '{0}') }} | Select-Object DisplayName,PackageName | Out-String"#,
        pat
    );
    run_and_maybe_append("Get-AppxProvisionedPackage -Online", ps3);

    // 4) Get-StartApps
    let ps4 = format!(
        r#"Get-StartApps | Where-Object {{ $_.AppID -like '{0}' -or $_.Name -like '{0}' }} | Out-String"#,
        pat
    );
    run_and_maybe_append("Get-StartApps (Start index)", ps4);

    // 5) winget list (fallback)
    let ps_winget = format!(
        r#"try {{ winget list --source winget | Where-Object {{ $_ -match '{0}' }} | Out-String }} catch {{ '' }}"#,
        safe
    );
    run_and_maybe_append("winget list (if available)", ps_winget);

    // 6) Registry uninstall keys
    let ps_reg = format!(r#"
        $pat = '{0}';
        $keys = @(
        'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall',
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall',
        'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall'
        );
        $result = @();
        foreach ($k in $keys) {{
        try {{
            Get-ChildItem $k -ErrorAction SilentlyContinue | ForEach-Object {{
            $p = Get-ItemProperty -Path ($_.PSPath) -ErrorAction SilentlyContinue;
            if ($p -and $p.DisplayName -and ($p.DisplayName -like ""*$pat*"")) {{
                $obj = [PSCustomObject]@{{ Key=$k; DisplayName=$p.DisplayName; UninstallString=$p.UninstallString }};
                $result += $obj;
            }}
            }}
        }} catch {{ }}
        }}
        if ($result.Count -gt 0) {{ $result | Format-List | Out-String }} else {{ Write-Output '' }}
        "#, safe);
    run_and_maybe_append("Registry: Uninstall keys (HKLM/HKCU)", ps_reg);

    (any_found, diag)
}


// Refresh installed vector (runs in background) — updates installed_state()
// Refresh installed vector (runs in background) — updates installed_state()
fn refresh_installed_states(log: Arc<Mutex<String>>) {
    // set refreshing = true
    {
        let mut r = refreshing_state().lock().unwrap();
        *r = true;
    }
    {
        let mut rp = refresh_progress_state().lock().unwrap();
        rp.running = true;
        rp.current = 0;
        rp.total = ITEMS.len();
    }

    let log_clone = log.clone();
    thread::spawn(move || {
        append_line(&log_clone, "🔎 Refresh: scanning installed status for all items...");
        let mut new_states = vec![false; ITEMS.len()];
        for (i, it) in ITEMS.iter().enumerate() {
            // update progress
            {
                let mut rp = refresh_progress_state().lock().unwrap();
                rp.current = i + 1;
            }

            // detect
            let (found, diag) = detect_app(it.pattern);
            append_line(&log_clone, format!("{} -> {}", it.label, if found { "installed" } else { "not found" }));
            if !diag.trim().is_empty() {
                append_line(&log_clone, diag);
            }

            if found {
                new_states[i] = true;
            }

            // small pause to avoid hammering + allow UI repaint cadence
            thread::sleep(Duration::from_millis(120));
        }

        // store states
        {
            let mut st = installed_state().lock().unwrap();
            *st = new_states;
        }
        append_line(&log_clone, "✅ Installed status refresh finished.");

        // clear progress + refreshing
        {
            let mut rp = refresh_progress_state().lock().unwrap();
            rp.running = false;
            rp.current = rp.total;
        }
        {
            let mut r = refreshing_state().lock().unwrap();
            *r = false;
        }
    });
}

// -------------------- UI --------------------

pub fn show_winapp_removal(
    ui: &mut egui::Ui,
    log: &Arc<Mutex<String>>,
    _show_popup: &mut bool,
    _popup_message: &mut String,
) {
    // ensure states lengths
    {
        let mut sel = selection_state().lock().unwrap();
        if sel.len() != ITEMS.len() {
            sel.resize(ITEMS.len(), false);
        }
        let mut inst = installed_state().lock().unwrap();
        if inst.len() != ITEMS.len() {
            inst.resize(ITEMS.len(), false);
        }
    }

    ui.heading("🗑 WinApp Removal");
    ui.add_space(6.0);

    // elevation hint + relaunch
    {
        let elevated = utils::is_elevated();
        if !elevated {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "⚠ Not running as Administrator. Some removals require elevation.");
                if ui.button("Relaunch as Administrator").clicked() {
                    match utils::relaunch_as_admin() {
                        Ok(_) => std::process::exit(0),
                        Err(e) => append_line(log, format!("❌ Failed to relaunch as admin: {}", e)),
                    }
                }
                // ui.add_space(6.0);
                // if ui.button("Refresh installed status").clicked() {
                //     refresh_installed_states(log.clone());
                // }
                ui.add_space(6.0);
                if ui.button("Continue without elevation").clicked() {
                    append_line(log, "ℹ Continuing without elevation (may fail for some packages).");
                }
            });
            ui.add_space(6.0);
        } else {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✅ Running as Administrator").color(egui::Color32::from_rgb(0, 255, 140)));
                // if ui.button("Refresh installed status").clicked() {
                //     refresh_installed_states(log.clone());
                // }
            });
            ui.add_space(6.0);
        }
    }

    // top progress
    {
        let p = progress_state().lock().unwrap().clone();
        if p.running {
            ui.horizontal(|ui| {
                ui.label(format!("{} ({}/{})", p.label, p.current, p.total));
                let frac = if p.total == 0 { 0.0 } else { p.current as f32 / p.total as f32 };
                ui.add(egui::ProgressBar::new(frac).show_percentage());
            });
            ui.add_space(6.0);
        }
    }

    // action buttons
    // action buttons: respect disabled state while refreshing
    let is_refreshing = *refreshing_state().lock().unwrap();

    // --- ACTION ROW: Refresh (always active) + other actions (disabled while refreshing) + danger buttons ---
    // --- ACTION ROW: Refresh (always active) + other actions (disabled while refreshing) + danger buttons ---
    // Styling: normal buttons -> neon green text on grey bg; hover -> black text on white bg.
    // Danger buttons remain as before (red -> neon on hover).
    {
        let neon_green = egui::Color32::from_rgb(0, 255, 140);
        let normal_text = egui::Color32::BLACK;
        let _normal_stroke = egui::Color32::from_gray(160);
        let danger_red  = egui::Color32::from_rgb(200, 30, 30);
        let danger_red2 = egui::Color32::from_rgb(220, 10, 10);

        // Helper for regular action buttons (Refresh/Remove SELECTED/Force remove SELECTED)
        let draw_action_btn = |ui: &mut egui::Ui, label: &str| -> egui::Response {
            let min_size = egui::Vec2::new(150.0, 30.0);
            let (rect, resp) = ui.allocate_at_least(min_size, egui::Sense::click());

            // pick the style fills from visuals so it matches the rest of the UI
            let visuals = ui.style().visuals.clone();
            let normal_bg = visuals.widgets.inactive.bg_fill; // grey-like background when idle
            let hover_bg  = egui::Color32::WHITE;              // white on hover as requested

            // normal: neon text on grey bg; hover: black text on white bg
            let bg_fill   = if resp.hovered() { hover_bg } else { normal_bg };
            let stroke_col= if resp.hovered() { normal_text } else { neon_green }; // stroke black on hover, green normally

            ui.painter().rect(
                rect,
                6.0,
                bg_fill,
                egui::Stroke::new(1.5, stroke_col),
                egui::StrokeKind::Middle,
            );

            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let text_col = if resp.hovered() { normal_text } else { neon_green };
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, font_id, text_col);

            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            resp
        };

        // Helper for danger buttons (unchanged appearance: red -> neon on hover)
        let draw_danger_btn = |ui: &mut egui::Ui, label: &str, color: egui::Color32, state: &Mutex<bool>| -> egui::Response {
            let min_size = egui::Vec2::new(160.0, 30.0);
            let (rect, resp) = ui.allocate_at_least(min_size, egui::Sense::click());

            let visuals = ui.style().visuals.clone();
            let hover_bg = visuals.widgets.hovered.bg_fill;
            let bg_fill = if resp.hovered() { hover_bg } else { egui::Color32::WHITE };

            let stroke_col = if resp.hovered() { neon_green } else { color };
            ui.painter().rect(
                rect,
                6.0,
                bg_fill,
                egui::Stroke::new(2.0, stroke_col),
                egui::StrokeKind::Middle,
            );

            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let text_col = if resp.hovered() { neon_green } else { color };
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, font_id, text_col);

            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            if resp.clicked() {
                *state.lock().unwrap() = true;
            }

            resp
        };

        // Hold responses to handle actions after UI borrow ends
        let mut resp_refresh_opt: Option<egui::Response> = None;
        let mut resp_remove_opt: Option<egui::Response> = None;
        let mut resp_force_opt: Option<egui::Response> = None;

        ui.horizontal(|ui| {
            // Refresh button (always active)
            resp_refresh_opt = Some(draw_action_btn(ui, "Refresh installed status"));
            ui.add_space(6.0);

            // The other action buttons are disabled while refreshing
            ui.add_enabled_ui(!is_refreshing, |ui| {
                resp_remove_opt = Some(draw_action_btn(ui, "Remove SELECTED"));
                ui.add_space(6.0);
                resp_force_opt = Some(draw_action_btn(ui, "Force remove SELECTED"));
                ui.add_space(6.0);

                // Danger buttons inline (unchanged behavior)
                draw_danger_btn(ui, "Remove ALL listed apps", danger_red, confirm_remove_all_state());
                ui.add_space(8.0);
                draw_danger_btn(ui, "Force remove ALL", danger_red2, confirm_force_remove_all_state());
            });
        });

        // Actions handling after drawing
        if let Some(resp_refresh) = resp_refresh_opt {
            if resp_refresh.clicked() {
                if !*refreshing_state().lock().unwrap() {
                    refresh_installed_states(log.clone());
                }
            }
        }
        if let Some(resp_remove) = resp_remove_opt {
            if resp_remove.clicked() {
                spawn_selected(false, log.clone());
            }
        }
        if let Some(resp_force) = resp_force_opt {
            if resp_force.clicked() {
                spawn_selected(true, log.clone());
            }
        }
    }

    // show refresh progress inline (sub butoane)
    {
        let rp = refresh_progress_state().lock().unwrap().clone();
        if rp.running {
            let frac = if rp.total == 0 { 0.0 } else { rp.current as f32 / rp.total as f32 };
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(format!("Scanning installed apps: {}/{}", rp.current, rp.total));
                ui.add(egui::ProgressBar::new(frac).show_percentage());
            });
            // while refreshing, ensure the UI continues to repaint frequently
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }


    // confirmation dialogs for bulk operations
    // Confirm Remove ALL
    if *confirm_remove_all_state().lock().unwrap() {
        egui::Window::new("Confirm: Remove ALL listed apps")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |uiw| {
                uiw.vertical_centered(|uiw| {
                    uiw.add_space(4.0);
                    uiw.label(egui::RichText::new(
                        "Warning — this will attempt to remove EVERY app from the listed items.\nThis can break system features and may require manual recovery.",
                    ).color(egui::Color32::from_rgb(255, 160, 0)));
                    uiw.add_space(6.0);
                    uiw.horizontal(|uiw| {
                        if uiw.button("Confirm — Remove ALL").clicked() {
                            // start operation and close dialog
                            spawn_bulk(false, log.clone());
                            *confirm_remove_all_state().lock().unwrap() = false;
                        }
                        if uiw.button("Cancel").clicked() {
                            *confirm_remove_all_state().lock().unwrap() = false;
                        }
                    });
                    uiw.add_space(4.0);
                });
            });
    }

    // Confirm Force remove ALL
    if *confirm_force_remove_all_state().lock().unwrap() {
        egui::Window::new("Confirm: Force remove ALL")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |uiw| {
                uiw.vertical_centered(|uiw| {
                    uiw.add_space(4.0);
                    uiw.label(egui::RichText::new(
                        "DANGEROUS: Force removal will attempt aggressive uninstall steps.\nThis can remove system-provisioned packages and may leave Windows in an unstable state.",
                    ).color(egui::Color32::from_rgb(220, 40, 40)));
                    uiw.add_space(6.0);
                    uiw.horizontal(|uiw| {
                        if uiw.button("Confirm — Force Remove ALL").clicked() {
                            spawn_bulk(true, log.clone());
                            *confirm_force_remove_all_state().lock().unwrap() = false;
                        }
                        if uiw.button("Cancel").clicked() {
                            *confirm_force_remove_all_state().lock().unwrap() = false;
                        }
                    });
                    uiw.add_space(4.0);
                });
            });
    }


    ui.add_space(8.0);

    // scrollable columns
    let avail_h = ui.available_height();
    egui::ScrollArea::vertical()
        .id_salt("winapp_scroll_v4")
        .auto_shrink([false; 2])
        .max_height(avail_h)
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                render_column(&mut cols[0], COL0_GROUPS, log);
                render_column(&mut cols[1], COL1_GROUPS, log);
            });
        });

    // force UI to repaint so updates from background threads appear immediately
    ui.ctx().request_repaint();

}

fn render_column(ui: &mut egui::Ui, groups: &[&str], _log: &Arc<Mutex<String>>) {
    for &g in groups {
        ui.group(|ui| {
            ui.label(egui::RichText::new(g).size(16.0));
            let mut sel = selection_state().lock().unwrap();
            let inst = installed_state().lock().unwrap();
            for (i, it) in ITEMS.iter().enumerate().filter(|(_, it)| it.group == g) {
                let mut v = sel[i];
                // create label with installed hint if available
                let mut label = it.label.to_string();
                if i < inst.len() && inst[i] {
                    label.push_str(" ");
                    // append small green installed marker
                    label.push_str("(installed)");
                }
                // draw checkbox with rich text display if installed
                if inst.get(i).copied().unwrap_or(false) {
                    if ui.checkbox(&mut v, egui::RichText::new(label).color(egui::Color32::from_rgb(0, 255, 140))).clicked() {
                        sel[i] = v;
                    }
                } else {
                    if ui.checkbox(&mut v, label).clicked() {
                        sel[i] = v;
                    }
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

            append_line(&log, format!("→ Processing: {}", it.label));

            // detect first
            let (found, diag) = detect_app(it.pattern);
            append_line(&log, diag);
            if !found {
                if force {
                    append_line(&log, format!("⚠ {}: Not detected but force=true -> attempting removal anyway.", it.label));
                } else {
                    append_line(&log, format!("ℹ {}: Application not found. Skipping.", it.label));
                    continue;
                }
            } else {
                append_line(&log, format!("ℹ {}: Detected installed. Proceeding with removal.", it.label));
            }

            // call removal (commands::remove_app returns multiline log)
            let res = commands::remove_app(it.pattern);
            for line in res.lines() {
                append_line(&log, line);
            }

            // short pause
            thread::sleep(Duration::from_millis(400));
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

            append_line(&log, format!("→ Processing: {}", it.label));

            // detect
            let (found, diag) = detect_app(it.pattern);
            append_line(&log, diag);
            if !found {
                if force {
                    append_line(&log, format!("⚠ {}: Not detected but force=true -> attempting removal anyway.", it.label));
                } else {
                    append_line(&log, format!("ℹ {}: Application not found. Skipping.", it.label));
                    continue;
                }
            } else {
                append_line(&log, format!("ℹ {}: Detected installed. Proceeding with removal.", it.label));
            }

            // remove
            let res = commands::remove_app(it.pattern);
            for line in res.lines() {
                append_line(&log, line);
            }

            // uncheck item after attempt
            {
                let mut s = selection_state().lock().unwrap();
                if *idx < s.len() {
                    s[*idx] = false;
                }
            }

            thread::sleep(Duration::from_millis(300));
        }

        // finalize progress
        {
            let mut p = progress_state().lock().unwrap();
            p.current = total;
            p.label.clear();
            p.running = false;
        }

        append_line(&log, if force { "✅ Force removal (selected) finished." } else { "✅ Removal (selected) finished." });
    });
}
