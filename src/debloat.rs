//! Permanent removal of bundled Windows apps.
//!
//! Uninstalling an Appx package is not enough: Windows feature updates re-provision every app
//! that is still provisioned. For each app we therefore
//! 1. add its package family to `AppxAllUserStore\Deprovisioned` (Windows skips those on update),
//! 2. remove the provisioned package (new user profiles don't get it),
//! 3. uninstall it for all existing users.
//! Packages are matched by their exact name; never by wildcard.

use serde::Deserialize;
use std::process::Command;

const DEPROVISIONED_ROOT: &str = r"HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\Deprovisioned";
const MICROSOFT_PUBLISHER_ID: &str = "8wekyb3d8bbwe";

#[derive(Clone, Copy)]
pub struct WinApp {
    pub group: &'static str,
    pub label: &'static str,
    pub package: &'static str,
    /// Package family name when the publisher is not Microsoft's 8wekyb3d8bbwe.
    pub family: Option<&'static str>,
    /// Part of the "rarely used apps" preset.
    pub recommended: bool,
}

const fn app(group: &'static str, label: &'static str, package: &'static str, recommended: bool) -> WinApp {
    WinApp { group, label, package, family: None, recommended }
}

const fn app_family(group: &'static str, label: &'static str, package: &'static str, family: &'static str, recommended: bool) -> WinApp {
    WinApp { group, label, package, family: Some(family), recommended }
}

pub const APPS: &[WinApp] = &[
    app("Communication", "Outlook (new)", "Microsoft.OutlookForWindows", true),
    app("Communication", "Mail & Calendar", "microsoft.windowscommunicationsapps", true),
    app("Communication", "Microsoft Teams (personal)", "MSTeams", true),
    app("Communication", "Teams Chat (legacy)", "MicrosoftTeams", true),
    app("Communication", "Skype", "Microsoft.SkypeApp", true),
    app("Communication", "People", "Microsoft.People", true),
    app("Communication", "GroupMe", "Microsoft.GroupMe10", true),
    app("Communication", "Messaging", "Microsoft.Messaging", true),
    app("Communication", "Phone Link", "Microsoft.YourPhone", false),
    app("AI & Bing", "Microsoft Copilot", "Microsoft.Copilot", true),
    app("AI & Bing", "Bing Search", "Microsoft.BingSearch", true),
    app("AI & Bing", "Microsoft News", "Microsoft.BingNews", true),
    app("AI & Bing", "Weather", "Microsoft.BingWeather", true),
    app("AI & Bing", "Money", "Microsoft.BingFinance", true),
    app("AI & Bing", "Sports", "Microsoft.BingSports", true),
    app("AI & Bing", "Cortana", "Microsoft.549981C3F5F10", true),
    app("Productivity", "Microsoft 365 (Office) hub", "Microsoft.MicrosoftOfficeHub", true),
    app("Productivity", "OneNote (Store)", "Microsoft.Office.OneNote", false),
    app("Productivity", "Sway", "Microsoft.Office.Sway", true),
    app("Productivity", "Microsoft To Do", "Microsoft.Todos", true),
    app("Productivity", "Journal", "Microsoft.MicrosoftJournal", true),
    app("Productivity", "Power Automate", "Microsoft.PowerAutomateDesktop", true),
    app("Productivity", "Dev Home", "Microsoft.Windows.DevHome", true),
    app("Productivity", "Sticky Notes", "Microsoft.MicrosoftStickyNotes", false),
    app_family("Media & Creativity", "Clipchamp", "Clipchamp.Clipchamp", "Clipchamp.Clipchamp_yxz26nhyzhsrt", true),
    app("Media & Creativity", "Films & TV", "Microsoft.ZuneVideo", true),
    app("Media & Creativity", "Media Player", "Microsoft.ZuneMusic", false),
    app("Media & Creativity", "Camera", "Microsoft.WindowsCamera", false),
    app("Media & Creativity", "Paint", "Microsoft.Paint", false),
    app("Media & Creativity", "Sound Recorder", "Microsoft.WindowsSoundRecorder", false),
    app("Media & Creativity", "3D Builder", "Microsoft.3DBuilder", true),
    app("Media & Creativity", "3D Viewer", "Microsoft.Microsoft3DViewer", true),
    app("Media & Creativity", "Print 3D", "Microsoft.Print3D", true),
    app("Media & Creativity", "Mixed Reality Portal", "Microsoft.MixedReality.Portal", true),
    app("Games & Xbox", "Solitaire Collection", "Microsoft.MicrosoftSolitaireCollection", true),
    app("Games & Xbox", "Edge Game Assist", "Microsoft.Edge.GameAssist", true),
    app("Games & Xbox", "Xbox app", "Microsoft.GamingApp", false),
    app("Games & Xbox", "Xbox Game Bar", "Microsoft.XboxGamingOverlay", false),
    app("Games & Xbox", "Xbox speech-to-text overlay", "Microsoft.XboxSpeechToTextOverlay", false),
    app("Games & Xbox", "Minecraft for Windows", "Microsoft.MinecraftUWP", false),
    app_family("Promoted apps", "Spotify", "SpotifyAB.SpotifyMusic", "SpotifyAB.SpotifyMusic_zpdnekdrzrea0", true),
    app_family("Promoted apps", "LinkedIn", "7EE7776C.LinkedInforWindows", "7EE7776C.LinkedInforWindows_w1wdnht996qgy", true),
    app_family("Promoted apps", "Disney+", "Disney.37853FC22B2CE", "Disney.37853FC22B2CE_6rarf9sa4v8jt", true),
    app("System & Misc", "Tips", "Microsoft.Getstarted", true),
    app("System & Misc", "Get Help", "Microsoft.GetHelp", true),
    app("System & Misc", "Feedback Hub", "Microsoft.WindowsFeedbackHub", true),
    app("System & Misc", "Maps", "Microsoft.WindowsMaps", true),
    app("System & Misc", "Family Safety", "MicrosoftCorporationII.MicrosoftFamily", true),
    app("System & Misc", "Wallet", "Microsoft.Wallet", true),
    app("System & Misc", "Mobile Plans", "Microsoft.OneConnect", true),
    app("System & Misc", "Clock", "Microsoft.WindowsAlarms", false),
    app("System & Misc", "Quick Assist", "MicrosoftCorporationII.QuickAssist", false),
    app_family("System & Misc", "Widgets", "MicrosoftWindows.Client.WebExperience", "MicrosoftWindows.Client.WebExperience_cw5n1h2txyewy", false),
];

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    Installed,
    ProvisionedOnly,
    Blocked,
    #[default]
    Absent,
}

#[derive(Clone, Default)]
pub struct AppStatus {
    pub state: AppState,
    /// Also listed in the Deprovisioned list (it may still be installed).
    pub blocked: bool,
    pub family: String,
}

#[derive(Clone, Default)]
pub struct Status {
    pub apps: Vec<AppStatus>,
    pub deprovisioned: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PsPackage {
    name: Option<String>,
    family_name: Option<String>,
    installed: Option<bool>,
    provisioned: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PsStatus {
    packages: Option<Vec<PsPackage>>,
    deprovisioned: Option<Vec<String>>,
}

fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn is_valid_token(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

pub fn default_family(app: &WinApp) -> String {
    app.family.map(str::to_string).unwrap_or_else(|| format!("{}_{}", app.package, MICROSOFT_PUBLISHER_ID))
}

fn powershell(script: &str) -> Result<String, String> {
    let mut command = Command::new(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output().map_err(|error| format!("Failed to start PowerShell: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

/// Reads installed and provisioned packages plus the Deprovisioned list in one PowerShell call.
pub fn read_status() -> Result<Status, String> {
    let script = format!("$root = {}\n", ps_quote(DEPROVISIONED_ROOT))
        + r#"
$ErrorActionPreference = 'SilentlyContinue'
$packages = @{}
foreach ($p in @(Get-AppxPackage -AllUsers)) {
  if (-not $packages.ContainsKey($p.Name)) {
    $packages[$p.Name] = [PSCustomObject]@{ Name = $p.Name; FamilyName = $p.PackageFamilyName; Installed = $true; Provisioned = $false }
  }
}
foreach ($p in @(Get-AppxProvisionedPackage -Online)) {
  if ($packages.ContainsKey($p.DisplayName)) { $packages[$p.DisplayName].Provisioned = $true }
  else { $packages[$p.DisplayName] = [PSCustomObject]@{ Name = $p.DisplayName; FamilyName = $p.DisplayName + '_' + (($p.PackageName -split '__')[-1]); Installed = $false; Provisioned = $true } }
}
$deprovisioned = @(Get-ChildItem -LiteralPath $root | ForEach-Object { $_.PSChildName })
ConvertTo-Json -InputObject ([PSCustomObject]@{ Packages = @($packages.Values); Deprovisioned = $deprovisioned }) -Depth 4 -Compress
"#;
    let json = powershell(&script)?;
    let parsed: PsStatus = serde_json::from_str(json.trim()).map_err(|error| format!("Failed to parse Windows apps status: {error}"))?;
    let packages = parsed.packages.unwrap_or_default();
    let deprovisioned = parsed.deprovisioned.unwrap_or_default();
    let blocked: std::collections::HashSet<String> = deprovisioned.iter().map(|name| name.to_lowercase()).collect();

    let apps = APPS
        .iter()
        .map(|app| {
            let pkg = packages.iter().find(|p| p.name.as_deref().is_some_and(|name| name.eq_ignore_ascii_case(app.package)));
            let family = pkg
                .and_then(|p| p.family_name.clone())
                .filter(|family| !family.is_empty())
                .unwrap_or_else(|| default_family(app));
            let is_blocked = blocked.contains(&family.to_lowercase());
            let state = match pkg {
                Some(p) if p.installed.unwrap_or(false) => AppState::Installed,
                Some(p) if p.provisioned.unwrap_or(false) => AppState::ProvisionedOnly,
                _ if is_blocked => AppState::Blocked,
                _ => AppState::Absent,
            };
            AppStatus { state, blocked: is_blocked, family }
        })
        .collect();

    Ok(Status { apps, deprovisioned })
}

/// Adds each app to the Deprovisioned list, removes its provisioned package and uninstalls it
/// for all users. `targets` are (package name, package family name) pairs.
pub fn deprovision_and_remove(targets: &[(String, String)]) -> Result<String, String> {
    if targets.is_empty() {
        return Err("No apps selected.".into());
    }
    if !crate::utils::is_elevated() {
        return Err("Administrator privileges are required. Relaunch Eoliann Windows Tools as Administrator.".into());
    }
    let mut entries = Vec::new();
    for (name, family) in targets {
        if !is_valid_token(name) || (!family.is_empty() && !is_valid_token(family)) {
            return Err(format!("Invalid package name: {name}"));
        }
        entries.push(format!("[PSCustomObject]@{{ Name = {}; Family = {} }}", ps_quote(name), ps_quote(family)));
    }

    let mut script = format!("$targets = @({})\n$root = {}\n", entries.join(", "), ps_quote(DEPROVISIONED_ROOT));
    script.push_str(r#"
$ErrorActionPreference = 'Continue'
$log = New-Object System.Collections.Generic.List[string]
$failed = 0
if (-not (Test-Path -LiteralPath $root)) { New-Item -Path $root -Force | Out-Null }
$installed = @(Get-AppxPackage -AllUsers -ErrorAction SilentlyContinue)
$provisioned = @(Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue)

foreach ($target in $targets) {
  $name = $target.Name
  $pkgs = @($installed | Where-Object { $_.Name -eq $name })
  $provs = @($provisioned | Where-Object { $_.DisplayName -eq $name })

  $families = @($pkgs | ForEach-Object { $_.PackageFamilyName }) + @($provs | ForEach-Object { $_.DisplayName + '_' + (($_.PackageName -split '__')[-1]) })
  if ($target.Family) { $families += $target.Family }
  $families = @($families | Where-Object { $_ } | Sort-Object -Unique)

  foreach ($family in $families) {
    $familyKey = Join-Path $root $family
    if (-not (Test-Path -LiteralPath $familyKey)) { New-Item -Path $familyKey -Force | Out-Null }
    if (Test-Path -LiteralPath $familyKey) { $log.Add("[$name] Added to Deprovisioned list: $family") | Out-Null }
    else { $failed++; $log.Add("[$name] FAILED to add to Deprovisioned list: $family") | Out-Null }
  }

  foreach ($prov in $provs) {
    try {
      Remove-AppxProvisionedPackage -Online -AllUsers -PackageName $prov.PackageName -ErrorAction Stop | Out-Null
      $log.Add("[$name] Removed provisioned package: $($prov.PackageName)") | Out-Null
    } catch {
      try {
        Remove-AppxProvisionedPackage -Online -PackageName $prov.PackageName -ErrorAction Stop | Out-Null
        $log.Add("[$name] Removed provisioned package: $($prov.PackageName)") | Out-Null
      } catch {
        $failed++
        $log.Add("[$name] FAILED to remove provisioned package $($prov.PackageName): $($_.Exception.Message)") | Out-Null
      }
    }
  }

  foreach ($pkg in $pkgs) {
    try {
      Remove-AppxPackage -Package $pkg.PackageFullName -AllUsers -ErrorAction Stop
      $log.Add("[$name] Uninstalled for all users: $($pkg.PackageFullName)") | Out-Null
    } catch {
      try {
        Remove-AppxPackage -Package $pkg.PackageFullName -ErrorAction Stop
        $log.Add("[$name] Uninstalled for current user: $($pkg.PackageFullName)") | Out-Null
      } catch {
        $failed++
        $log.Add("[$name] FAILED to uninstall $($pkg.PackageFullName): $($_.Exception.Message)") | Out-Null
      }
    }
  }

  if ($pkgs.Count -eq 0 -and $provs.Count -eq 0) {
    $log.Add("[$name] Not installed. Blocked from being reinstalled by Windows updates.") | Out-Null
  }
}

$header = if ($failed -eq 0) { "Debloat completed for $($targets.Count) app(s). Removed apps will not be reinstalled by Windows feature updates." } else { "Debloat finished with $failed error(s)." }
$header + "`n" + ($log -join "`n")
"#);
    powershell(&script)
}

/// Removes a package family from the Deprovisioned list so Windows/Store can install it again.
pub fn allow_reinstall(family: &str) -> Result<String, String> {
    let family = family.trim();
    if !is_valid_token(family) {
        return Err(format!("Invalid package family name: {family}"));
    }
    if !crate::utils::is_elevated() {
        return Err("Administrator privileges are required. Relaunch Eoliann Windows Tools as Administrator.".into());
    }
    let script = format!("$path = Join-Path {} {}\n", ps_quote(DEPROVISIONED_ROOT), ps_quote(family))
        + r#"
if (Test-Path -LiteralPath $path) {
  Remove-Item -LiteralPath $path -Recurse -Force -ErrorAction Stop
  "Removed from Deprovisioned list: $(Split-Path $path -Leaf). Reinstall the app from Microsoft Store if you need it now."
} else {
  "Not in Deprovisioned list: $(Split-Path $path -Leaf)"
}
"#;
    powershell(&script)
}
