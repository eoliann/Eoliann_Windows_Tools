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
