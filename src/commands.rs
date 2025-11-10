#![allow(dead_code)]

use crate::utils::run_command; // păstrează utilitarul tău existent

use std::io::{self, BufReader};
use std::process::{Command as StdCommand, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use std::io::BufRead; // Import BufRead aici
// ---------- Helper logging ----------
fn push_line(log: &Arc<Mutex<String>>, line: &str) {
    if let Ok(mut lg) = log.lock() {
        lg.push_str(line);
        lg.push('\n');
    }
}

// ---------- Stream runners (cu citire STDOUT + STDERR în paralel) ----------

/// Rulează un proces, citește stdout+stderr în paralel, scrie în `log` cu prefix și așteaptă să iasă.
fn run_command_stream_and_wait(log: Arc<Mutex<String>>, cmd: &str, args: &[&str]) -> std::io::Result<i32> {
    let prefix = format!("{} {}", cmd, args.join(" "));
 
    #[allow(unused_mut)]
    let mut child = {
        let mut c = StdCommand::new(cmd);
        c.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            c.creation_flags(CREATE_NO_WINDOW);
        }
        c.spawn()?
    };

    // STDOUT
    let out_log = log.clone();
    let out_prefix = prefix.clone();
    let out_handle = if let Some(stdout) = child.stdout.take() {
        Some(thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                let msg = format!("[OUT] [{}] {}", out_prefix, line);
                push_line(&out_log, &msg);
            }
        }))
    } else { None };

    // STDERR
    let err_log = log.clone();
    let err_prefix = prefix.clone();
    let err_handle = if let Some(stderr) = child.stderr.take() {
        Some(thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                let msg = format!("[ERR] [{}] {}", err_prefix, line);
                push_line(&err_log, &msg);
            }
        }))
    } else { None };

    let status = child.wait()?;
    if let Some(h) = out_handle { let _ = h.join(); }
    if let Some(h) = err_handle { let _ = h.join(); }

    Ok(status.code().unwrap_or(-1))
}

// ---------- Funcții de business ----------

#[allow(dead_code)]
pub fn toggle_context_menu() -> String {
    // Verifică existența cheii pentru meniul clasic (Win10)
    let check = StdCommand::new("reg")
        .args(&["query", r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}"])
        .output();

    let mut message = String::new();

    match check {
        Ok(output) if output.status.success() => {
            // cheia există -> revii la meniul Win11
            let result = StdCommand::new("reg")
                .args(&[
                    "delete",
                    r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}",
                    "/f",
                ])
                .output();

            if let Ok(out) = result {
                if out.status.success() {
                    message.push_str("✅ Switched to Windows 11 context menu.\n");
                } else {
                    return String::from_utf8_lossy(&out.stderr).to_string();
                }
            } else if let Err(e) = result {
                return format!("Error: {}", e);
            }
        }
        _ => {
            // cheia nu există -> adaugi pentru meniul clasic (Win10-like)
            let result = StdCommand::new("reg")
                .args(&[
                    "add",
                    r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
                    "/ve", "/t", "REG_SZ", "/d", "", "/f",
                ])
                .output();

            if let Ok(out) = result {
                if out.status.success() {
                    message.push_str("✅ Switched to Classic context menu.\n");
                } else {
                    return String::from_utf8_lossy(&out.stderr).to_string();
                }
            } else if let Err(e) = result {
                return format!("Error: {}", e);
            }
        }
    }

    // Restart Explorer pentru aplicare instantă
    let _ = StdCommand::new("taskkill").args(&["/f", "/im", "explorer.exe"]).output();
    let _ = StdCommand::new("explorer.exe").spawn();

    message.push_str("🔄 Explorer restarted. Changes applied instantly!");
    message
}

pub fn verify_system_integrity_live(log: Arc<Mutex<String>>) {
    thread::spawn(move || {
        {
            let mut l = log.lock().unwrap();
            l.clear();
            l.push_str("🔍 Starting system integrity verification...\n\n");
            l.push_str("Note: For best results, run the app as Administrator.\n\n");
        }

        // Rulare secvențială ca să știi clar progresul și să eviți suprapuneri grele în log
        let _ = run_command_stream_and_wait(log.clone(), "sfc", &["/scannow"]);
        let _ = run_command_stream_and_wait(log.clone(), "DISM", &["/Online", "/Cleanup-Image", "/CheckHealth"]);
        let _ = run_command_stream_and_wait(log.clone(), "DISM", &["/Online", "/Cleanup-Image", "/ScanHealth"]);
        let _ = run_command_stream_and_wait(log.clone(), "DISM", &["/Online", "/Cleanup-Image", "/RestoreHealth"]);

        push_line(&log, "✅ Verification finished.");
    });
}

#[allow(dead_code)]
pub fn disk_cleanup() -> String {
    run_command("cleanmgr.exe /verylowdisk")
}

pub fn clean_temporary_files() -> String {
    let mut cmd = StdCommand::new("powershell");
    cmd.args(&[
        "-Command",
        "Remove-Item -Path $env:TEMP\\* -Recurse -Force -ErrorAction SilentlyContinue; \
         Remove-Item -Path C:\\Windows\\Temp\\* -Recurse -Force -ErrorAction SilentlyContinue; \
         Remove-Item -Path C:\\Windows\\Prefetch\\* -Recurse -Force -ErrorAction SilentlyContinue",
    ]);
    #[cfg(windows)]
    { cmd.creation_flags(CREATE_NO_WINDOW); }

    match cmd.output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if stdout.is_empty() {
                "Temporary files cleaned successfully.".to_string()
            } else {
                format!("Temporary files cleaned with warnings:\n{}", stdout)
            }
        }
        Err(e) => format!("Eroare la curățare: {}", e),
    }
}

#[allow(dead_code)]
pub fn empty_recycle_bin() -> String {
    // Varianta verbose: COM Shell.Application + mesaj pentru fiecare item
    let ps_script = r#"
$bin = (New-Object -ComObject Shell.Application).NameSpace(10)
$bin.Items() | ForEach-Object {
  Write-Output ('Deleting ' + $_.Name + ' from Recycle Bin')
  try { Remove-Item $_.Path -Recurse -Force -ErrorAction SilentlyContinue } catch {}
}
"#;

    let mut cmd = std::process::Command::new("powershell");
    cmd.args([
        "-ExecutionPolicy", "Unrestricted",
        "-NoProfile",
        "-Command", ps_script,
    ]);
    #[cfg(windows)]
    { cmd.creation_flags(CREATE_NO_WINDOW); }

    match cmd.output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            if out.trim().is_empty() {
                "Recycle Bin is already empty or items were in use.".to_string()
            } else {
                format!("Recycle Bin cleanup:\n{}", out)
            }
        }
        Err(e) => format!("Error emptying Recycle Bin: {}", e),
    }
}

#[allow(dead_code)]
pub fn quick_access_settings(section: &str) -> String {
    run_command(&format!("explorer ms-settings:{}", section))
}

#[allow(dead_code)]
pub fn network_reset() -> String {
    let mut output = String::new();
    output.push_str(&run_command("netsh winsock reset"));
    output.push_str(&run_command("ipconfig /flushdns"));
    output.push_str(&run_command("ipconfig /release"));
    output.push_str(&run_command("ipconfig /renew"));
    output.push_str("\nℹ For full effect, a restart might be required.");
    output
}

#[allow(dead_code)]
pub fn power_plan_switcher(guid: &str) -> String {
    run_command(&format!("powercfg /setactive {}", guid))
}

#[allow(dead_code)]
pub fn change_theme(mode: &str) -> String {
    match mode {
        "dark" => run_command(
            "reg add HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize /v AppsUseLightTheme /t REG_DWORD /d 0 /f"
        ),
        "light" => run_command(
            "reg add HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize /v AppsUseLightTheme /t REG_DWORD /d 1 /f"
        ),
        _ => "Invalid mode".to_string(),
    }
}

#[allow(dead_code)]
pub fn disable_sleep() -> String {
    let cmds = [
        "powercfg /change standby-timeout-ac 0",
        "powercfg /change standby-timeout-dc 0",
        "powercfg /change hibernate-timeout-ac 0",
        "powercfg /change hibernate-timeout-dc 0",
    ];

    for cmd in cmds {
        let _ = run_command(cmd);
    }
    "✅ Sleep & Hibernate disabled (AC + DC)".to_string()
}

#[allow(dead_code)]
pub fn disable_hdd() -> String {
    let cmds = [
        "powercfg /change disk-timeout-ac 0",
        "powercfg /change disk-timeout-dc 0",
    ];

    for cmd in cmds {
        let _ = run_command(cmd);
    }
    "✅ HDD/SSD turn off disabled (AC + DC)".to_string()
}

#[allow(dead_code)]
pub fn disable_monitor() -> String {
    let cmds = [
        "powercfg /change monitor-timeout-ac 0",
        "powercfg /change monitor-timeout-dc 0",
    ];

    for cmd in cmds {
        let _ = run_command(cmd);
    }
    "✅ Monitor turn off disabled (AC + DC)".to_string()
}

#[allow(dead_code)]
pub fn remove_app(package: &str) -> String {
    // Robust remove: tries current user, -AllUsers (if supported), remove provisioned + DISM fallback
    let ps = format!(r#"
$needle = "{0}"
Write-Output "=== Remove (current user) matching: $needle ==="
Get-AppxPackage |
  Where-Object {{ $_.Name -like "*{0}*" -or $_.PackageFullName -like "*{0}*" }} |
  ForEach-Object {{
    try {{ Remove-AppxPackage -Package $_.PackageFullName -ErrorAction Stop; Write-Output ("REMOVED_CURRENTUSER:" + $_.PackageFullName) }} catch {{ Write-Output ("ERR_REMOVE_CURRENTUSER:" + $_.PackageFullName + " -> " + $_.Exception.Message) }}
  }}

Write-Output "=== Attempting -AllUsers (requires elevation) matching: $needle ==="
try {{
  Get-AppxPackage -AllUsers |
    Where-Object {{ $_.Name -like "*{0}*" -or $_.PackageFullName -like "*{0}*" }} |
    ForEach-Object {{
      try {{ Remove-AppxPackage -Package $_.PackageFullName -AllUsers -ErrorAction Stop; Write-Output ("REMOVED_ALLUSERS:" + $_.PackageFullName) }} catch {{ Write-Output ("ERR_REMOVE_ALLUSERS:" + $_.PackageFullName + " -> " + $_.Exception.Message) }}
    }}
}} catch {{
  Write-Output ("WARN: -AllUsers failed or not supported: " + $_.Exception.Message)
}}

Write-Output "=== Removing provisioned packages (image) matching: $needle ==="
Get-AppxProvisionedPackage -Online |
  Where-Object {{ $_.DisplayName -like "*{0}*" -or $_.PackageName -like "*{0}*" }} |
  ForEach-Object {{
    try {{ Remove-AppxProvisionedPackage -Online -PackageName $_.PackageName -ErrorAction Stop; Write-Output ("REMOVED_PROVISIONED:" + $_.PackageName) }} catch {{ Write-Output ("ERR_REMOVE_PROVISIONED:" + $_.PackageName + " -> " + $_.Exception.Message) }}
  }}

Write-Output "=== DISM fallback for provisioned packages ==="
$prov = Get-AppxProvisionedPackage -Online |
        Where-Object {{ $_.DisplayName -like "*{0}*" -or $_.PackageName -like "*{0}*" }}
foreach ($p in $prov) {{
  try {{
    $pkg = $p.PackageName
    Write-Output ("DISM_REMOVE:" + $pkg)
    dism.exe /Online /Remove-ProvisionedAppxPackage /PackageName:"$pkg"
  }} catch {{
    Write-Output ("ERR_DISM:" + $p.PackageName + " -> " + $_.Exception.Message)
  }}
}}
Write-Output "=== Done ==="
"#, package);

    crate::utils::run_powershell(&ps)
}

#[allow(dead_code)]
pub fn remove_app_force(package: &str) -> String {
    // Force remove: same as above, with additional attempts by PackageFamilyName
    let ps = format!(r#"
        $needle = "{0}"
        Write-Output "=== FORCE Remove (current user) matching: $needle ==="
        Get-AppxPackage |
        Where-Object {{ $_.Name -like "*{0}*" -or $_.PackageFullName -like "*{0}*" }} |
        ForEach-Object {{
            try {{ Remove-AppxPackage -Package $_.PackageFullName -ErrorAction SilentlyContinue; Write-Output ("REMOVED_CURRENTUSER:" + $_.PackageFullName) }} catch {{ Write-Output ("ERR_REMOVE_CURRENTUSER:" + $_.PackageFullName + " -> " + $_.Exception.Message) }}
        }}

        Write-Output "=== FORCE Attempting -AllUsers (requires elevation) ==="
        try {{
        Get-AppxPackage -AllUsers |
            Where-Object {{ $_.Name -like "*{0}*" -or $_.PackageFullName -like "*{0}*" }} |
            ForEach-Object {{
            try {{ Remove-AppxPackage -Package $_.PackageFullName -AllUsers -ErrorAction SilentlyContinue; Write-Output ("REMOVED_ALLUSERS:" + $_.PackageFullName) }} catch {{ Write-Output ("ERR_REMOVE_ALLUSERS:" + $_.PackageFullName + " -> " + $_.Exception.Message) }}
            }}
        }} catch {{
        Write-Output ("WARN: -AllUsers failed: " + $_.Exception.Message)
        }}

        Write-Output "=== FORCE Removing provisioned packages (image) ==="
        Get-AppxProvisionedPackage -Online |
        Where-Object {{ $_.DisplayName -like "*{0}*" -or $_.PackageName -like "*{0}*" }} |
        ForEach-Object {{
            try {{ Remove-AppxProvisionedPackage -Online -PackageName $_.PackageName -ErrorAction SilentlyContinue; Write-Output ("REMOVED_PROVISIONED:" + $_.PackageName) }} catch {{ Write-Output ("ERR_REMOVE_PROVISIONED:" + $_.PackageName + " -> " + $_.Exception.Message) }}
        }}

        Write-Output "=== FORCE DISM fallback ==="
        $prov = Get-AppxProvisionedPackage -Online |
                Where-Object {{ $_.DisplayName -like "*{0}*" -or $_.PackageName -like "*{0}*" }}
        foreach ($p in $prov) {{
        try {{
            $pkg = $p.PackageName
            Write-Output ("DISM_REMOVE:" + $pkg)
            dism.exe /Online /Remove-ProvisionedAppxPackage /PackageName:"$pkg"
        }} catch {{
            Write-Output ("ERR_DISM:" + $p.PackageName + " -> " + $_.Exception.Message)
        }}
        }}

        Write-Output "=== FORCE: Attempt removal by PackageFamilyName and wildcard ==="
        $family = Get-AppxPackage -AllUsers | Where-Object {{ $_.PackageFamilyName -like "*{0}*" }} | Select-Object -ExpandProperty PackageFullName -Unique
        foreach ($f in $family) {{
        try {{ Remove-AppxPackage -Package $f -AllUsers -ErrorAction SilentlyContinue; Write-Output ("REMOVED_BY_FAMILY:" + $f) }} catch {{ Write-Output ("ERR_REMOVE_BY_FAMILY:" + $f + " -> " + $_.Exception.Message) }}
        }}

        Write-Output "=== FORCE Done ==="
        "#, package);

    crate::utils::run_powershell(&ps)
}

#[allow(dead_code)]
pub fn remove_all_apps(force: bool) -> String {
    use std::time::Instant;

    let apps: Vec<(&str, &str)> = vec![
        ("Microsoft Family Safety", "MicrosoftCorporationII.MicrosoftFamily"),
        ("Outlook for Windows", "Microsoft.OutlookForWindows"),
        ("Clipchamp", "Clipchamp.Clipchamp"),
        ("3D Builder", "Microsoft.3DBuilder"),
        ("3D Viewer", "Microsoft.Microsoft3DViewer"),
        ("Bing Weather", "Microsoft.BingWeather"),
        ("Bing Sports", "Microsoft.BingSports"),
        ("Bing Finance", "Microsoft.BingFinance"),
        ("Office Hub", "Microsoft.MicrosoftOfficeHub"),
        ("Bing News", "Microsoft.BingNews"),
        ("OneNote", "Microsoft.Office.OneNote"),
        ("Sway", "Microsoft.Office.Sway"),
        ("Windows Phone", "Microsoft.WindowsPhone"),
        ("CommsPhone", "Microsoft.CommsPhone"),
        ("Your Phone", "Microsoft.YourPhone"),
        ("Get Started", "Microsoft.Getstarted"),
        ("Xbox App Stub", "Microsoft.549981C3F5F10"),
        ("Messaging", "Microsoft.Messaging"),
        ("Voice Recorder", "Microsoft.WindowsSoundRecorder"),
        ("Mixed Reality Portal", "Microsoft.MixedReality.Portal"),
        ("Feedback Hub", "Microsoft.WindowsFeedbackHub"),
        ("Alarms & Clock", "Microsoft.WindowsAlarms"),
        ("Camera", "Microsoft.WindowsCamera"),
        ("MS Paint", "Microsoft.MSPaint"),
        ("Maps", "Microsoft.WindowsMaps"),
        ("Minecraft for Windows", "Microsoft.MinecraftUWP"),
        ("People", "Microsoft.People"),
        ("Wallet", "Microsoft.Wallet"),
        ("Print 3D", "Microsoft.Print3D"),
        ("OneConnect", "Microsoft.OneConnect"),
        ("Solitaire Collection", "Microsoft.MicrosoftSolitaireCollection"),
        ("Sticky Notes", "Microsoft.MicrosoftStickyNotes"),
        ("Mail & Calendar", "microsoft.windowscommunicationsapps"),
        ("Skype", "Microsoft.SkypeApp"),
        ("GroupMe", "Microsoft.GroupMe10"),
        ("Teams", "MSTeams"),
        ("To-Do", "Microsoft.Todos"),
    ];

    let start = Instant::now();
    let mut results = String::from("📋 Bulk Removal Report:\n\n");

    for (name, package) in apps {
        let output = if force {
            remove_app(package) // Call the unified remove_app function
        } else {
            remove_app(package)
        };
        results.push_str(&format!("{name}: {output}\n"));
    }

    let elapsed = start.elapsed().as_secs_f32();
    results.push_str(&format!("\n⏱ Completed in {:.1}s", elapsed));

    results
}

#[allow(dead_code)]
pub fn disable_telemetry() -> String {
    let ps_script = r#"
    # Dezactivează task-urile de telemetrie
    $tasks = @(
        "Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser",
        "Microsoft\Windows\Application Experience\ProgramDataUpdater",
        "Microsoft\Windows\Autochk\Proxy",
        "Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
        "Microsoft\Windows\Customer Experience Improvement Program\UsbCeip",
        "Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector",
        "Microsoft\Windows\Feedback\Siuf\DmClient",
        "Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload",
        "Microsoft\Windows\Windows Error Reporting\QueueReporting",
        "Microsoft\Windows\Application Experience\MareBackup",
        "Microsoft\Windows\Application Experience\StartupAppTask",
        "Microsoft\Windows\Application Experience\PcaPatchDbTask",
        "Microsoft\Windows\Maps\MapsUpdateTask"
    )
        foreach ($task in $tasks) {
            schtasks /Change /TN $task /Disable 2>&1
        }

        # Setări registry pentru a dezactiva telemetria
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection" -Name "AllowTelemetry" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection" -Name "AllowTelemetry" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" -Name "ContentDeliveryAllowed" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "HKCU:\SOFTWARE\Microsoft\Siuf\Rules" -Name "NumberOfSIUFInPeriod" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection" -Name "DoNotShowFeedbackNotifications" -Type DWord -Value 1 -Force -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo" -Name "DisabledByGroupPolicy" -Type DWord -Value 1 -Force -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting" -Name "Disabled" -Type DWord -Value 1 -Force -ErrorAction SilentlyContinue

        Write-Output '✅ Telemetry disabled successfully'
        "#;

        crate::utils::run_powershell(ps_script)
    }

#[allow(dead_code)]
pub fn disable_location_tracking() -> String {
    let ps_script = r#"
    # Dezactivează Location Tracking prin registry
    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location" -Name "Value" -Value "Deny" -Force -ErrorAction SilentlyContinue
    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Sensor\Overrides\{BFA794E4-F964-4FDB-90F6-51056BFE4B44}" -Name "SensorPermissionState" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Services\lfsvc\Service\Configuration" -Name "Status" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
    Set-ItemProperty -Path "HKLM:\SYSTEM\Maps" -Name "AutoUpdateEnabled" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue

    Write-Output '✅ Location Tracking disabled successfully'
    "#;

        crate::utils::run_powershell(ps_script)
}

#[allow(dead_code)]
pub fn disable_wifi_sense() -> String {
    let ps_script = r#"
    # Dezactivează Wifi-Sense prin registry
    Set-ItemProperty -Path "HKLM:\Software\Microsoft\PolicyManager\default\WiFi\AllowWiFiHotSpotReporting" -Name "Value" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue
    Set-ItemProperty -Path "HKLM:\Software\Microsoft\PolicyManager\default\WiFi\AllowAutoConnectToWiFiSenseHotspots" -Name "Value" -Type DWord -Value 0 -Force -ErrorAction SilentlyContinue

    Write-Output '✅ Wifi-Sense disabled successfully'
    "#;

    crate::utils::run_powershell(ps_script)
}

/// Disable Windows Consumer Features (prevents automatic Store app/game installs).
/// Applies: sets HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent\DisableWindowsConsumerFeatures = 1
/// Requires admin. Returns textual result for log.
pub fn disable_consumer_features() -> String {
    let ps_script = r#"
    Write-Host 'Disabling Windows Consumer Features...'
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' -Name 'DisableWindowsConsumerFeatures' -Value 1 -Type DWord -Force
    Write-Output '✅ DisableWindowsConsumerFeatures set to 1. Some default Store apps may become inaccessible. A restart may be required.'
    "#;
    crate::utils::run_powershell(ps_script)
}

/// Enable (or restore) Windows Consumer Features by setting the policy to 0.
pub fn enable_consumer_features() -> String {
    let ps_script = r#"
    Write-Host 'Enabling Windows Consumer Features (restoring policy)...'
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' -Name 'DisableWindowsConsumerFeatures' -Value 0 -Type DWord -Force
    Write-Output '✅ DisableWindowsConsumerFeatures set to 0. A restart may be required for full effect.'
    "#;
    crate::utils::run_powershell(ps_script)
}

#[allow(dead_code)]
/// ✅ Enable End Task With Right Click
pub fn enable_end_task_right_click() -> String {
    let ps_script = r#"
        $path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings'
        $name = 'TaskbarEndTask'
        $value = 1

        if (-not (Test-Path $path)) {
            New-Item -Path $path -Force | Out-Null
        }

        New-ItemProperty -Path $path -Name $name -PropertyType DWord -Value $value -Force | Out-Null
        Write-Output '✅ End Task with Right-Click enabled successfully.'
    "#;

    crate::utils::run_powershell(ps_script)
}

#[allow(dead_code)]
/// ❌ Disable End Task With Right Click (Undo)
pub fn disable_end_task_right_click() -> String {
    let ps_script = r#"
        $path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings'
        $name = 'TaskbarEndTask'
        $value = 0

        if (-not (Test-Path $path)) {
            New-Item -Path $path -Force | Out-Null
        }

        New-ItemProperty -Path $path -Name $name -PropertyType DWord -Value $value -Force | Out-Null
        Write-Output '↩ End Task with Right-Click reverted (disabled).'
    "#;

    crate::utils::run_powershell(ps_script)
}

// ... rest of file unchanged (kept for brevity in this message)
// The posted original file is preserved below this point. Only the top-level `#![allow(dead_code)]` was added
// to silence the compiler warnings about unused functions/statics. If you prefer a narrower fix,
// I can add #[allow(dead_code)] only to specific items or remove the attribute and instead wire the missing calls
// from the UI so those functions become used.


#[allow(dead_code)]
/// 🔹 Disable Recall
pub fn disable_recall() -> String {
    let ps_script = r#"
        Write-Host 'Disable Recall'
        New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsAI' -Force | Out-Null
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsAI' -Name 'DisableAIDataAnalysis' -Value 1 -Type DWord
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsAI' -Name 'AllowRecallEnablement' -Value 0 -Type DWord

        New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy' -Force | Out-Null
        Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy' -Name 'VerifiedAndReputablePolicyState' -Value 0 -Type DWord

        DISM /Online /Disable-Feature /FeatureName:Recall /Quiet /NoRestart
        Write-Host 'Please restart your computer in order for the changes to be fully applied.'
    "#;

    crate::utils::run_powershell(ps_script)
}

#[allow(dead_code)]
pub fn enable_recall() -> String {
    let ps_script = r#"
        Write-Host 'Enable Recall'
        DISM /Online /Enable-Feature /FeatureName:Recall /Quiet /NoRestart
        Write-Host 'Please restart your computer in order for the changes to be fully applied.'
    "#;

    crate::utils::run_powershell(ps_script)
}

#[allow(dead_code)]
/// 🔹 Debloat Microsoft Edge
pub fn debloat_edge() -> String {
    let ps_script = r#"
        Write-Host 'Applying Debloat Edge Tweaks...'

        # Edge Update - disable desktop shortcut
        New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\EdgeUpdate' -Force | Out-Null
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\EdgeUpdate' -Name 'CreateDesktopShortcutDefault' -Value 0 -Type DWord

        # Edge policies
        New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Force | Out-Null

        $settings = @{
            EdgeEnhanceImagesEnabled = 0
            PersonalizationReportingEnabled = 0
            ShowRecommendationsEnabled = 0
            HideFirstRunExperience = 1
            UserFeedbackAllowed = 0
            ConfigureDoNotTrack = 1
            AlternateErrorPagesEnabled = 0
            EdgeCollectionsEnabled = 0
            EdgeFollowEnabled = 0
            EdgeShoppingAssistantEnabled = 0
            MicrosoftEdgeInsiderPromotionEnabled = 0
            ShowMicrosoftRewards = 0
            WebWidgetAllowed = 0
            DiagnosticData = 0
            EdgeAssetDeliveryServiceEnabled = 0
            CryptoWalletEnabled = 0
            WalletDonationEnabled = 0
        }

        foreach ($key in $settings.Keys) {
            Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name $key -Value $settings[$key] -Type DWord
        }

        Write-Host '✅ Edge Debloat applied successfully.'
    "#;

    crate::utils::run_powershell(ps_script)
}

#[allow(dead_code)]
// === Adobe Network Block ===
pub fn adobe_network_block() -> String {
    run_command(r#"powershell -Command "
        $remoteHostsUrl = 'https://raw.githubusercontent.com/Ruddernation-Designs/Adobe-URL-Block-List/master/hosts'
        $localHostsPath = 'C:\Windows\System32\drivers\etc\hosts'
        $tempHostsPath = 'C:\Windows\System32\drivers\etc\temp_hosts'

        try {
            Invoke-WebRequest -Uri $remoteHostsUrl -OutFile $tempHostsPath
            Write-Output 'Downloaded the remote HOSTS file to a temporary location.'
        } catch {
            Write-Output 'Failed to download the HOSTS file.'
        }

        try {
            $localHostsContent = Get-Content $localHostsPath -ErrorAction Stop
            $blockStartExists = $localHostsContent -like '*#AdobeNetBlock-start*'
            if ($blockStartExists) {
                Write-Output 'AdobeNetBlock-start already exists. Skipping addition.'
            } else {
                $newBlockContent = Get-Content $tempHostsPath -ErrorAction Stop |
                    Where-Object { $_ -notmatch '^\s*#' -and $_ -ne '' }
                $newBlockHeader = '#AdobeNetBlock-start'
                $newBlockFooter = '#AdobeNetBlock-end'
                $combinedContent = $localHostsContent + $newBlockHeader, $newBlockContent, $newBlockFooter
                $combinedContent | Set-Content $localHostsPath -Encoding ASCII
                Write-Output 'Successfully added the AdobeNetBlock.'
            }
        } catch {
            Write-Output 'Error during processing.'
        }

        Remove-Item $tempHostsPath -ErrorAction Ignore

        try {
            ipconfig /flushdns | Out-Null
            Write-Output 'DNS cache flushed successfully.'
        } catch {
            Write-Output 'Failed to flush DNS cache.'
        }
    ""#)
}

#[allow(dead_code)]
// === Adobe Debloat ===
pub fn adobe_debloat() -> String {
    run_command(r#"powershell -NoProfile -ExecutionPolicy Bypass -Command "
        function CCStopper {
            $path = 'C:\Program Files (x86)\Common Files\Adobe\Adobe Desktop Common\ADS\Adobe Desktop Service.exe'
            if (Test-Path $path) {
                Takeown /f $path
                $acl = Get-Acl $path
                $acl.SetOwner([System.Security.Principal.NTAccount]'Administrators')
                $acl | Set-Acl $path
                Rename-Item -Path $path -NewName 'Adobe Desktop Service.exe.old' -Force
                Write-Output '✅ Adobe Desktop Service disabled.'
            } else {
                Write-Output 'ℹ️ Adobe Desktop Service not found in default location.'
            }
        }

        function AcrobatUpdates {
            $rootPath = 'HKLM:\SOFTWARE\WOW6432Node\Adobe\Adobe ARM\Legacy\Acrobat'
            $subKeys = Get-ChildItem -Path $rootPath | Where-Object { $_.PSChildName -like '{*}' }
            foreach ($subKey in $subKeys) {
                $fullPath = Join-Path -Path $rootPath -ChildPath $subKey.PSChildName
                try {
                    Set-ItemProperty -Path $fullPath -Name Mode -Value 0
                    Write-Output '✅ Acrobat Updates disabled.'
                } catch {
                    Write-Output \"⚠️ Registry Key for Acrobat Updates not found: $fullPath\"
                }
            }
        }

        CCStopper
        AcrobatUpdates

        $services = @('AGSService','AGMService','AdobeUpdateService','Adobe Acrobat Update',
                      'Adobe Genuine Monitor Service','AdobeARMservice','Adobe Licensing Console',
                      'CCXProcess','AdobeIPCBroker','CoreSync')
        foreach ($svc in $services) {
            try {
                Set-Service -Name $svc -StartupType Disabled -ErrorAction SilentlyContinue
                Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
                Write-Output \"✅ Disabled service: $svc\"
            } catch {
                Write-Output \"⚠️ Failed to disable service: $svc\"
            }
        }
    ""#)
}

#[allow(dead_code)]
// === Disable Microsoft Copilot ===
pub fn disable_copilot() -> String {
    run_command(r##"powershell -NoProfile -ExecutionPolicy Bypass -Command "
        Write-Output '🛑 Removing Microsoft Copilot...'

        # Registry tweaks
        try {
            New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot' -Force | Out-Null
            Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot' -Name 'TurnOffWindowsCopilot' -Value 1 -Type DWord
            Write-Output '✅ Copilot disabled via HKLM policy.'
        } catch {
            Write-Output '⚠️ Failed to apply HKLM Copilot policy.'
        }

        try {
            New-Item -Path 'HKCU:\Software\Policies\Microsoft\Windows\WindowsCopilot' -Force | Out-Null
            Set-ItemProperty -Path 'HKCU:\Software\Policies\Microsoft\Windows\WindowsCopilot' -Name 'TurnOffWindowsCopilot' -Value 1 -Type DWord
            Write-Output '✅ Copilot disabled via HKCU policy.'
        } catch {
            Write-Output '⚠️ Failed to apply HKCU Copilot policy.'
        }

        try {
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'ShowCopilotButton' -Value 0 -Type DWord
            Write-Output '✅ Copilot button removed from taskbar.'
        } catch {
            Write-Output '⚠️ Failed to remove Copilot button.'
        }

        # Optional: remove package
        try {
            dism /online /remove-package /package-name:Microsoft.Windows.Copilot | Out-Null
            Write-Output '✅ Copilot package removal attempted.'
        } catch {
            Write-Output '⚠️ Failed to remove Copilot package.'
        }

        Write-Output '➡️ Please restart Windows to complete changes.'
    "##)
}

#[allow(dead_code)]
// === Set Display for Performance ===
pub fn set_display_for_performance() -> String {
    run_command(r##"powershell -NoProfile -ExecutionPolicy Bypass -Command "
        Write-Output '🎛️ Applying Display Performance Tweaks...'

        # Registry changes
        try {
            Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'DragFullWindows' -Value '0' -Type String
            Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'MenuShowDelay' -Value '200' -Type String
            Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop\WindowMetrics' -Name 'MinAnimate' -Value '0' -Type String
            Set-ItemProperty -Path 'HKCU:\Control Panel\Keyboard' -Name 'KeyboardDelay' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'ListviewAlphaSelect' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'ListviewShadow' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'TaskbarAnimations' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects' -Name 'VisualFXSetting' -Value 3 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\DWM' -Name 'EnableAeroPeek' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'TaskbarMn' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'TaskbarDa' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'ShowTaskViewButton' -Value 0 -Type DWord
            Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Search' -Name 'SearchboxTaskbarMode' -Value 0 -Type DWord
            Write-Output '✅ Registry tweaks applied.'
        } catch {
            Write-Output '⚠️ Failed to apply some registry tweaks.'
        }

        # Apply UserPreferencesMask for Performance
        try {
            Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'UserPreferencesMask' -Type Binary -Value ([byte[]](144,18,3,128,16,0,0,0))
            Write-Output '✅ UserPreferencesMask set for performance.'
        } catch {
            Write-Output '⚠️ Failed to update UserPreferencesMask.'
        }

        Write-Output '➡️ Please log off or restart for changes to take effect.'
    "##)
}

use std::process::Output;

// Rulează un proces ascuns și capturează output.
fn run_hidden(cmd: &str, args: &[&str]) -> std::io::Result<Output> {
    let mut c = std::process::Command::new(cmd);
    c.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c.output()
}

/// Încearcă să verifice/actualizeze winget. Întoarce true dacă e gata de folosire.
pub fn ensure_winget_ready(log: Arc<Mutex<String>>) -> bool {
    match run_hidden("winget", &["--version"]) {
        Ok(o) if o.status.success() => {
            push_line(&log, "✅ winget available.");
            // încercăm și un source update (nu strică)
            let _ = run_hidden("winget", &["source", "update"]);
            true
        }
        _ => {
            push_line(&log, "⚠ winget not found. Attempting to help you install 'App Installer' (Microsoft Store)...");
            // Deschide pagina App Installer în Store; utilizatorul trebuie să confirme instalarea.
            let _ = std::process::Command::new("powershell")
                .args(&[
                    "-ExecutionPolicy", "Bypass",
                    "-Command",
                    "Start-Process 'ms-windows-store://pdp/?productid=9NBLGGH4NNS1'"
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            push_line(&log, "Please install/update 'App Installer' from the Store, then retry.");
            false
        }
    }
}

/// Verifică dacă un id winget este instalat (winget list --id).
pub fn winget_is_installed(id: &str) -> bool {
    match run_hidden("winget", &["list", "--id", id, "-e"]) {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout);
            // winget scrie „No installed package found…” când nu găsește nimic
            o.status.success() && !out.to_lowercase().contains("no installed package")
        }
        Err(_) => false,
    }
}

/// Instalează pachetul (silent) și streamează în log.
pub fn winget_install(id: &str, log: Arc<Mutex<String>>) -> i32 {
    let args = ["install", "--id", id, "-e", "--silent", "--accept-package-agreements", "--accept-source-agreements"];
    match run_command_stream_and_wait(log, "winget", &args) {
        Ok(code) => code,
        Err(_) => -1,
    }
}

/// Dezinstalează pachetul (unde se poate) și streamează în log.
pub fn winget_uninstall(id: &str, log: Arc<Mutex<String>>) -> i32 {
    let args = ["uninstall", "--id", id, "-e", "--silent"];
    match run_command_stream_and_wait(log, "winget", &args) {
        Ok(code) => code,
        Err(_) => -1,
    }
}

/// Upgrade pentru pachet (silent) și streamează în log.
pub fn winget_upgrade(id: &str, log: Arc<Mutex<String>>) -> i32 {
    let args = ["upgrade", "--id", id, "-e", "--silent", "--accept-package-agreements", "--accept-source-agreements"];
    match run_command_stream_and_wait(log, "winget", &args) {
        Ok(code) => code,
        Err(_) => -1,
    }
}

/// seteaza un dns custom pentru toate adaptoarele de retea
pub fn set_dns(provider: &str) -> String {
    match provider {
        "Google" => run_dns("8.8.8.8", "8.8.4.4"),
        "Cloudflare" => run_dns("1.1.1.1", "1.0.0.1"),
        "Cloudflare_Malware" => run_dns("1.1.1.2", "1.0.0.2"),
        "Cloudflare_Malware_Adult" => run_dns("1.1.1.3", "1.0.0.3"),
        "Open_DNS" => run_dns("208.67.222.222", "208.67.220.220"),
        "Quad9" => run_dns("9.9.9.9", "149.112.112.112"),
        "AdGuard_Ads_Trackers" => run_dns("94.140.14.14", "94.140.15.15"),
        "AdGuard_Ads_Trackers_Malware_Adult" => run_dns("94.140.14.15", "94.140.15.16"),
        "dns0.eu_Open" => run_dns("193.110.81.254", "185.253.5.254"),
        "dns0.eu_ZERO" => run_dns("193.110.81.9", "185.253.5.9"),
        "dns0.eu_KIDS" => run_dns("193.110.81.1", "185.253.5.1"),
        "Automatic (DHCP)" => reset_dns(),
        _ => "❌ Unknown DNS provider".to_string(),
    }
}

fn run_dns(primary: &str, secondary: &str) -> String {
    let mut output = String::new();

    // Get all network adapter names
    let get_interfaces_cmd = "Get-NetAdapter | Select-Object -ExpandProperty Name";
    let interfaces_output = run_command(&format!("powershell -Command \"{}\"", get_interfaces_cmd));

    let interfaces: Vec<&str> = interfaces_output
        .lines()
        .filter(|s| !s.trim().is_empty())
        .collect();

    if interfaces.is_empty() {
        return "❌ No network adapters found.".to_string();
    }

    for iface in interfaces {
        // Set primary DNS
        let set_primary_cmd = format!(
            "netsh interface ip set dns name=\"{}\" static {}",
            iface, primary
        );
        let res_primary = run_command(&set_primary_cmd);
        output.push_str(&format!("Setting primary DNS for '{}' to {}: {}\n", iface, primary, res_primary.trim()));

        // Set secondary DNS
        let set_secondary_cmd = format!(
            "netsh interface ip add dns name=\"{}\" {} index=2",
            iface, secondary
        );
        let res_secondary = run_command(&set_secondary_cmd);
        output.push_str(&format!("Setting secondary DNS for '{}' to {}: {}\n", iface, secondary, res_secondary.trim()));
    }

    // Flush DNS cache
    let flush_dns_cmd = "ipconfig /flushdns";
    let res_flush = run_command(flush_dns_cmd);
    output.push_str(&format!("Flushing DNS cache: {}\n", res_flush.trim()));

    output
}

fn reset_dns() -> String {
    let iface_output = std::process::Command::new("cmd")
        .args(["/C", "netsh interface show interface"])
        .output();

    let iface_name = match iface_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().find(|l| l.contains("Connected")) {
                line.split_whitespace().last().unwrap_or("Ethernet").to_string()
            } else {
                "Ethernet".to_string()
            }
        }
        Err(_) => "Ethernet".to_string(),
    };

    let cmd = format!(
        "netsh interface ip set dns name=\"{iface}\" dhcp",
        iface = iface_name
    );

    match std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                format!("✅ DNS reset to Automatic (DHCP) on {iface_name}")
            } else {
                format!(
                    "❌ Failed to reset DNS: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        }
        Err(e) => format!("❌ Error running command: {e}"),
    }
}

use std::io::Read;
/// Tipul logului partajat între threads/UI
type SharedOutput = Arc<Mutex<String>>;

/// Rulează o comandă și trimite output-ul în `output_log`
fn run_command_and_log(cmd: &str, args: &[&str], output_log: &SharedOutput) -> bool {
    let mut child = StdCommand::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("❌ Failed to spawn process");

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }

    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }

    let status = child.wait().unwrap();

    let mut log = output_log.lock().unwrap();
    log.push_str(&format!("▶ {} {}\n", cmd, args.join(" ")));
    if !stdout.is_empty() {
        log.push_str(&format!("{}\n", stdout));
    }
    if !stderr.is_empty() {
        log.push_str(&format!("{}\n", stderr));
    }
    log.push_str(&format!("Exit code: {}\n\n", status.code().unwrap_or(-1)));

    status.success()
}

pub fn upgrade_all_apps_with_log(output_log: SharedOutput) {
    // 1. Verificare instalare winget și chocolatey
    let winget_installed = check_package_manager_installed("winget", &output_log);
    let choco_installed = check_package_manager_installed("choco", &output_log);

    // 2 & 3. Instalare/actualizare package managers
    if !winget_installed {
        log_message(&output_log, "Installing winget...");
        install_winget(&output_log);
    } else {
        log_message(&output_log, "Checking winget updates...");
        update_winget(&output_log);
    }

    if !choco_installed {
        log_message(&output_log, "Installing Chocolatey...");
        install_chocolatey(&output_log);
    } else {
        log_message(&output_log, "Updating Chocolatey...");
        run_command_and_log("choco", &["upgrade", "chocolatey", "-y"], &output_log);
    }

    // 4. Verificare finală și upgrade aplicații
    let winget_ready = check_package_manager_installed("winget", &output_log);
    let choco_ready = check_package_manager_installed("choco", &output_log);

    if !winget_ready && !choco_ready {
        log_message(&output_log, "ERROR: No package managers available!");
        return;
    }

    // 5. Upgrade pe fiecare canal disponibil
    if choco_ready {
        log_message(&output_log, "\n=== Upgrading apps via Chocolatey ===");
        run_command_and_log("choco", &["upgrade", "all", "-y"], &output_log);
    }

    if winget_ready {
        log_message(&output_log, "\n=== Upgrading apps via Winget ===");
        run_command_and_log("winget", &["upgrade", "--all", "--silent"], &output_log);
    }

    log_message(&output_log, "\n=== Upgrade process completed ===");
}

fn check_package_manager_installed(manager: &str, output_log: &SharedOutput) -> bool {
    let result = StdCommand::new(manager)
 .arg("--version")
        .output();
    
    match result {
        Ok(output) => {
            let installed = output.status.success();
            if installed {
                log_message(output_log, &format!("{} is installed", manager));
            } else {
                log_message(output_log, &format!("{} not found", manager));
            }
            installed
        }
        Err(_) => {
            log_message(output_log, &format!("{} not found", manager));
            false
        }
    }
}

fn update_winget(output_log: &SharedOutput) {
    // Winget se actualizează prin App Installer din Microsoft Store
    // Nu există comandă directă, dar putem verifica versiunea
    log_message(output_log, "Winget updates automatically via Microsoft Store");
}

fn install_winget(output_log: &SharedOutput) {
    log_message(output_log, "Installing winget via App Installer...");
    // Descarcă și instalează App Installer de pe GitHub
    let url = "https://github.com/microsoft/winget-cli/releases/latest/download/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle";
    
    run_command_and_log(
        "powershell",
        &["-Command", &format!("Add-AppxPackage -Path (Invoke-WebRequest -Uri '{}' -OutFile '$env:TEMP\\winget.msixbundle' -PassThru).Path", url)],
        output_log
    );
}

fn install_chocolatey(output_log: &SharedOutput) {
    log_message(output_log, "Installing Chocolatey...");
    let install_script = "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))";
    
    run_command_and_log(
        "powershell",
        &["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", install_script],
        output_log
    );
}

fn log_message(output_log: &SharedOutput, message: &str) {
    if let Ok(mut log) = output_log.lock() {
        log.push_str(message);
        log.push_str("\n");
    }
}

/// Reinstall Winget via Chocolatey
#[allow(dead_code)]
pub fn reinstall_winget_with_log(output_log: SharedOutput) {
    run_command_and_log("choco", &["install", "winget", "-y", "--force"], &output_log);
}

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag: true dacă operațiunea este în desfășurare.
pub static CREATE_RESTORE_POINT_RUNNING: AtomicBool = AtomicBool::new(false);

/// Runs the "create restore point" PowerShell script in a background thread
/// and appends INFO/WARNING/ERROR/SUCCESS lines into `log`.
pub fn create_restore_point_live(log: Arc<Mutex<String>>) {
    // Dacă e deja în execuție, scriem un mesaj și ieșim rapid.
    if CREATE_RESTORE_POINT_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        let mut lg = log.lock().unwrap();
        if lg.is_empty() {
            *lg = "INFO: Restore point creation already running.".to_string();
        } else {
            *lg = format!("{}\nINFO: Restore point creation already running.", lg);
        }
        return;
    }

    // RAII guard pentru a ne asigura că flag-ul e resetat la final, oricum s-ar termina thread-ul.
    struct RunningGuard;
    impl Drop for RunningGuard {
        fn drop(&mut self) {
            CREATE_RESTORE_POINT_RUNNING.store(false, Ordering::SeqCst);
        }
    }

    let log_for_thread = log.clone();

    thread::spawn(move || {
        let _guard = RunningGuard; // se va executa Drop când thread-ul se termină

        // Inițial: mesaj de start
        {
            let mut lg = log_for_thread.lock().unwrap();
            if lg.is_empty() {
                *lg = "INFO: Starting restore point creation...".to_string();
            } else {
                *lg = format!("{}\nINFO: Starting restore point creation...", lg);
            }
        }

        let script = r#"
            # Check if the user has administrative privileges
            if (-Not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
                Write-Output "ERROR: Please run this as administrator."
                exit 1
            }

            # Ensure System Restore is enabled on the system drive
            try {
                Enable-ComputerRestore -Drive "$env:SystemDrive" | Out-Null
            } catch {
                Write-Output ("WARNING: Could not explicitly enable System Restore: " + $_.Exception.Message)
            }

            # Allow multiple restore points per day if the policy key doesn't exist
            $exists = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore" -Name "SystemRestorePointCreationFrequency" -ErrorAction SilentlyContinue
            if ($null -eq $exists) {
                Write-Output "INFO: Changing system to allow multiple restore points per day..."
                try {
                    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore" -Name "SystemRestorePointCreationFrequency" -Value 0 -Type DWord -Force -ErrorAction Stop
                } catch {
                    Write-Output ("WARNING: Failed to set SystemRestorePointCreationFrequency: " + $_.Exception.Message)
                }
            }

            # Try to import module (Get-ComputerRestorePoint)
            try {
                Import-Module Microsoft.PowerShell.Management -ErrorAction Stop
            } catch {
                Write-Output ("ERROR: Failed to load Microsoft.PowerShell.Management module: " + $_.Exception.Message)
                exit 1
            }

            # Get restore points for today
            try {
                $existingRestorePoints = Get-ComputerRestorePoint | Where-Object { $_.CreationTime.Date -eq (Get-Date).Date }
            } catch {
                Write-Output ("ERROR: Failed to retrieve restore points: " + $_.Exception.Message)
                exit 1
            }

            if ($existingRestorePoints.Count -eq 0) {
                $description = "System Restore Point created by Eoliann Windows Tools on $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
                try {
                    Checkpoint-Computer -Description $description -RestorePointType "MODIFY_SETTINGS" -ErrorAction Stop
                    Write-Output "SUCCESS: System Restore Point Created Successfully"
                } catch {
                    Write-Output ("ERROR: Failed to create restore point: " + $_.Exception.Message)
                    exit 1
                }
            } else {
                Write-Output "INFO: A restore point already exists for today; skipping creation."
            }
            "#;

        // Spawn PowerShell și capture stdout/stderr
        let mut child = match StdCommand::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy").arg("Bypass")
            .arg("-Command").arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let mut lg = log_for_thread.lock().unwrap();
                *lg = format!("{}{}\nERROR: Failed to spawn PowerShell: {}", lg, if lg.is_empty() { "" } else { "\n" }, e);
                return;
            }
        };

        // Preluăm stdout/stderr și le citim în thread-uri separate; ca să păstrăm istoricul, facem append.
        let mut handles = Vec::new();

        if let Some(out) = child.stdout.take() {
            let lg_clone = log_for_thread.clone();
            handles.push(thread::spawn(move || {
                let reader = BufReader::new(out);
                for line_res in reader.lines() {
                    let line = line_res.unwrap_or_default();
                    let mut lg = lg_clone.lock().unwrap();
                    if lg.is_empty() {
                        *lg = line.clone();
                    } else {
                        *lg = format!("{}\n{}", lg, line);
                    }
                }
            }));
        }

        if let Some(err) = child.stderr.take() {
            let lg_clone = log_for_thread.clone();
            handles.push(thread::spawn(move || {
                let reader = BufReader::new(err);
                for line_res in reader.lines() {
                    let line = line_res.unwrap_or_default();
                    // Prefixăm liniile din stderr cu "ERROR: " dacă nu sunt deja prefixate
                    let to_append = if line.starts_with("ERROR:") || line.starts_with("WARNING:") || line.starts_with("INFO:") || line.starts_with("SUCCESS:") {
                        line.clone()
                    } else {
                        format!("ERROR: {}", line)
                    };
                    let mut lg = lg_clone.lock().unwrap();
                    if lg.is_empty() {
                        *lg = to_append;
                    } else {
                        *lg = format!("{}\n{}", lg, to_append);
                    }
                }
            }));
        }

        // Așteptăm terminarea procesului
        match child.wait() {
            Ok(status) => {
                // așteptăm reader threads
                for h in handles {
                    let _ = h.join();
                }

                let mut lg = log_for_thread.lock().unwrap();
                if status.success() {
                    // adăugăm un mesaj final de succes (doar dacă nu a mai apărut deja unul)
                    if !lg.contains("SUCCESS:") {
                        if lg.is_empty() {
                            *lg = "SUCCESS: Restore point creation finished successfully.".to_string();
                        } else {
                            *lg = format!("{}\nSUCCESS: Restore point creation finished successfully.", lg);
                        }
                    }
                } else {
                    if lg.is_empty() {
                        *lg = format!("ERROR: PowerShell exited with status: {}.", status);
                    } else {
                        *lg = format!("{}\nERROR: PowerShell exited with status: {}.", lg, status);
                    }
                }
            }
            Err(e) => {
                let mut lg = log_for_thread.lock().unwrap();
                if lg.is_empty() {
                    *lg = format!("ERROR: Failed to wait for PowerShell: {}", e);
                } else {
                    *lg = format!("{}\nERROR: Failed to wait for PowerShell: {}", lg, e);
                }
            }
        }

        // RunningGuard va fi drop-uit aici și va reseta CREATE_RESTORE_POINT_RUNNING = false
    });
}


/// Flag global: true dacă *orice* operațiune din fereastra asta e în execuție.
pub static GLOBAL_OP_RUNNING: AtomicBool = AtomicBool::new(false);

/// Guard RAII: când este droppuit, resetează `GLOBAL_OP_RUNNING` = false.
pub struct GlobalOpGuard;

impl Drop for GlobalOpGuard {
    fn drop(&mut self) {
        GLOBAL_OP_RUNNING.store(false, Ordering::SeqCst);
    }
}

/// Încearcă să seteze flag-ul global. Dacă reușește, adaugă un mesaj inițial în `log`
/// și returnează `Some(GlobalOpGuard)` — mută acel guard în thread pentru a păstra flag-ul.
/// Dacă nu reușește (altă operațiune rulează deja), returnează `None` și scrie un mesaj în `log`.
#[allow(dead_code)]
pub fn try_start_global_op(op_name: &str, log: &Arc<Mutex<String>>) -> Option<GlobalOpGuard> {
    // încercăm tranzacțional să setăm flag-ul
    match GLOBAL_OP_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            // setat cu succes — scriem mesaj inițial
            let mut lg = log.lock().unwrap();
            if lg.is_empty() {
                *lg = format!("INFO: Starting {}...", op_name);
            } else {
                *lg = format!("{}\nINFO: Starting {}...", lg, op_name);
            }
            Some(GlobalOpGuard)
        }
        Err(_) => {
            // deja rulează altceva
            let mut lg = log.lock().unwrap();
            if lg.is_empty() {
                *lg = "INFO: Another operation is already running. Please wait...".to_string();
            } else {
                *lg = format!("{}\nINFO: Another operation is already running. Please wait...", lg);
            }
            None
        }
    }
}

/// Disable Activity History: sets EnableActivityFeed, PublishUserActivities, UploadUserActivities = 0
/// This also attempts light cleanup for Timeline/Activity data. Requires admin. Returns textual result.
pub fn disable_activity_history() -> String {
    let ps = r#"
    Write-Host 'Disabling Activity History policies...'
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Name 'EnableActivityFeed' -Value 0 -Type DWord -Force
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Name 'PublishUserActivities' -Value 0 -Type DWord -Force
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Name 'UploadUserActivities' -Value 0 -Type DWord -Force

    # Optional cleanup: clear Timeline/Activity & recent items (may require admin)
    try {
        $timeline = "$env:LOCALAPPDATA\ConnectedDevicesPlatform\Livedata"
        if (Test-Path $timeline) { Remove-Item -Path $timeline -Recurse -Force -ErrorAction SilentlyContinue }
    } catch {
        # ignore errors from cleanup
    }

    Write-Output '✅ Activity History policies set to 0. Recent docs/clipboard/run history may require additional cleanup and a restart.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Enable Activity History (restore) by setting policies back to 1.
/// Note: this sets the policy values to 1; if you prefer to remove the policy values instead, modify the script.
pub fn enable_activity_history() -> String {
    let ps = r#"
    Write-Host 'Restoring Activity History policies...'
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Name 'EnableActivityFeed' -Value 1 -Type DWord -Force
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Name 'PublishUserActivities' -Value 1 -Type DWord -Force
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\System' -Name 'UploadUserActivities' -Value 1 -Type DWord -Force
    Write-Output '✅ Activity History policies set to 1.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Disable Storage Sense for the current user by setting StoragePolicy '01' = 0.
/// Note: acest tweak modifică HKCU și afectează utilizatorul curent (nu necesită admin).
/// Returnează textul rezultatului pentru log.
pub fn disable_storage_sense() -> String {
    let ps = r#"
    Write-Host 'Disabling Storage Sense for current user...'
    New-Item -Path 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy' -Force | Out-Null
    Set-ItemProperty -Path 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy' -Name '01' -Value 0 -Type DWord -Force
    Write-Output '✅ Storage Sense disabled (StoragePolicy[01] = 0) for current user. Temporary files will no longer be auto-deleted by Storage Sense.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Enable (restore) Storage Sense for the current user by setting StoragePolicy '01' = 1.
/// Returnează textul rezultatului pentru log.
pub fn enable_storage_sense() -> String {
    let ps = r#"
    Write-Host 'Enabling Storage Sense for current user (restoring)...'
    New-Item -Path 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy' -Force | Out-Null
    Set-ItemProperty -Path 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy' -Name '01' -Value 1 -Type DWord -Force
    Write-Output '✅ Storage Sense enabled (StoragePolicy[01] = 1) for current user.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Set Hibernation as default (good for laptops).
/// Most modern laptops have connected standby enabled which drains the battery;
/// this enables hibernation and exposes the relevant power options in UI.
/// Requires admin. Returns textual result for logging.
pub fn set_hibernation_default() -> String {
    let ps = r#"
    Write-Host 'Setting Hibernation as default (applying registry tweaks and powercfg settings)...'

    # Expose Hibernation powersettings in Power Options (Attributes = 2)
    New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\238C9FA8-0AAD-41ED-83F4-97BE242C8F20\7bc4a2f9-d8fc-4469-b07b-33eb785aaca0' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\238C9FA8-0AAD-41ED-83F4-97BE242C8F20\7bc4a2f9-d8fc-4469-b07b-33eb785aaca0' -Name 'Attributes' -Value 2 -Type DWord -Force

    New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\abfc2519-3608-4c2a-94ea-171b0ed546ab\94ac6d29-73ce-41a6-809f-6363ba21b47e' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\abfc2519-3608-4c2a-94ea-171b0ed546ab\94ac6d29-73ce-41a6-809f-6363ba21b47e' -Name 'Attributes' -Value 2 -Type DWord -Force

    # Turn on hibernation and tweak default timeouts to prefer hibernate (good for battery)
    Write-Host 'Turning on hibernation...'
    Start-Process -FilePath powercfg -ArgumentList '/hibernate on' -NoNewWindow -Wait

    # Adjust timeouts (these mirror the example; adapt numbers as desired)
    Start-Process -FilePath powercfg -ArgumentList '/change standby-timeout-ac 60' -NoNewWindow -Wait
    Start-Process -FilePath powercfg -ArgumentList '/change standby-timeout-dc 60' -NoNewWindow -Wait
    Start-Process -FilePath powercfg -ArgumentList '/change monitor-timeout-ac 10' -NoNewWindow -Wait
    Start-Process -FilePath powercfg -ArgumentList '/change monitor-timeout-dc 1' -NoNewWindow -Wait

    Write-Output '✅ Hibernation enabled and defaults applied. A restart is recommended for all changes to take full effect.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Restore previous/default Hibernation settings (undo).
/// Restores registry Attributes values to conservative defaults and turns hibernation off.
/// Requires admin. Returns textual result for logging.
pub fn restore_hibernation_defaults() -> String {
    let ps = r#"
    Write-Host 'Restoring Hibernation defaults (undo)...'

    # Restore Attributes to original values (as per manifest defaults)
    # First path: original was 1
    New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\238C9FA8-0AAD-41ED-83F4-97BE242C8F20\7bc4a2f9-d8fc-4469-b07b-33eb785aaca0' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\238C9FA8-0AAD-41ED-83F4-97BE242C8F20\7bc4a2f9-d8fc-4469-b07b-33eb785aaca0' -Name 'Attributes' -Value 1 -Type DWord -Force

    # Second path: original was 0
    New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\abfc2519-3608-4c2a-94ea-171b0ed546ab\94ac6d29-73ce-41a6-809f-6363ba21b47e' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\abfc2519-3608-4c2a-94ea-171b0ed546ab\94ac6d29-73ce-41a6-809f-6363ba21b47e' -Name 'Attributes' -Value 0 -Type DWord -Force

    # Turn off hibernation (if you want to disable)
    Write-Host 'Turning off hibernation...'
    Start-Process -FilePath powercfg -ArgumentList '/hibernate off' -NoNewWindow -Wait

    # Restore timeouts to conservative defaults (example values)
    Start-Process -FilePath powercfg -ArgumentList '/change standby-timeout-ac 15' -NoNewWindow -Wait
    Start-Process -FilePath powercfg -ArgumentList '/change standby-timeout-dc 15' -NoNewWindow -Wait
    Start-Process -FilePath powercfg -ArgumentList '/change monitor-timeout-ac 15' -NoNewWindow -Wait
    Start-Process -FilePath powercfg -ArgumentList '/change monitor-timeout-dc 15' -NoNewWindow -Wait

    Write-Output '✅ Hibernation and power settings restored to defaults. A restart is recommended.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Set Time to UTC (Dual Boot)
/// Essential for computers that are dual booting with Linux. Sets HKLM:\SYSTEM\CurrentControlSet\Control\TimeZoneInformation\RealTimeIsUniversal = 1
/// Requires Administrator. Returns textual result for log.
pub fn set_time_utc() -> String {
    let ps = r#"
    Write-Host 'Setting RealTimeIsUniversal = 1 (use UTC for hardware clock)...'
    New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\TimeZoneInformation' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\TimeZoneInformation' -Name 'RealTimeIsUniversal' -Value 1 -Type DWord -Force
    Write-Output '✅ RealTimeIsUniversal set to 1. The hardware clock will be treated as UTC (good for dual-boot with Linux). A reboot is recommended.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Restore Time to Local (undo)
/// Restores RealTimeIsUniversal to 0 (default Windows behaviour — hardware clock is local time).
/// Requires Administrator. Returns textual result for log.
pub fn restore_time_local() -> String {
    let ps = r#"
    Write-Host 'Restoring RealTimeIsUniversal = 0 (use local time for hardware clock)...'
    New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\TimeZoneInformation' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\TimeZoneInformation' -Name 'RealTimeIsUniversal' -Value 0 -Type DWord -Force
    Write-Output '✅ RealTimeIsUniversal set to 0. The hardware clock will be treated as local time. A reboot is recommended.'
    "#;
    crate::utils::run_powershell(ps)
}

/// Remove OneDrive: moves OneDrive files to user profile default folders and uninstalls OneDrive.
/// Requires Administrator. Returns textual result for display/logging.
pub fn remove_onedrive() -> String {
    let ps = r#"
    $OneDrivePath = $($env:OneDrive)
    Write-Host "Removing OneDrive"
    $regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\OneDriveSetup.exe"

    if (Test-Path $regPath) {
        try {
            $OneDriveUninstallString = Get-ItemPropertyValue $regPath -Name "UninstallString" -ErrorAction Stop

            # Extract executable and args robustly (support quoted path)
            $OneDriveExe = ""
            $OneDriveArgs = ""
            if ($OneDriveUninstallString -match '^\s*"(.*?)"(.*)$') {
                $OneDriveExe = $matches[1]
                $OneDriveArgs = $matches[2].Trim()
            } elseif ($OneDriveUninstallString -match '^\s*(\S+)(.*)$') {
                $OneDriveExe = $matches[1]
                $OneDriveArgs = $matches[2].Trim()
            } else {
                $OneDriveExe = $OneDriveUninstallString.Trim()
                $OneDriveArgs = ""
            }

            # Ensure we always pass /silent to the uninstall if possible
            if ($OneDriveExe -eq "") {
                Write-Host "Could not parse uninstall executable from registry value: $OneDriveUninstallString" -ForegroundColor Red
            } else {
                if ($OneDriveArgs -ne "") { $OneDriveArgs = "$OneDriveArgs /silent" } else { $OneDriveArgs = "/silent" }
                Write-Host "Running uninstall: $OneDriveExe $OneDriveArgs"
                Start-Process -FilePath $OneDriveExe -ArgumentList $OneDriveArgs -NoNewWindow -Wait -ErrorAction Stop
            }
        } catch {
            Write-Host "Failed to run uninstall string: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "OneDrive doesn't seem to be installed anymore" -ForegroundColor Yellow
        return
    }

    # Check if OneDrive got Uninstalled
    if (-not (Test-Path $regPath)) {
        Write-Host "Copy downloaded Files from the OneDrive Folder to Root UserProfile"
        try {
            if (Test-Path $OneDrivePath) {
                # Move files from OneDrive to user profile root (robocopy /mov /e /xj)
                Start-Process -FilePath powershell -ArgumentList "robocopy '$($OneDrivePath)' '$($env:USERPROFILE.TrimEnd())\ ' /mov /e /xj" -NoNewWindow -Wait
            } else {
                Write-Host "OneDrive folder not found at $OneDrivePath" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "Robocopy failed: $_" -ForegroundColor Yellow
        }

        Write-Host "Removing OneDrive leftovers"
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:localappdata\Microsoft\OneDrive"
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:localappdata\OneDrive"
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:programdata\Microsoft OneDrive"
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:systemdrive\OneDriveTemp"
        reg delete "HKEY_CURRENT_USER\Software\Microsoft\OneDrive" -f 2>$null

        # check if directory is empty before removing:
        try {
            if (Test-Path $OneDrivePath) {
                $count = (Get-ChildItem $OneDrivePath -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object).Count
                if ($count -eq 0) {
                    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $OneDrivePath
                } else {
                    Write-Host "Note: OneDrive folder still contains $count items. Manual check recommended." -ForegroundColor Yellow
                }
            }
        } catch {
            # ignore
        }

        Write-Host "Remove OneDrive from explorer sidebar"
        try {
            Set-ItemProperty -Path "HKCR:\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}" -Name "System.IsPinnedToNameSpaceTree" -Value 0 -ErrorAction SilentlyContinue
            Set-ItemProperty -Path "HKCR:\Wow6432Node\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}" -Name "System.IsPinnedToNameSpaceTree" -Value 0 -ErrorAction SilentlyContinue
        } catch { }

        Write-Host "Removing run hook for new users"
        try {
            reg load "hku\Default" "C:\Users\Default\NTUSER.DAT" 2>$null
            reg delete "HKEY_USERS\Default\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v "OneDriveSetup" /f 2>$null
            reg unload "hku\Default" 2>$null
        } catch { }

        Write-Host "Removing startmenu entry"
        Remove-Item -Force -ErrorAction SilentlyContinue "$env:userprofile\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\OneDrive.lnk"

        Write-Host "Removing scheduled task(s)"
        Get-ScheduledTask -TaskPath '\' -TaskName 'OneDrive*' -ea SilentlyContinue | Unregister-ScheduledTask -Confirm:$false -ErrorAction SilentlyContinue

        Write-Host "Shell Fixing: restoring default user shell folders"
        $ushell = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders'
        Set-ItemProperty -Path $ushell -Name "AppData" -Value "$env:userprofile\AppData\Roaming" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Cache" -Value "$env:userprofile\AppData\Local\Microsoft\Windows\INetCache" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Cookies" -Value "$env:userprofile\AppData\Local\Microsoft\Windows\INetCookies" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Favorites" -Value "$env:userprofile\Favorites" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "History" -Value "$env:userprofile\AppData\Local\Microsoft\Windows\History" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Local AppData" -Value "$env:userprofile\AppData\Local" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "My Music" -Value "$env:userprofile\Music" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "My Video" -Value "$env:userprofile\Videos" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "NetHood" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Network Shortcuts" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "PrintHood" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Printer Shortcuts" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Programs" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Start Menu\Programs" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Recent" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Recent" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "SendTo" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\SendTo" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Start Menu" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Start Menu" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Startup" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Templates" -Value "$env:userprofile\AppData\Roaming\Microsoft\Windows\Templates" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "{374DE290-123F-4565-9164-39C4925E467B}" -Value "$env:userprofile\Downloads" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Desktop" -Value "$env:userprofile\Desktop" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "My Pictures" -Value "$env:userprofile\Pictures" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "Personal" -Value "$env:userprofile\Documents" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "{F42EE2D3-909F-4907-8871-4C22FC0BF756}" -Value "$env:userprofile\Documents" -Type ExpandString -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $ushell -Name "{0DDD015D-B06C-45D5-8C4C-F59713854639}" -Value "$env:userprofile\Pictures" -Type ExpandString -ErrorAction SilentlyContinue

        Write-Host "Restarting explorer"
        taskkill.exe /F /IM "explorer.exe" 2>$null
        Start-Process "explorer.exe"

        Write-Host "Waiting for explorer to complete loading"
        Start-Sleep 5

        Write-Host "Please Note - The OneDrive folder at $OneDrivePath may still have items in it. You must manually delete it, but all files should already be copied to the base user folder." -ForegroundColor Yellow
        Write-Host "If there are files missing afterwards, please Login to Onedrive.com and download them manually" -ForegroundColor Yellow

        Write-Output "✅ OneDrive removal completed (files moved where possible)."
    } else {
        Write-Host "Something went wrong during the uninstallation of OneDrive" -ForegroundColor Red
        Write-Output "❌ OneDrive uninstall may have failed."
    }
    "#;
    crate::utils::run_powershell(ps)
}

/// Install (restore) OneDrive via winget (Undo)
pub fn install_onedrive() -> String {
    let ps = r#"
    Write-Host "Install OneDrive via winget"
    try {
        Start-Process -FilePath winget -ArgumentList "install -e --accept-source-agreements --accept-package-agreements --silent Microsoft.OneDrive" -NoNewWindow -Wait
        Write-Output "✅ OneDrive installation (winget) completed."
    } catch {
        Write-Output "❌ OneDrive installation failed: $_"
    }
    "#;
    crate::utils::run_powershell(ps)
}

/// Download and run OO Shutup 10 (Invoke-WebRequest -> save to %temp% -> Start-Process)
/// Note: downloads an executable from the internet and runs it. This may trigger AV/SmartScreen.
/// Requires network access; elevation may be requested by the executable itself.
pub fn run_ooshutup10() -> String {
    let ps = r#"
    try {
        $OOSU_filepath = Join-Path $env:TEMP 'OOSU10.exe'
        $Initial_ProgressPreference = $ProgressPreference
        $ProgressPreference = 'SilentlyContinue' # speed up Invoke-WebRequest
        Write-Host 'Downloading OO Shutup 10 to' $OOSU_filepath
        Invoke-WebRequest -Uri 'https://dl5.oo-software.com/files/ooshutup10/OOSU10.exe' -OutFile $OOSU_filepath -UseBasicParsing -ErrorAction Stop
        Write-Host 'Download complete. Starting OO Shutup 10...'
        Start-Process -FilePath $OOSU_filepath -WorkingDirectory $env:TEMP
        Write-Output '✅ OO Shutup 10 launched. Please follow the application UI to apply settings.'
    } catch {
        Write-Output ('⚠ ERROR downloading or launching OO Shutup 10: ' + $_.Exception.Message)
    } finally {
        $ProgressPreference = $Initial_ProgressPreference
    }
    "#;
    crate::utils::run_powershell(ps)
}

/// Reads HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarAl
/// Returns true when value == 1 (center), false otherwise.
pub fn get_taskbar_alignment() -> bool {
    let ps = r#"
        try {
            $v = Get-ItemPropertyValue -Path 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'TaskbarAl' -ErrorAction Stop
            if ($v -eq 1) { Write-Output '1' } else { Write-Output '0' }
        } catch {
            Write-Output '0'
        }
        "#;
    let out = crate::utils::run_powershell(ps);
    out.lines().rev().find(|s| !s.trim().is_empty()).unwrap_or("0").trim() == "1"
}

/// Set Taskbar alignment robustly: try Set-ItemProperty, fallback to reg.exe if needed.
/// DOES NOT attempt to create the key with New-Item (avoids permission errors).
pub fn set_taskbar_alignment_center(enabled: bool) -> String {
    let value = if enabled { 1 } else { 0 };
    let state = if enabled { "Center" } else { "Left" };

    let ps = format!(r#"
        try {{
            Write-Host 'Setting Taskbar alignment to: {state}'
            $Path = 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced'

            # Try direct PowerShell registry write first (no New-Item)
            try {{
                Set-ItemProperty -Path $Path -Name 'TaskbarAl' -Value {value} -Type DWord -Force -ErrorAction Stop
            }} catch {{
                Write-Host 'Set-ItemProperty failed, attempting reg.exe fallback...' -ForegroundColor Yellow
                $regPath = 'HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced'
                reg add "$regPath" /v TaskbarAl /t REG_DWORD /d {value} /f > $null
            }}

            Write-Output '✅ Taskbar alignment set to {state}. Log off or restart Explorer for full effect.'
        }} catch {{
            Write-Output ('⚠ ERROR: Failed to set Taskbar alignment: ' + $_.Exception.Message)
        }}
        "#, state = state, value = value);

    crate::utils::run_powershell(&ps)
}

pub fn enable_center_taskbar() -> String { set_taskbar_alignment_center(true) }
pub fn disable_center_taskbar() -> String { set_taskbar_alignment_center(false) }

use std::{fs, env, time::SystemTime};

pub fn show_hidden_files(enabled: bool) -> Result<String, io::Error> {
    // Scriptul folosește argument pozițional ($args[0]) -> "True" / "False"
    // Setează Hidden = 1 (show) sau 2 (don't show). Apoi forțează reîmprospătare Explorer.
    let lines = [
        r#"# Eoliann: Toggle show hidden files - positional arg: True/False"#,
        r#"$raw = if ($args.Length -gt 0) { $args[0] } else { 'False' }"#,
        r#"$IsEnabled = $false"#,
        r#"if ($raw -eq 'True' -or $raw -eq 'true' -or $raw -eq '$true' -or $raw -eq '1' -or $raw -eq 1) { $IsEnabled = $true }"#,
        r#""#,
        r#"function Invoke-ShowHiddenFiles {"#,
        r#"    Param($Enabled)"#,
        r#"    try {"#,
        r#"        # Use correct values: 1 = show hidden, 2 = do not show"#,
        r#"        if ($Enabled) { Write-Host 'Enabling Hidden Files'; $value = 1 } else { Write-Host 'Disabling Hidden Files'; $value = 2 }"#,
        r#"        $Path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced'"#,
        r#"        Set-ItemProperty -Path $Path -Name Hidden -Value $value -Type DWord -ErrorAction Stop"#,
        r#"    } catch [System.Security.SecurityException] {"#,
        r#"        Write-Warning 'Unable to set registry due to a Security Exception'"#,
        r#"    } catch [System.Management.Automation.ItemNotFoundException] {"#,
        r#"        Write-Warning $psitem.Exception.ErrorRecord"#,
        r#"    } catch {"#,
        r#"        Write-Warning 'Unhandled exception while setting Hidden'"#,
        r#"        Write-Warning $_.Exception.StackTrace"#,
        r#"    }"#,
        r#"}"#,
        r#""#,
        r#"Invoke-ShowHiddenFiles $IsEnabled"#,
        r#""#,
        r#"# --- Refresh open Explorer windows ---"#,
        r#"$shell = New-Object -ComObject Shell.Application"#,
        r#"try { $shell.Windows() | ForEach-Object { try { $_.Refresh() } catch {} } } catch {}"#,
        r#"try { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($shell) | Out-Null } catch {}"#,
        r#""#,
        r#"# --- Notify shell of association/setting change via SHChangeNotify ---"#,
        r#"$sig = @""#,
        r#"using System;"#,
        r#"using System.Runtime.InteropServices;"#,
        r#"public static class NativeMethods {"#,
        r#"    [DllImport("shell32.dll")]"#,
        r#"    public static extern void SHChangeNotify(long wEventId, uint uFlags, IntPtr dwItem1, IntPtr dwItem2);"#,
        r#"}"#,
        r#""@"#,
        r#"try {"#,
        r#"    Add-Type $sig -ErrorAction SilentlyContinue"#,
        r#"    # SHCNE_ASSOCCHANGED = 0x08000000, SHCNF_FLUSH = 0x1000"#,
        r#"    [NativeMethods]::SHChangeNotify(0x08000000, 0x1000, [IntPtr]::Zero, [IntPtr]::Zero)"#,
        r#"} catch {}"#,
        r#""#,
        r#"Write-Output 'DONE'"#,
    ];

    let script = lines.join("\r\n");

    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let tmp_path = env::temp_dir().join(format!("eoliann_show_hidden_{}.ps1", now));

    fs::write(&tmp_path, script)?;

    // debug: useful to inspect generated script if anything fail
    println!("Script path: {:?}", tmp_path);

    // pass positional string argument (no -Enabled switch)
    let enabled_arg = if enabled { "True" } else { "False" };

    let output = std::process::Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&tmp_path)
        .arg(enabled_arg) // positional argument -> $args[0] in script
        .output()?;

    // cleanup temp file
    let _ = fs::remove_file(&tmp_path);

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        if !stdout.is_empty() { Ok(stdout) } else { Ok(stderr) }
    } else {
        let mut msg = stderr;
        if msg.is_empty() { msg = stdout; }
        Err(io::Error::new(io::ErrorKind::Other, msg))
    }
}

/// Toggle show/hide file extensions for known file types (HideFileExt).
pub fn show_file_extensions(show: bool) -> Result<String, std::io::Error> {
    use std::{env, fs, time::SystemTime};
    use std::io;
    use std::process::Command;

    let ps = r#"
        param([string]$ShowRaw)
        $Show = $false
        if ($null -eq $ShowRaw -or [string]::IsNullOrWhiteSpace($ShowRaw)) {
            $Show = $true
        } else {
            $s = $ShowRaw.Trim()
            if ($s -in @('True','true','1','Yes','yes')) { $Show = $true } else { $Show = $false }
        }
        try {
            $value = if ($Show) { 0 } else { 1 }
            $Path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced'
            Set-ItemProperty -Path $Path -Name HideFileExt -Value $value -ErrorAction Stop
            try {
                $shell = New-Object -ComObject Shell.Application
                $shell.Windows() | ForEach-Object { try { $_.Refresh() } catch {} }
                [Runtime.InteropServices.Marshal]::ReleaseComObject($shell) | Out-Null
            } catch {}
            try {
                $sig = @"
        using System;
        using System.Runtime.InteropServices;
        public static class Native {
            [DllImport("shell32.dll")]
            public static extern void SHChangeNotify(int wEventId, uint uFlags, IntPtr dwItem1, IntPtr dwItem2);
        }
        "@
                Add-Type $sig -ErrorAction SilentlyContinue
                [Native]::SHChangeNotify(0x8000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)
            } catch {}
            Write-Output ("OK: HideFileExt set to " + $value)
        } catch [System.UnauthorizedAccessException] {
            Write-Output ("ERROR: Permission denied - " + $_.Exception.Message)
            exit 2
        } catch {
            Write-Output ("ERROR: " + $_.Exception.Message)
            exit 1
        }
        "#;

    // write temporary script
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let tmp = env::temp_dir().join(format!("eoliann_show_ext_{}.ps1", now));
    fs::write(&tmp, ps)?;

    let arg = if show { "True" } else { "False" };

    let try_exec = |exe: &str| -> Result<(bool, String, String), io::Error> {
        let output = Command::new(exe)
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&tmp)
            .arg(arg)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok((output.status.success(), stdout, stderr))
    };

    let res = match try_exec("powershell") {
        Ok((true, out, _)) if !out.is_empty() => Ok(out),
        Ok((true, out, err)) if out.is_empty() && !err.is_empty() => Ok(err),
        Ok((true, out, _)) => Ok(if out.is_empty() { "OK".to_string() } else { out }),
        Ok((false, _, err)) if !err.is_empty() => Err(io::Error::new(io::ErrorKind::Other, err)),
        Ok((false, out, _)) if !out.is_empty() => Err(io::Error::new(io::ErrorKind::Other, out)),
        Ok((false, _, _)) => {
            // try pwsh fallback
            match try_exec("pwsh") {
                Ok((true, out, _)) => Ok(if out.is_empty() { "OK".to_string() } else { out }),
                Ok((false, _, err2)) => Err(io::Error::new(io::ErrorKind::Other, format!("powershell failed and pwsh failed: {}", err2))),
                Err(e2) => Err(io::Error::new(io::ErrorKind::Other, format!("powershell failed and pwsh spawn error: {}", e2))),
            }
        }
        Err(e) => {
            // powershell spawn error, try pwsh
            match try_exec("pwsh") {
                Ok((true, out, _)) => Ok(if out.is_empty() { "OK".to_string() } else { out }),
                Ok((false, _, err2)) => Err(io::Error::new(io::ErrorKind::Other, format!("powershell spawn error: {}; pwsh failed: {}", e, err2))),
                Err(e2) => Err(io::Error::new(io::ErrorKind::Other, format!("powershell spawn error: {}; pwsh spawn error: {}", e, e2))),
            }
        }
    };

    let _ = fs::remove_file(&tmp);
    res
}

/// Reset Windows Update. Runs a PowerShell script that attempts to repair Windows Update.
/// If `aggressive` is true the script will take additional, potentially slow or risky steps.
pub fn start_reset_windows_update(aggressive: bool, tx: std::sync::mpsc::Sender<String>) -> Result<(), String> {
    // script PowerShell mai verbos care emite PROG:... la etape importante
    let script_lines = vec![
        r#"function Write-Log([string]$m) { Write-Output (\"[WUF] \" + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss') + \" - \" + $m) }"#,
        r#"$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)"#,
        r#"if (-not $isAdmin) { Write-Log 'NOT_ELEVATED'; Write-Output 'PROG:0:NOT_ELEVATED'; exit 2 }"#,
        r#"param([bool]$AggressiveFromCLI)"#,
        r#"Write-Log 'Starting Reset Windows Update...'; Write-Output 'PROG:1:Starting Reset Windows Update'"#,
        r#"if ($AggressiveFromCLI) { Write-Output 'PROG:2:Aggressive mode enabled' } else { Write-Output 'PROG:2:Non-aggressive mode' }"#,
        r#"Write-Output 'PROG:5:Stopping services...'"#,
        r#"foreach ($svc in @('BITS','wuauserv','appidsvc','cryptsvc')) { Write-Output ('PROG:7:Stopping ' + $svc); try { Stop-Service -Name $svc -Force -ErrorAction Stop; Write-Output ('PROG:9:' + $svc + ' stopped') } catch { Write-Output ('PROG:9:' + $svc + ' stop failed: ' + $_) } }"#,
        r#"Write-Output 'PROG:12:Removing QMGR data files...'; try { Remove-Item -Path \"$env:allusersprofile\\Application Data\\Microsoft\\Network\\Downloader\\qmgr*.dat\" -ErrorAction Stop; Write-Output 'PROG:15:QMGR removed' } catch { Write-Output ('PROG:15:QMGR remove error: ' + $_) }"#,
        r#"if ($AggressiveFromCLI) { Write-Output 'PROG:20:Renaming DataStore and Catroot2 (aggressive)'; try { Rename-Item -Path \"$env:systemroot\\SoftwareDistribution\\DataStore\" -NewName 'DataStore.bak' -ErrorAction Stop; Write-Output 'PROG:25:DataStore renamed' } catch { Write-Output ('PROG:25:DataStore rename: ' + $_) } ; try { Rename-Item -Path \"$env:systemroot\\System32\\Catroot2\" -NewName 'catroot2.bak' -ErrorAction Stop; Write-Output 'PROG:28:catroot2 renamed' } catch { Write-Output ('PROG:28:catroot2 rename: ' + $_) } }"#,
        r#"Write-Output 'PROG:30:Renaming Download folder'; try { Rename-Item -Path \"$env:systemroot\\SoftwareDistribution\\Download\" -NewName 'Download.bak' -ErrorAction Stop; Write-Output 'PROG:35:Download renamed' } catch { Write-Output ('PROG:35:Download rename: ' + $_) }"#,
        r#"Write-Output 'PROG:38:Removing WindowsUpdate.log'; try { Remove-Item -Path \"$env:systemroot\\WindowsUpdate.log\" -ErrorAction Stop; Write-Output 'PROG:40:WindowsUpdate.log removed' } catch { Write-Output ('PROG:40:WindowsUpdate.log remove: ' + $_) }"#,
        r#"Write-Output 'PROG:45:Registering DLLs (may be slow)'; $old=Get-Location; Set-Location $env:systemroot\\system32; $DLLs = @('atl.dll','urlmon.dll','mshtml.dll','shdocvw.dll','browseui.dll','jscript.dll','vbscript.dll','scrrun.dll','msxml.dll','msxml3.dll','msxml6.dll'); $i=0; foreach($dll in $DLLs){ $i++; $p = 45 + [int](40 * ($i / $DLLs.Count)); Write-Output ('PROG:' + $p + ':regsvr32 ' + $dll); try { & regsvr32.exe /s $dll } catch { Write-Output ('PROG:' + $p + ':regsvr32 error ' + $dll + ' -> ' + $_) } } ; Set-Location $old; Write-Output 'PROG:85:DLL register done'"#,
        r#"if (Test-Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate') { Write-Output 'PROG:86:Removing WSUS client settings'; foreach($v in @('AccountDomainSid','PingID','SusClientId')) { try { & REG DELETE \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\" /v $v /f } catch { Write-Output ('PROG:87:REG delete ' + $v + ' error: ' + $_) } } }"#,
        r#"Write-Output 'PROG:90:Resetting WinSock and IP stack'; try { & netsh winsock reset } catch { Write-Output ('PROG:90:netsh winsock error: ' + $_) } ; try { & netsh winhttp reset proxy } catch { Write-Output ('PROG:92:netsh winhttp error: ' + $_) } ; try { & netsh int ip reset } catch { Write-Output ('PROG:94:netsh int ip error: ' + $_) }"#,
        r#"Write-Output 'PROG:96:Removing BITS jobs'; try {
            $bitsJobs = Get-BitsTransfer -ErrorAction SilentlyContinue
            if ($bitsJobs -and $bitsJobs.Count -gt 0) {
                foreach ($job in $bitsJobs) {
                    try {
                        Remove-BitsTransfer -BitsJob $job -ErrorAction SilentlyContinue
                        Write-Output ("PROG:96:Removed BITS job: " + ($job.JobId -as [string]))
                    } catch {
                        Write-Output ("PROG:96:Remove-BitsTransfer error: " + $_.ToString())
                    }
                }
            } else {
                Write-Output "PROG:96:No BITS jobs found"
            }
        } catch {
            Write-Output ("PROG:96:Get-BitsTransfer error: " + $_.ToString())
        }"#,
        r#"Write-Output 'PROG:98:Starting services'; Try { Get-Service BITS | Set-Service -StartupType Manual -PassThru | Start-Service } Catch { Write-Output ('PROG:98:BITS start error: ' + $_) } ; Try { Get-Service wuauserv | Set-Service -StartupType Manual -PassThru | Start-Service } Catch { Write-Output ('PROG:99:wuauserv start error: ' + $_) }"#,
        r#"Write-Output 'PROG:100:Forcing Update detection'; Try { (New-Object -ComObject Microsoft.Update.AutoUpdate).DetectNow() } Catch { Write-Output ('PROG:100:DetectNow error: ' + $_) }"#,
        r#"Write-Output 'PROG:100:Completed Reset Windows Update'; Write-Output 'DONE'"#,
    ];

    let script = script_lines.join("\r\n");
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_err(|e| format!("time err: {}", e))?.as_millis();
    let tmp = env::temp_dir().join(format!("eoliann_reset_wu_{}.ps1", now));

    fs::write(&tmp, script).map_err(|e| format!("Eroare la scriere script: {}", e))?;

    let aggressive_arg = if aggressive { "True" } else { "False" };

    let mut child = StdCommand::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&tmp)
        .arg("-AggressiveFromCLI")
        .arg(aggressive_arg)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Eroare la spawn powershell: {}", e))?;

    // read stdout
    if let Some(out) = child.stdout.take() {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                let _ = tx_clone.send(line);
            }
        });
    }

    // read stderr
    if let Some(err) = child.stderr.take() {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                let _ = tx_clone.send(format!("ERR: {}", line));
            }
        });
    }

    // wait for process to finish
    let status = child.wait().map_err(|e| format!("Wait error: {}", e))?;

    // final message
    let _ = tx.send(format!("PROCESS_EXIT:{}", status.code().unwrap_or(-1)));
    // cleanup
    let _ = fs::remove_file(&tmp);
    Ok(())
}

