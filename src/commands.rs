use crate::utils::run_command;

use std::process::Command;

pub fn toggle_context_menu() -> String {
    // Verificăm dacă cheia există
    let check = Command::new("reg")
        .args(&["query", r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}"])
        .output();

    let mut message = String::new();

    match check {
        Ok(output) if output.status.success() => {
            // cheia există -> ștergem pentru a reveni la meniul Win11
            let result = Command::new("reg")
                .args(&["delete", r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}", "/f"])
                .output();

            if let Ok(out) = result {
                if out.status.success() {
                    message.push_str("✅ Switched to Windows 11 context menu.\n");
                } else {
                    return String::from_utf8_lossy(&out.stderr).to_string();
                }
            }
        }
        _ => {
            // cheia nu există -> adăugăm pentru meniul clasic
            let result = Command::new("reg")
                .args(&[
                    "add",
                    r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
                    "/ve", "/t", "REG_SZ", "/d", "", "/f"
                ])
                .output();

            if let Ok(out) = result {
                if out.status.success() {
                    message.push_str("✅ Switched to Classic context menu.\n");
                } else {
                    return String::from_utf8_lossy(&out.stderr).to_string();
                }
            }
        }
    }

    // Restartăm Explorer pentru aplicarea instantă
    let _ = Command::new("taskkill").args(&["/f", "/im", "explorer.exe"]).output();
    let _ = Command::new("explorer.exe").spawn();

    message.push_str("🔄 Explorer restarted. Changes applied instantly!");
    message
}


pub fn disk_cleanup() -> String {
    run_command("cleanmgr.exe /verylowdisk")
}

pub fn quick_access_settings(section: &str) -> String {
    run_command(&format!("explorer ms-settings:{}", section))
}

pub fn network_reset() -> String {
    let mut output = String::new();
    output.push_str(&run_command("netsh winsock reset"));
    output.push_str(&run_command("ipconfig /flushdns"));
    output.push_str(&run_command("ipconfig /release"));
    output.push_str(&run_command("ipconfig /renew"));
    output
}

pub fn power_plan_switcher(guid: &str) -> String {
    run_command(&format!("powercfg /setactive {}", guid))
}

pub fn change_theme(mode: &str) -> String {
    match mode {
        "dark" => run_command("reg add HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize /v AppsUseLightTheme /t REG_DWORD /d 0 /f"),
        "light" => run_command("reg add HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize /v AppsUseLightTheme /t REG_DWORD /d 1 /f"),
        _ => "Invalid mode".to_string(),
    }
}

pub fn disable_sleep() -> String {
    let cmds = [
        "powercfg /change standby-timeout-ac 0",
        "powercfg /change standby-timeout-dc 0",
        "powercfg /change hibernate-timeout-ac 0",
        "powercfg /change hibernate-timeout-dc 0",
    ];

    let mut results = String::new();
    for cmd in cmds {
        results.push_str(&format!("{}\n", crate::utils::run_command(cmd)));
    }
    "✅ Sleep & Hibernate disabled (AC + DC)".to_string()
}

pub fn disable_hdd() -> String {
    let cmds = [
        "powercfg /change disk-timeout-ac 0",
        "powercfg /change disk-timeout-dc 0",
    ];

    let mut results = String::new();
    for cmd in cmds {
        results.push_str(&format!("{}\n", crate::utils::run_command(cmd)));
    }
    "✅ HDD/SSD turn off disabled (AC + DC)".to_string()
}

pub fn disable_monitor() -> String {
    let cmds = [
        "powercfg /change monitor-timeout-ac 0",
        "powercfg /change monitor-timeout-dc 0",
    ];

    let mut results = String::new();
    for cmd in cmds {
        results.push_str(&format!("{}\n", crate::utils::run_command(cmd)));
    }
    "✅ Monitor turn off disabled (AC + DC)".to_string()
}

use std::time::Instant;

pub fn remove_app(package: &str) -> String {
    // Remove for current user
    let user_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxPackage '{}' | Remove-AppxPackage\"",
        package
    );
    let result_user = crate::utils::run_command(&user_cmd);

    // Remove provisioned (system-wide)
    let system_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxProvisionedPackage -Online | Where-Object DisplayName -like '{}' | Remove-AppxProvisionedPackage -Online\"",
        package
    );
    let result_system = crate::utils::run_command(&system_cmd);

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

pub fn remove_app_force(package: &str) -> String {
    // Step 1: User-level
    let user_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxPackage '{}' | Remove-AppxPackage\"",
        package
    );
    let result_user = crate::utils::run_command(&user_cmd);

    // Step 2: Provisioned
    let system_cmd = format!(
        "powershell -ExecutionPolicy Unrestricted -Command \"Get-AppxProvisionedPackage -Online | Where-Object DisplayName -like '{}' | Remove-AppxProvisionedPackage -Online\"",
        package
    );
    let result_system = crate::utils::run_command(&system_cmd);

    // Step 3: DISM Force Remove
    let dism_cmd = format!(
        "dism /Online /Remove-ProvisionedAppxPackage /PackageName:{}",
        package
    );
    let result_dism = crate::utils::run_command(&dism_cmd);

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

pub fn remove_all_apps(force: bool) -> String {
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
