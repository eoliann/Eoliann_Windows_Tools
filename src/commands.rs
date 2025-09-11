use crate::utils::run_command; // păstrează utilitarul tău existent

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(windows)]
use std::os::windows::process::CommandExt; // .creation_flags pentru a ascunde fereastra

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
        let mut c = Command::new(cmd);
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
    let check = Command::new("reg")
        .args(&["query", r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}"])
        .output();

    let mut message = String::new();

    match check {
        Ok(output) if output.status.success() => {
            // cheia există -> revii la meniul Win11
            let result = Command::new("reg")
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
            let result = Command::new("reg")
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
    let _ = Command::new("taskkill").args(&["/f", "/im", "explorer.exe"]).output();
    let _ = Command::new("explorer.exe").spawn();

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
    let mut cmd = Command::new("powershell");
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
    // Remove for current user
    let user_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxPackage '{}' | Remove-AppxPackage\"",
        package
    );
    let result_user = run_command(&user_cmd);

    // Remove provisioned (system-wide)
    let system_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxProvisionedPackage -Online | Where-Object DisplayName -like '{}' | Remove-AppxProvisionedPackage -Online\"",
        package
    );
    let result_system = run_command(&system_cmd);

    if (result_user.trim().is_empty() || result_user.contains("completed"))
        && (result_system.trim().is_empty() || result_system.contains("completed"))
    {
        format!("✅ {package} removed (User + Provisioned).")
    } else {
        format!(
            "⚠ Attempted removal of {package}\nUser: {}\nSystem: {}",
            result_user.trim(),
            result_system.trim()
        )
    }
}

#[allow(dead_code)]
pub fn remove_app_force(package: &str) -> String {
    // Step 1: User-level
    let user_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxPackage '{}' | Remove-AppxPackage\"",
        package
    );
    let result_user = run_command(&user_cmd);

    // Step 2: Provisioned
    let system_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxProvisionedPackage -Online | Where-Object DisplayName -like '{}' | Remove-AppxProvisionedPackage -Online\"",
        package
    );
    let result_system = run_command(&system_cmd);

    // Step 3: DISM Force Remove (poate necesita numele exact al pachetului în unele cazuri)
    let dism_cmd = format!("dism /Online /Remove-ProvisionedAppxPackage /PackageName:{}", package);
    let result_dism = run_command(&dism_cmd);

    if result_dism.contains("completed")
        || (result_user.trim().is_empty() && result_system.trim().is_empty())
    {
        format!("✅ {package} removed (Force).")
    } else {
        format!(
            "⚠ Attempted force removal of {package}\nUser: {}\nSystem: {}\nDISM: {}",
            result_user.trim(),
            result_system.trim(),
            result_dism.trim()
        )
    }
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
            remove_app_force(package)
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

/// ❌ Disable End Task With Right Click (Undo)
#[allow(dead_code)]
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
