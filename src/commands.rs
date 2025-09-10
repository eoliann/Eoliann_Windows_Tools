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
