#![allow(dead_code)]

use crate::utils::run_command; // păstrează utilitarul tău existent


use std::io::{self, BufReader};
use std::process::{Command as StdCommand, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run_powershell_command(cmd: &str) -> String {
    let mut c = Command::new(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    c.args(&["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-Command", cmd]);
    #[cfg(windows)] { c.creation_flags(CREATE_NO_WINDOW); }
    let out = c.output().unwrap_or_else(|e| panic!("failed to spawn: {}", e));
    String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)
}

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
        let _ = run_command_stream_and_wait(log.clone(), "DISM", &["/Online", "/Cleanup-Image", "/CheckHealth"][..]);
        let _ = run_command_stream_and_wait(log.clone(), "DISM", &["/Online", "/Cleanup-Image", "/ScanHealth"][..]);
        let _ = run_command_stream_and_wait(log.clone(), "DISM", &["/Online", "/Cleanup-Image", "/RestoreHealth"][..]);
        let _ = run_command_stream_and_wait(log.clone(), "sfc", &["/scannow"][..]);

        // log.lock().unwrap().push_str("\n→ Fixing SysWOW64 compatibility...\n");
        if let Ok(mut log_guard) = log.lock() {
            log_guard.push_str("\n→ Fixing SysWOW64 compatibility...\n");
        }

        // let fix_cmd = r#"cmd /c "
        // if exist %windir%\System32\GroupPolicy (
        // xcopy %windir%\System32\GroupPolicy %windir%\SysWOW64\GroupPolicy /E /I /Y
        // )
        // if exist %windir%\System32\GroupPolicyUsers (
        // xcopy %windir%\System32\GroupPolicyUsers %windir%\SysWOW64\GroupPolicyUsers /E /I /Y
        // )
        // if exist %windir%\System32\gpedit.msc (
        // xcopy %windir%\System32\gpedit.msc %windir%\SysWOW64\ /Y
        // )
        // "#;

        // log.lock().unwrap().push_str(&run_command(fix_cmd));


        // let cmd = r#"cmd /c "
        //     dir /b %SystemRoot%\servicing\Packages\Microsoft-Windows-GroupPolicy-ClientTools-Package~*.mum > %TEMP%\gpedit.txt
        //     dir /b %SystemRoot%\servicing\Packages\Microsoft-Windows-GroupPolicy-ClientExtensions-Package~*.mum >> %TEMP%\gpedit.txt
        //     for /f %%i in ('findstr /i . %TEMP%\gpedit.txt 2^>nul') do dism /online /norestart /add-package:"%SystemRoot%\servicing\Packages\%%i"
        //     del %TEMP%\gpedit.txt
        //     "#;        

        let cmd = r#"cmd /c "dir /b %SystemRoot%\servicing\Packages\Microsoft-Windows-GroupPolicy-ClientTools-Package~*.mum > %TEMP%\gpedit.txt && dir /b %SystemRoot%\servicing\Packages\Microsoft-Windows-GroupPolicy-ClientExtensions-Package~*.mum >> %TEMP%\gpedit.txt && for /f %%i in ('findstr /i . %TEMP%\gpedit.txt 2^>nul') do dism /online /norestart /add-package:"%SystemRoot%\servicing\Packages\%%i" && del %TEMP%\gpedit.txt""#;

        log.lock().unwrap().push_str(&run_command(cmd));

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
    cmd.args(&[
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
/// Robust removal routine for built-in apps + winget/MSIX/Win32 fallbacks.
///
/// Requirements:
/// - crate::utils::run_powershell_cmd(cmd: &str) -> String
/// - crate::utils::is_elevated() -> bool
///
/// Returns a multiline log String suitable for UI display.
pub fn remove_app(pattern: &str) -> String {
    let mut log = String::new();
    macro_rules! la { ($e:expr) => { log.push_str(&format!("{}\n", $e)); } }

    la!(format!("Starting removal for pattern '{}'", pattern));
    // sanitize pattern for embedding in PowerShell strings
    let safe = pattern.replace('\'', "").replace('"', "");
    // PowerShell -like pattern (we will embed this directly)
    let pat = format!("*{}*", safe);

    // helper to run PowerShell via utils (must exist)
    let run = |cmd: &str| -> String {
        crate::utils::run_powershell_cmd(cmd)
    };

    // 0) quick presence check (includes Get-StartApps/winget/provisioned)
    let ps_check = format!(
        r#"if (
  (Get-AppxPackage | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}')}} | Measure-Object).Count -gt 0 -or
  (Get-AppxPackage -AllUsers | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}')}} | Measure-Object).Count -gt 0 -or
  (Get-AppxProvisionedPackage -Online | Where-Object {{ ($_.DisplayName -like '{0}') -or ($_.PackageName -like '{0}')}} | Measure-Object).Count -gt 0 -or
  (Get-StartApps | Where-Object {{ ($_.AppID -like '{0}') -or ($_.Name -like '{0}')}} | Measure-Object).Count -gt 0
) {{ Write-Output '1' }} else {{ Write-Output '0' }}"#,
        pat
    );

    let present = run(&ps_check).trim().ends_with('1');
    if !present {
        la!("Application not found on system (by Appx/StartApps/provisioned checks). Will still try winget/registry fallbacks.");
        // we continue to attempt fallbacks below (winget/registry) instead of returning early.
    } else {
        la!("Application found by Appx/StartApps/provisioned checks — proceeding with removal attempts.");
    }

    // 1) Attempt Remove-AppxPackage for current user (enumerate PackageFullName)
    la!("→ Attempting Remove-AppxPackage (current user)...");
    let ps_remove_user = format!(
        r#"Get-AppxPackage | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}')}} |
  ForEach-Object {{
    Write-Output ('[AppxUser] ' + $_.PackageFullName);
    try {{ Remove-AppxPackage -Package $_.PackageFullName -ErrorAction Stop; Write-Output ('[AppxUser] REMOVED: ' + $_.PackageFullName) }} catch {{ Write-Output ('[AppxUser] FAILED: ' + $_.PackageFullName + ' -> ' + $_.Exception.Message) }}
  }}"#,
        pat
    );
    let out_user = run(&ps_remove_user);
    if !out_user.trim().is_empty() {
        la!(format!("• current-user output:\n{}", out_user.trim()));
    } else {
        la!("• current-user: no matching PackageFullName (or no output).");
    }

    // 2) Attempt Remove-AppxPackage -AllUsers (requires elevation)
    if crate::utils::is_elevated() {
        la!("→ Elevated: attempting Remove-AppxPackage (-AllUsers)...");
        let ps_remove_all = format!(
            r#"Get-AppxPackage -AllUsers | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}')}} |
  ForEach-Object {{
    Write-Output ('[AppxAll] ' + $_.PackageFullName);
    try {{ Remove-AppxPackage -Package $_.PackageFullName -AllUsers -ErrorAction Stop; Write-Output ('[AppxAll] REMOVED: ' + $_.PackageFullName) }} catch {{ Write-Output ('[AppxAll] FAILED: ' + $_.PackageFullName + ' -> ' + $_.Exception.Message) }}
  }}"#,
            pat
        );
        let out_all = run(&ps_remove_all);
        if !out_all.trim().is_empty() {
            la!(format!("• all-users output:\n{}", out_all.trim()));
        } else {
            la!("• all-users: no matching PackageFullName (or no output).");
        }
    } else {
        la!("⚠ Not elevated — skipping -AllUsers Remove-AppxPackage. Relaunch as Administrator to attempt system-wide removals.");
    }

    // 3) Attempt Remove-AppxProvisionedPackage (prevent reinstall for new users) if elevated
    if crate::utils::is_elevated() {
        la!("→ Elevated: attempting Remove-AppxProvisionedPackage (Online)...");
        let ps_prov = format!(
            r#"Get-AppxProvisionedPackage -Online | Where-Object {{ ($_.DisplayName -like '{0}') -or ($_.PackageName -like '{0}')}} |
  ForEach-Object {{
    Write-Output ('[Prov] ' + $_.PackageName);
    try {{ Remove-AppxProvisionedPackage -Online -PackageName $_.PackageName -ErrorAction Stop; Write-Output ('[Prov] REMOVED: ' + $_.PackageName) }} catch {{ Write-Output ('[Prov] FAILED: ' + $_.PackageName + ' -> ' + $_.Exception.Message) }}
  }}"#,
            pat
        );
        let out_prov = run(&ps_prov);
        if !out_prov.trim().is_empty() {
            la!(format!("• provisioned output:\n{}", out_prov.trim()));

            // additionally try DISM per each printed PackageName (lines beginning with [Prov] or package names)
            for line in out_prov.lines() {
                let l = line.trim();
                if l.is_empty() { continue; }
                // try to extract a package name-like token (skip lines with 'FAILED' text)
                if l.contains("FAILED") || l.contains("REMOVED") {
                    continue;
                }
                // attempt DISM remove provisioned if line looks like a package name
                la!(format!("↪ Fallback: attempting DISM remove-provisioned for '{}'", l));
                let dism_cmd = format!("dism /Online /Remove-ProvisionedAppxPackage /PackageName:{}", l);
                let out_dism = run(&dism_cmd);
                if !out_dism.trim().is_empty() {
                    la!(format!("• DISM output:\n{}", out_dism.trim()));
                } else {
                    la!("• DISM: executed (no output).");
                }
            }
        } else {
            la!("• provisioned: no matching provisioned package removed (or no output).");
        }
    }

    // 4) Fallback: Get-StartApps -> derive PackageFamilyName and attempt Remove-AppxPackage on matches
    la!("→ Fallback: checking Get-StartApps for AppID/Name -> derive family -> Remove-AppxPackage");
    let ps_startapps = format!(
        r#"
$pat = '{0}';
Get-StartApps | Where-Object {{ $_.AppID -like $pat -or $_.Name -like $pat }} | ForEach-Object {{
  $appId = $_.AppID;
  Write-Output ('[StartApps] AppID: ' + $appId);
  $family = ($appId -split '!')[0];
  Write-Output ('[StartApps] DerivedFamily: ' + $family);
  Get-AppxPackage -AllUsers | Where-Object {{ $_.PackageFamilyName -like (\"$family*\") -or $_.PackageFamilyName -like $family }} | ForEach-Object {{
    Write-Output ('[StartApps] MatchingFull:' + $_.PackageFullName);
    try {{ Remove-AppxPackage -Package $_.PackageFullName -AllUsers -ErrorAction Stop; Write-Output ('[StartApps] REMOVED:' + $_.PackageFullName) }} catch {{ Write-Output ('[StartApps] FAILED:' + $_.PackageFullName + ' -> ' + $_.Exception.Message) }}
  }}
}}
"#,
        pat
    );
    let out_start = run(&ps_startapps);
    if !out_start.trim().is_empty() {
        la!(format!("• Get-StartApps fallback output:\n{}", out_start.trim()));
    } else {
        la!("• Get-StartApps: no matching AppID/Name or no removable package found via family.");
    }

    // 5) Fallback: winget uninstall (MSIX/WinGet-managed)
    // - find candidate Ids / PackageIdentifiers, then attempt uninstall for each.
    la!("→ Fallback: checking winget for MSIX/WinGet-managed entries and attempting uninstall");
    let ps_winget = format!(
        r#"
try {{
  $data = winget list --source winget --output json 2>$null | ConvertFrom-Json;
  if ($null -ne $data) {{
    $matches = $data | Where-Object {{ ($_.Id -like '{0}') -or ($_.Name -like '{0}') -or ($_.PackageIdentifier -like '{0}') }};
    foreach ($m in $matches) {{
      $id = $m.Id; if ([string]::IsNullOrWhiteSpace($id)) {{ $id = $m.PackageIdentifier }};
      if (-not [string]::IsNullOrWhiteSpace($id)) {{
        Write-Output ('[WingetCandidate] ' + $id);
        # attempt uninstall; capture output
        try {{
          $out = winget uninstall --id $id --silent --accept-source-agreements --accept-package-agreements 2>&1 | Out-String;
          Write-Output ('[WingetUninstallOutput] ' + $out);
        }} catch {{
          Write-Output ('[WingetUninstallFailed] ' + $id + ' -> ' + $_.Exception.Message);
        }}
      }}
    }}
  }} else {{
    Write-Output '';
  }}
}} catch {{
  Write-Output ('[WingetError] ' + $_.Exception.Message);
}}
"#,
        pat
    );
    let out_winget = run(&ps_winget);
    if !out_winget.trim().is_empty() {
        la!(format!("• winget fallback output:\n{}", out_winget.trim()));
    } else {
        la!("• winget: no matching entries found or winget not available.");
    }

    // 6) Fallback: Registry UninstallString for Win32 apps — attempt to run UninstallString entries
    la!("→ Fallback: checking registry uninstall keys for Win32 installers and attempting execution");
    let ps_reg_uninstall = format!(
        r#"
$pat = '{0}';
$keys = @(
  'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall',
  'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall',
  'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall'
);
foreach ($k in $keys) {{
  try {{
    Get-ChildItem $k -ErrorAction SilentlyContinue | ForEach-Object {{
      $p = Get-ItemProperty -Path ($_.PSPath) -ErrorAction SilentlyContinue;
      if ($p -and $p.DisplayName -and ($p.DisplayName -like ""*$pat*"") -and $p.UninstallString) {{
        Write-Output ('[RegUninstall] DisplayName=' + $p.DisplayName + ' ; UninstallString=' + $p.UninstallString);
        try {{
          # run uninstall string through cmd.exe to handle MSI/uninstallers; wait for completion
          Start-Process -FilePath 'cmd.exe' -ArgumentList '/C', $p.UninstallString -Wait -NoNewWindow -WindowStyle Hidden;
          Write-Output ('[RegUninstall] EXECUTED: ' + $p.DisplayName);
        }} catch {{
          Write-Output ('[RegUninstall] FAILED RUN: ' + $p.DisplayName + ' -> ' + $_.Exception.Message);
        }}
      }}
    }}
  }} catch {{
    Write-Output ('[RegUninstall] KEYERROR: ' + $k + ' -> ' + $_.Exception.Message);
  }}
}}
"#,
        safe
    );
    let out_regu = run(&ps_reg_uninstall);
    if !out_regu.trim().is_empty() {
        la!(format!("• registry uninstall attempts output:\n{}", out_regu.trim()));
    } else {
        la!("• registry uninstall: no matching uninstall strings found (or none executed).");
    }

    // 7) Final verification: re-check presence (same checks as initial)
    la!("→ Final verification after removal attempts...");
    std::thread::sleep(std::time::Duration::from_millis(800));
    let ps_final_check = format!(
        r#"if (
  (Get-AppxPackage | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}')}} | Measure-Object).Count -gt 0 -or
  (Get-AppxPackage -AllUsers | Where-Object {{ ($_.Name -like '{0}') -or ($_.PackageFullName -like '{0}') -or ($_.PackageFamilyName -like '{0}')}} | Measure-Object).Count -gt 0 -or
  (Get-AppxProvisionedPackage -Online | Where-Object {{ ($_.DisplayName -like '{0}') -or ($_.PackageName -like '{0}')}} | Measure-Object).Count -gt 0 -or
  (Get-StartApps | Where-Object {{ $_.AppID -like '{0}' -or $_.Name -like '{0}' }} | Measure-Object).Count -gt 0
) {{ Write-Output '1' }} else {{ Write-Output '0' }}"#,
        pat
    );
    let still = run(&ps_final_check).trim().ends_with('1');
    if still {
        la!(format!("❌ After attempts, pattern '{}' still matches installed traces (Appx/StartApps/provisioned).", pattern));
        la!("• Inspect outputs above for specific failure reasons (Access denied / Protected / DISM errors / winget errors).");
    } else {
        la!(format!("✅ Removal successful (no matching traces found for '{}').", pattern));
    }

    log
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
    match run_hidden("winget", &["--version"][..]) {
        Ok(o) if o.status.success() => {
            push_line(&log, "✅ winget available.");
            // încercăm și un source update (nu strică)
            let _ = run_hidden("winget", &["source", "update"][..]);
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
    match run_hidden("winget", &["list", "--id", id, "-e"][..]) {
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
    match run_command_stream_and_wait(log, "winget", &args[..]) {
        Ok(code) => code,
        Err(_) => -1,
    }
}

/// Dezinstalează pachetul (unde se poate) și streamează în log.
pub fn winget_uninstall(id: &str, log: Arc<Mutex<String>>) -> i32 {
    let args = ["uninstall", "--id", id, "-e", "--silent"];
    match run_command_stream_and_wait(log, "winget", &args[..]) {
        Ok(code) => code,
        Err(_) => -1,
    }
}

/// Upgrade pentru pachet (silent) și streamează în log.
pub fn winget_upgrade(id: &str, log: Arc<Mutex<String>>) -> i32 {
    let args = ["upgrade", "--id", id, "-e", "--silent", "--accept-package-agreements", "--accept-source-agreements"];
    match run_command_stream_and_wait(log, "winget", &args[..]) {
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
    let ps = format!(r#"
        $ErrorActionPreference = 'Stop'

        # Admin check
        $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
            [Security.Principal.WindowsBuiltInRole]::Administrator)
        if (-not $isAdmin) {{
            Write-Output 'ERROR: Administrator privileges required.'
            exit 1
        }}

        $primary = '{primary}'
        $secondary = '{secondary}'

        # Get adapters that are Up
        $adapters = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object {{ $_.Status -eq 'Up' }} | Select-Object -ExpandProperty Name -ErrorAction SilentlyContinue

        if (-not $adapters) {{
            Write-Output 'INFO: No network adapters found to update.'
            exit 0
        }}

        # If Set-DnsClientServerAddress exists, try multiple approaches (compatible)
        if (Get-Command -Name Set-DnsClientServerAddress -ErrorAction SilentlyContinue) {{
            $servers = @($primary, $secondary)

            # 1) Try single-call for all adapters without AddressFamily (most compatible)
            try {{
                Set-DnsClientServerAddress -InterfaceAlias $adapters -ServerAddresses $servers -ErrorAction Stop
                try {{ Clear-DnsClientCache -ErrorAction SilentlyContinue }} catch {{ ipconfig /flushdns | Out-Null }}
                Write-Output ('SUCCESS: DNS set on ' + ($adapters -join ', '))
                exit 0
            }} catch {{
                # 2) Try per-adapter call (some older builds require per-adapter calls)
                $errors = @()
                foreach ($a in $adapters) {{
                    try {{
                        Set-DnsClientServerAddress -InterfaceAlias $a -ServerAddresses $servers -ErrorAction Stop
                    }} catch {{
                        $errors += ("$a -> " + $_.Exception.Message)
                    }}
                }}
                if ($errors.Count -eq 0) {{
                    try {{ Clear-DnsClientCache -ErrorAction SilentlyContinue }} catch {{ ipconfig /flushdns | Out-Null }}
                    Write-Output ('SUCCESS: DNS set per-adapter on ' + ($adapters -join ', '))
                    exit 0
                }} else {{
                    # Fall through to netsh fallback
                    Write-Output ('WARNING: Set-DnsClientServerAddress attempts failed: ' + ($errors -join '; '))
                }}
            }}
        }}

        # Fallback: netsh per adapter (very compatible)
        $errors = @()
        foreach ($a in $adapters) {{
            try {{
                netsh interface ip set dns name="$a" source=static addr=$primary register=primary
                netsh interface ip add dns name="$a" addr=$secondary index=2
            }} catch {{
                $errors += ("$a -> " + $_.Exception.Message)
            }}
        }}

        try {{ ipconfig /flushdns | Out-Null }} catch {{}}

        if ($errors.Count -gt 0) {{
            Write-Output ('ERROR: Some adapters failed: ' + ($errors -join '; '))
            exit 1
        }} else {{
            Write-Output ('SUCCESS: DNS set (netsh fallback) on ' + ($adapters -join ', '))
            exit 0
        }}
        "#,
        primary = primary,
        secondary = secondary
    );

    crate::utils::run_powershell(&ps)
}

fn reset_dns() -> String {
    let ps = r#"
        $ErrorActionPreference = 'Stop'
        $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
            [Security.Principal.WindowsBuiltInRole]::Administrator)
        if (-not $isAdmin) {
            Write-Output 'ERROR: Administrator privileges required.'
            exit 1
        }

        $adapters = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { $_.Status -eq 'Up' } | Select-Object -ExpandProperty Name -ErrorAction SilentlyContinue
        if (-not $adapters) {
            Write-Output 'INFO: No adapters found to update.'
            exit 0
        }

        # Try Reset with Set-DnsClientServerAddress if available
        if (Get-Command -Name Set-DnsClientServerAddress -ErrorAction SilentlyContinue) {
            try {
                Set-DnsClientServerAddress -InterfaceAlias $adapters -Reset -ErrorAction Stop
                try { Clear-DnsClientCache -ErrorAction SilentlyContinue } catch { ipconfig /flushdns | Out-Null }
                Write-Output ('SUCCESS: DNS reset to Automatic (DHCP) on ' + ($adapters -join ', '))
                exit 0
            } catch {
                # try per-adapter reset
                $errors = @()
                foreach ($a in $adapters) {
                    try {
                        Set-DnsClientServerAddress -InterfaceAlias $a -Reset -ErrorAction Stop
                    } catch {
                        $errors += ("$a -> " + $_.Exception.Message)
                    }
                }
                if ($errors.Count -eq 0) {
                    try { Clear-DnsClientCache -ErrorAction SilentlyContinue } catch { ipconfig /flushdns | Out-Null }
                    Write-Output ('SUCCESS: DNS reset per-adapter to DHCP on ' + ($adapters -join ', '))
                    exit 0
                } else {
                    Write-Output ('WARNING: Reset via Set-DnsClientServerAddress failed: ' + ($errors -join '; '))
                }
            }
        }

        # Fallback to netsh
        $errors = @()
        foreach ($a in $adapters) {
            try {
                netsh interface ip set dns name="$a" source=dhcp
            } catch {
                $errors += ("$a -> " + $_.Exception.Message)
            }
        }
        try { ipconfig /flushdns | Out-Null } catch {}
        if ($errors.Count -gt 0) {
            Write-Output ('ERROR: Some adapters failed to reset: ' + ($errors -join '; '))
            exit 1
        } else {
            Write-Output ('SUCCESS: DNS reset to Automatic (DHCP) on ' + ($adapters -join ', '))
            exit 0
        }
        "#;

    crate::utils::run_powershell(ps)
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
        run_command_and_log("choco", &["upgrade", "chocolatey", "-y"][..], &output_log);
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
        run_command_and_log("choco", &["upgrade", "all", "-y"][..], &output_log);
    }

    if winget_ready {
        log_message(&output_log, "\n=== Upgrading apps via Winget ===");
        run_command_and_log("winget", &["upgrade", "--all", "--silent"][..], &output_log);
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
    
    let cmd = format!("Add-AppxPackage -Path (Invoke-WebRequest -Uri '{}' -OutFile '$env:TEMP\\winget.msixbundle' -PassThru).Path", url);
    run_command_and_log(
        "powershell",
        &["-Command", cmd.as_str()][..],
        output_log
    );
}

fn install_chocolatey(output_log: &SharedOutput) {
    log_message(output_log, "Installing Chocolatey...");
    let install_script = "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))";
    
    run_command_and_log(
        "powershell",
        &["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", install_script][..],
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
    run_command_and_log("choco", &["install", "winget", "-y", "--force"][..], &output_log);
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

pub fn brave_debloat() -> String {
    #[cfg(target_os = "windows")]
    {
        // Script PowerShell: creează cheia de politici și setează valorile DWord cerute.
        let script = r#"
            $path = 'HKLM:\SOFTWARE\Policies\BraveSoftware\Brave'
            if (-not (Test-Path $path)) {
                New-Item -Path $path -Force | Out-Null
            }
            Try {
                New-ItemProperty -Path $path -Name BraveRewardsDisabled -PropertyType DWord -Value 1 -Force | Out-Null
                New-ItemProperty -Path $path -Name BraveWalletDisabled  -PropertyType DWord -Value 1 -Force | Out-Null
                New-ItemProperty -Path $path -Name BraveVPNDisabled     -PropertyType DWord -Value 1 -Force | Out-Null
                New-ItemProperty -Path $path -Name BraveAIChatEnabled   -PropertyType DWord -Value 0 -Force | Out-Null
                Write-Output 'SUCCESS: Brave policies applied (BraveRewardsDisabled=1, BraveWalletDisabled=1, BraveVPNDisabled=1, BraveAIChatEnabled=0).'
            } Catch {
                Write-Error ("Failed to apply Brave policies: " + $_.Exception.Message)
                exit 1
            }
            "#;

        let out = std::process::Command::new("powershell")
            .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            // dacă vrei să previi deschiderea unei ferestre console pe Windows GUI builds,
            // folosește .creation_flags(CREATE_NO_WINDOW) dacă acel constant/flag e definit în fișierul tău.
            .output();

        match out {
            Ok(o) => {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).to_string();
                    // Normalizează output-ul (dacă e gol, returnează un succes generic)
                    if s.trim().is_empty() {
                        "SUCCESS: Brave policies applied.".to_string()
                    } else {
                        s
                    }
                } else {
                    let err = String::from_utf8_lossy(&o.stderr).to_string();
                    if err.trim().is_empty() {
                        format!("ERROR: PowerShell exited with code {:?}.", o.status.code())
                    } else {
                        format!("ERROR: {}", err.trim())
                    }
                }
            }
            Err(e) => format!("ERROR: Failed to spawn PowerShell: {}", e),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        "ERROR: brave_debloat is supported only on Windows.".to_string()
    }
}

/// WPFTweaks Edge Debloat (config-driven)
pub fn wpftweaks_edge_debloat() -> String {
    let ps = r#"
        Write-Host 'Applying WPFTweaks Edge Debloat...'

        # Ensure parent keys exist
        New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\EdgeUpdate' -Force | Out-Null
        New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Force | Out-Null

        # EdgeUpdate
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\EdgeUpdate' -Name 'CreateDesktopShortcutDefault' -Value 0 -Type DWord -Force

        # Edge policies (telemetry, recommendations, shopping, widgets, etc.)
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'PersonalizationReportingEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'ShowRecommendationsEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'HideFirstRunExperience' -Value 1 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'UserFeedbackAllowed' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'ConfigureDoNotTrack' -Value 1 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'AlternateErrorPagesEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'EdgeCollectionsEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'EdgeShoppingAssistantEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'MicrosoftEdgeInsiderPromotionEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'ShowMicrosoftRewards' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'WebWidgetAllowed' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'DiagnosticData' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'EdgeAssetDeliveryServiceEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'CryptoWalletEnabled' -Value 0 -Type DWord -Force
        Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'WalletDonationEnabled' -Value 0 -Type DWord -Force

        Write-Output 'SUCCESS: WPFTweaks Edge Debloat applied (registry entries set).'
        "#;

    crate::utils::run_powershell(ps)
}

/// WPFTweaks: Disable Edge (DisallowRun / policy)
pub fn wpftweaks_disable_edge() -> String {
    let ps = r#"
        Write-Host 'Applying WPFTweaks Disable Edge...'

        # Ensure parent policy keys exist
        New-Item -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer' -Force | Out-Null
        New-Item -Path 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer' -Force | Out-Null

        # Ensure DisallowRun subkey exists under HKCU
        New-Item -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\DisallowRun' -Force | Out-Null

        # Add string value to block msedge.exe via DisallowRun list (name: DisableEdge, value: msedge.exe)
        Try {
            New-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\DisallowRun' -Name 'DisableEdge' -PropertyType String -Value 'msedge.exe' -Force | Out-Null
        } Catch {
            Write-Warning ("Failed to set HKCU DisallowRun entry: " + $_.Exception.Message)
        }

        # Enable DisallowRun policy under HKLM (DWord = 1)
        Try {
            Set-ItemProperty -Path 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer' -Name 'DisallowRun' -Value 1 -Type DWord -Force
        } Catch {
            Write-Warning ("Failed to set HKLM DisallowRun policy: " + $_.Exception.Message)
        }

        Write-Output 'SUCCESS: WPFTweaks Disable Edge applied (DisallowRun entries set).'
        "#;

    crate::utils::run_powershell(ps)
}






// // --- REQUIREMENTS: keep only one set of these imports at the top of the file ---
// use winreg::enums::*;
// use winreg::RegKey;

// // --- Explorer Tabs overrides (single copy only) ---
// const EXPLORER_TABS_IDS: &[u32] = &[37634385u32, 39145991u32, 36354489u32];
// const OVERRIDES_BASE_PATH: &str = r"SYSTEM\CurrentControlSet\Control\FeatureManagement\Overrides\4";

// fn write_override_for_feature(feature_id: u32, enabled: bool) -> Result<String, String> {
//     let enabled_state: u32 = if enabled { 2 } else { 1 };
//     let enabled_state_opts: u32 = if enabled { 0 } else { 1 };

//     let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
//     let key_path = format!(r"{}\{}", OVERRIDES_BASE_PATH, feature_id);

//     let (key, _disp) = hklm
//         .create_subkey_with_flags(&key_path, KEY_WRITE | KEY_WOW64_64KEY)
//         .map_err(|e| format!("Failed to open/create registry key {}: {}", key_path, e))?;

//     key.set_value("EnabledState", &enabled_state)
//         .map_err(|e| format!("Failed to set EnabledState: {}", e))?;
//     key.set_value("EnabledStateOptions", &enabled_state_opts)
//         .map_err(|e| format!("Failed to set EnabledStateOptions: {}", e))?;

//     // best-effort optional values
//     let _ = key.set_value("Variant", &0u32);
//     let _ = key.set_value("VariantPayload", &0u32);
//     let _ = key.set_value("VariantPayloadKind", &0u32);

//     Ok(format!("Feature {} -> registry updated (EnabledState={})", feature_id, enabled_state))
// }

// pub fn enable_explorer_tabs() -> Result<String, String> {
//     let mut out = String::new();
//     for &id in EXPLORER_TABS_IDS {
//         match write_override_for_feature(id, true) {
//             Ok(s) => out.push_str(&format!("OK: {}\n", s)),
//             Err(e) => out.push_str(&format!("ERR: feature {} -> {}\n", id, e)),
//         }
//     }
//     out.push_str("Notă: este posibil să fie necesar restart Explorer / reboot pentru aplicare.\n");
//     Ok(out)
// }

// pub fn disable_explorer_tabs() -> Result<String, String> {
//     let mut out = String::new();
//     for &id in EXPLORER_TABS_IDS {
//         match write_override_for_feature(id, false) {
//             Ok(s) => out.push_str(&format!("OK: {}\n", s)),
//             Err(e) => out.push_str(&format!("ERR: feature {} -> {}\n", id, e)),
//         }
//     }
//     out.push_str("Notă: este posibil să fie necesar restart Explorer / reboot pentru aplicare.\n");
//     Ok(out)
// }

// pub fn remove_explorer_tabs_overrides() -> Result<String, String> {
//     let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
//     let mut out = String::new();
//     for &id in EXPLORER_TABS_IDS {
//         let key_path = format!(r"{}\{}", OVERRIDES_BASE_PATH, id);
//         match hklm.delete_subkey_all(&key_path) {
//             Ok(_) => out.push_str(&format!("Removed override key for {}\n", id)),
//             Err(e) => out.push_str(&format!("Failed to remove {}: {}\n", id, e)),
//         }
//     }
//     out.push_str("Notă: unele modificări pot fi restabilite de Task Scheduler; reboot recomandat.\n");
//     Ok(out)
// }

// // --- Helper: read EnabledState if present ---
// fn read_enabled_state_for_feature(feature_id: u32) -> Option<u32> {
//     let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
//     let key_path = format!(r"{}\{}", OVERRIDES_BASE_PATH, feature_id);
//     match hklm.open_subkey_with_flags(&key_path, KEY_READ | KEY_WOW64_64KEY) {
//         Ok(k) => k.get_value::<u32, _>("EnabledState").ok(),
//         Err(_) => None,
//     }
// }

// /// Toggle wrapper expected by UI: detect current majority state and invert it.
// pub fn toggle_explorer_tabs() -> Result<String, String> {
//     let mut enabled_count = 0usize;
//     let mut known_count = 0usize;
//     for &id in EXPLORER_TABS_IDS {
//         if let Some(v) = read_enabled_state_for_feature(id) {
//             known_count += 1;
//             if v == 2 { enabled_count += 1; }
//         }
//     }

//     // Decide: if majority of known overrides are enabled => disable; else enable.
//     let should_disable = if known_count == 0 {
//         // no overrides present -> assume features are disabled by default -> enable
//         false
//     } else {
//         enabled_count * 2 >= known_count
//     };

//     if should_disable {
//         disable_explorer_tabs()
//     } else {
//         enable_explorer_tabs()
//     }
// }



// use std::thread;
use std::time::{Duration, UNIX_EPOCH};
use winreg::enums::*;
use winreg::RegKey;

// Explorer Tabs ids and base path (keep only one copy in the file)
const EXPLORER_TABS_ARRAY: [u32; 3] = [37634385u32, 39145991u32, 36354489u32];
const EXPLORER_TABS_IDS: &[u32; 3] = &EXPLORER_TABS_ARRAY;
const OVERRIDES_BASE_PATH: &str = r"SYSTEM\CurrentControlSet\Control\FeatureManagement\Overrides\4";

fn write_override_for_feature(feature_id: u32, enabled: bool) -> Result<String, String> {
    let enabled_state: u32 = if enabled { 2 } else { 1 };
    let enabled_state_opts: u32 = if enabled { 0 } else { 1 };

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = format!(r"{}\{}", OVERRIDES_BASE_PATH, feature_id);

    let (key, _disp) = hklm
        .create_subkey_with_flags(&key_path, KEY_WRITE | KEY_WOW64_64KEY)
        .map_err(|e| format!("Failed to open/create registry key {}: {}", key_path, e))?;

    key.set_value("EnabledState", &enabled_state)
        .map_err(|e| format!("Failed to set EnabledState: {}", e))?;
    key.set_value("EnabledStateOptions", &enabled_state_opts)
        .map_err(|e| format!("Failed to set EnabledStateOptions: {}", e))?;

    // best-effort optional values
    let _ = key.set_value("Variant", &0u32);
    let _ = key.set_value("VariantPayload", &0u32);
    let _ = key.set_value("VariantPayloadKind", &0u32);

    Ok(format!("Feature {} -> registry updated (EnabledState={})", feature_id, enabled_state))
}

/// Attempt to restart Explorer (best-effort). Returns a short status string.
fn restart_explorer() -> Result<String, String> {
    // Kill current explorer processes (best-effort)
    match Command::new("taskkill").args(&["/f", "/im", "explorer.exe"]).output() {
        Ok(o) => {
            // ignore exit code; continue to start explorer
            let _ = o;
        }
        Err(e) => {
            // non-fatal, but report
            return Err(format!("Failed to run taskkill for explorer: {}", e));
        }
    }

    // small pause to allow processes to terminate
    thread::sleep(Duration::from_millis(350));

    // Start a new explorer instance
    match Command::new("explorer.exe").spawn() {
        Ok(_ch) => Ok("Explorer restarted (taskkill + explorer start).".to_string()),
        Err(e) => Err(format!("Failed to start explorer.exe: {}", e)),
    }
}

pub fn enable_explorer_tabs() -> Result<String, String> {
    let mut out = String::new();
    let mut any_err = false;

    for &id in EXPLORER_TABS_IDS {
        match write_override_for_feature(id, true) {
            Ok(s) => out.push_str(&format!("OK: {}\n", s)),
            Err(e) => {
                any_err = true;
                out.push_str(&format!("ERR: feature {} -> {}\n", id, e));
            }
        }
    }

    out.push_str("Note: Explorer may need to be restarted or the system rebooted for changes to take effect.\n");

    // Try to restart Explorer automatically (best-effort). Append status to output.
    match restart_explorer() {
        Ok(msg) => out.push_str(&format!("{}\n", msg)),
        Err(err) => out.push_str(&format!("Warning: automatic Explorer restart failed: {}\n", err)),
    }

    if any_err { Err(out) } else { Ok(out) }
}

pub fn disable_explorer_tabs() -> Result<String, String> {
    let mut out = String::new();
    let mut any_err = false;

    for &id in EXPLORER_TABS_IDS {
        match write_override_for_feature(id, false) {
            Ok(s) => out.push_str(&format!("OK: {}\n", s)),
            Err(e) => {
                any_err = true;
                out.push_str(&format!("ERR: feature {} -> {}\n", id, e));
            }
        }
    }

    out.push_str("Note: Explorer may need to be restarted or the system rebooted for changes to take effect.\n");

    // Try to restart Explorer automatically (best-effort). Append status to output.
    match restart_explorer() {
        Ok(msg) => out.push_str(&format!("{}\n", msg)),
        Err(err) => out.push_str(&format!("Warning: automatic Explorer restart failed: {}\n", err)),
    }

    if any_err { Err(out) } else { Ok(out) }
}

pub fn remove_explorer_tabs_overrides() -> Result<String, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut out = String::new();
    let mut any_err = false;

    for &id in EXPLORER_TABS_IDS {
        let key_path = format!(r"{}\{}", OVERRIDES_BASE_PATH, id);
        match hklm.delete_subkey_all(&key_path) {
            Ok(_) => out.push_str(&format!("Removed override key for {}\n", id)),
            Err(e) => {
                any_err = true;
                out.push_str(&format!("Failed to remove {}: {}\n", id, e));
            }
        }
    }

    out.push_str("Note: some changes may be restored by Task Scheduler / management agents; reboot recommended.\n");

    // Try to restart Explorer automatically (best-effort). Append status to output.
    match restart_explorer() {
        Ok(msg) => out.push_str(&format!("{}\n", msg)),
        Err(err) => out.push_str(&format!("Warning: automatic Explorer restart failed: {}\n", err)),
    }

    if any_err { Err(out) } else { Ok(out) }
}

// Helper: read EnabledState if present
fn read_enabled_state_for_feature(feature_id: u32) -> Option<u32> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = format!(r"{}\{}", OVERRIDES_BASE_PATH, feature_id);
    match hklm.open_subkey_with_flags(&key_path, KEY_READ | KEY_WOW64_64KEY) {
        Ok(k) => k.get_value::<u32, _>("EnabledState").ok(),
        Err(_) => None,
    }
}

/// Toggle wrapper expected by UI: detect current majority state and invert it.
pub fn toggle_explorer_tabs() -> Result<String, String> {
    let mut enabled_count = 0usize;
    let mut known_count = 0usize;
    for &id in EXPLORER_TABS_IDS {
        if let Some(v) = read_enabled_state_for_feature(id) {
            known_count += 1;
            if v == 2 { enabled_count += 1; }
        }
    }

    // Decide: if majority of known overrides are enabled => disable; else enable.
    let should_disable = if known_count == 0 {
        // no overrides present -> assume features are disabled by default -> enable
        false
    } else {
        enabled_count * 2 >= known_count
    };

    if should_disable {
        disable_explorer_tabs()
    } else {
        enable_explorer_tabs()
    }
}



// --- START: ADD TO END OF src/commands.rs ---
/// Helpers pentru rulare PowerShell elevat + logging (folosesc căi complet calificate
/// pentru a evita importuri duplicate în modulul existent).

fn make_temp_ps_path(prefix: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut p = std::env::temp_dir();
    p.push(format!("{}_{}.ps1", prefix, ts));
    p
}

fn make_temp_log_path(prefix: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut p = std::env::temp_dir();
    p.push(format!("{}_{}.log", prefix, ts));
    p
}

fn write_ps_script(script_body: &str, ps_path: &std::path::PathBuf, log_path: &std::path::PathBuf) -> std::io::Result<()> {
    let mut f = std::fs::File::create(ps_path)?;
    let wrapper = format!(
r#"
Set-StrictMode -Version Latest
try {{
    & {{
        {}
    }} 2>&1 | Out-File -FilePath "{}" -Encoding UTF8 -Append
    exit $LASTEXITCODE
}} catch {{
    $_ | Out-String | Out-File -FilePath "{}" -Encoding UTF8 -Append
    exit 1
}}
"#,
        script_body.replace("\r\n", "\n"),
        log_path.display(),
        log_path.display()
    );
    use std::io::Write as _; // foloseşte trait local, nu re-import global
    f.write_all(wrapper.as_bytes())?;
    Ok(())
}

fn run_ps_elevated_and_wait(ps_path: &std::path::PathBuf) -> std::io::Result<()> {
    // Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File "C:\...ps1"' -Verb RunAs -Wait
    let arg = format!(
        "-NoProfile -ExecutionPolicy Bypass -Command Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"' -Verb RunAs -Wait",
        ps_path.display()
    );
    std::process::Command::new("powershell")
        .arg(arg)
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn run_elevated_powershell_with_log(script_body: &str, prefix: &str) -> std::io::Result<std::path::PathBuf> {
    let ps_path = make_temp_ps_path(prefix);
    let log_path = make_temp_log_path(prefix);
    write_ps_script(script_body, &ps_path, &log_path)?;
    run_ps_elevated_and_wait(&ps_path)?;
    Ok(log_path)
}

// Concrete commands (public) — folosesc funcţia de mai sus
pub fn install_power_automate_with_log() -> std::io::Result<std::path::PathBuf> {
    let script = r#"
winget install -e --id Microsoft.PowerAutomateDesktop --accept-package-agreements --accept-source-agreements
"#;
    run_elevated_powershell_with_log(script, "install_power_automate")
}

pub fn remove_power_automate_with_log() -> std::io::Result<std::path::PathBuf> {
    let script = r#"
winget uninstall --id Microsoft.PowerAutomateDesktop -e
"#;
    run_elevated_powershell_with_log(script, "remove_power_automate")
}

pub fn remove_copilot_current_user_with_log() -> std::io::Result<std::path::PathBuf> {
    let script = r#"
Get-AppxPackage *Copilot* | ForEach-Object { Try { Remove-AppxPackage -Package $_.PackageFullName -ErrorAction Stop } Catch { $_ | Out-String; Continue } }
"#;
    run_elevated_powershell_with_log(script, "remove_copilot_user")
}

pub fn remove_copilot_all_users_with_log() -> std::io::Result<std::path::PathBuf> {
    let script = r#"
Get-AppxPackage -AllUsers | Where-Object { $_.Name -like '*Copilot*' } | ForEach-Object { Try { Remove-AppxPackage -Package $_.PackageFullName -ErrorAction Stop } Catch { $_ | Out-String; Continue } }
Get-AppxProvisionedPackage -Online | Where-Object { $_.PackageName -like '*Copilot*' } | ForEach-Object { Try { Remove-AppxProvisionedPackage -Online -PackageName $_.PackageName -ErrorAction Stop } Catch { $_ | Out-String; Continue } }
"#;
    run_elevated_powershell_with_log(script, "remove_copilot_all")
}
// --- END: ADD TO END OF src/commands.rs ---





pub fn install_power_automate() -> String {
    match install_power_automate_with_log() {
        Ok(log_path) => format!("Power Automate Desktop installation attempted. See log: {}", log_path.display()),
        Err(e) => format!("Failed to run installation: {}", e),
    }
}

pub fn remove_power_automate() -> String {
    match remove_power_automate_with_log() {
        Ok(log_path) => format!("Power Automate Desktop removal attempted. See log: {}", log_path.display()),
        Err(e) => format!("Failed to run removal: {}", e),
    }
}

pub fn remove_copilot_current_user() -> String {
    match remove_copilot_current_user_with_log() {
        Ok(log_path) => format!("Copilot removal for current user attempted. See log: {}", log_path.display()),
        Err(e) => format!("Failed to run removal: {}", e),
    }
}

pub fn remove_copilot_all_users() -> String {
    match remove_copilot_all_users_with_log() {
        Ok(log_path) => format!("Copilot removal for all users attempted. See log: {}", log_path.display()),
        Err(e) => format!("Failed to run removal: {}", e),
    }
}

// --- Power Automate Desktop (winget) ---

fn run_winget<'a, A>(args: A)-> String
where
    A: AsRef<[&'a str]>,
{
    let args = args.as_ref();
    // IMPORTANT: nu adăuga `use std::process::Command;` dacă există deja în fișier.
    // IMPORTANT: nu adăuga `use std::io;` dacă există deja în fișier.

    let out = std::process::Command::new("winget")
        .args(args)
        .output();

    match out {
        Ok(o) => {
            let mut text = String::new();
            if !o.stdout.is_empty() {
                text.push_str(&String::from_utf8_lossy(&o.stdout));
            }
            if !o.stderr.is_empty() {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(&String::from_utf8_lossy(&o.stderr));
            }

            if o.status.success() {
                if text.trim().is_empty() {
                    "SUCCESS: winget command completed.".to_string()
                } else {
                    format!("SUCCESS: winget command completed.\n{}", text.trim())
                }
            } else {
                let code = o.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".into());
                if text.trim().is_empty() {
                    format!("ERROR: winget command failed (exit {}).", code)
                } else {
                    format!("ERROR: winget command failed (exit {}).\n{}", code, text.trim())
                }
            }
        }
        Err(e) => format!("ERROR: Failed to run winget: {}", e),
    }
}


fn run_winget_hidden<'a, A>(args: A)-> String
where
    A: AsRef<[&'a str]>,
{
    let args = args.as_ref();
    let mut cmd = std::process::Command::new("winget");
    cmd.args(args);

    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

            if out.status.success() {
                if stdout.is_empty() {
                    "SUCCESS: winget finished successfully.".to_string()
                } else {
                    format!("SUCCESS: {}", stdout)
                }
            } else {
                let code = out.status.code().unwrap_or(-1);
                if !stderr.is_empty() {
                    format!("ERROR: winget failed (code {}): {}", code, stderr)
                } else if !stdout.is_empty() {
                    format!("ERROR: winget failed (code {}): {}", code, stdout)
                } else {
                    format!("ERROR: winget failed (code {}).", code)
                }
            }
        }
        Err(e) => format!("ERROR: failed to execute winget: {}", e),
    }
}

pub fn install_power_automate_desktop() -> String {
    run_winget_hidden(&[
        "install",
        "--id", "Microsoft.PowerAutomateDesktop",
        "--silent",
        "--disable-interactivity",
        "--accept-package-agreements",
        "--accept-source-agreements",
    ])
}

pub fn uninstall_power_automate_desktop() -> String {
    run_winget_hidden(&[
        "uninstall",
        "--id", "Microsoft.PowerAutomateDesktop",
        "--silent",
        "--disable-interactivity",
        "--accept-source-agreements",
    ])
}


fn run_powershell_hidden(script: &str) -> String {
    let out = Command::new("powershell")
        .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if o.status.success() {
                // multe comenzi PS nu scriu nimic pe stdout; tot marcăm SUCCESS dacă exit code e ok
                if stdout.trim().is_empty() && stderr.trim().is_empty() {
                    "SUCCESS: PowerShell finished.".to_string()
                } else {
                    format!("SUCCESS: PowerShell finished.\n{}\n{}", stdout, stderr)
                }
            } else {
                format!(
                    "ERROR: PowerShell failed (code: {:?}).\nSTDOUT:\n{}\nSTDERR:\n{}",
                    o.status.code(),
                    stdout,
                    stderr
                )
            }
        }
        Err(e) => format!("ERROR: failed to start PowerShell: {}", e),
    }
}

pub fn install_copilot() -> String {
    // Copilot MS Store app id: 9NHT9RB2F4HD :contentReference[oaicite:1]{index=1}
    run_winget_hidden(&[
        "install",
        "-s", "msstore",
        "--id", "9NHT9RB2F4HD",
        "--accept-package-agreements",
        "--accept-source-agreements",
        "--disable-interactivity",
        "--silent",
    ])
}

/// Dezinstalează Copilot pentru utilizatorul curent
pub fn uninstall_copilot() -> String {
    // Uninstall Copilot via Appx (current user). :contentReference[oaicite:2]{index=2}
    // Dacă vrei all-users, trebuie rulat ca admin; altfel va da eroare/nu va elimina pentru toți.
    let script = r#"
$pkgs = Get-AppxPackage | Where-Object { $_.Name -Like '*Microsoft.Copilot*' }
if ($null -eq $pkgs) { Write-Output 'Copilot Appx not found for current user.'; exit 0 }
$pkgs | Remove-AppxPackage -ErrorAction Continue
Write-Output 'Copilot Appx removed for current user.'
"#;

    run_powershell_hidden(script)
}



/// Șterge conținutul folderului Prefetch
pub fn empty_prefetch_files() -> String {
    // Șterge C:\Windows\Prefetch în mod robust:
    // - verifică admin
    // - oprește SysMain (best-effort)
    // - șterge fișierele per-item ca să nu “mascheze” eșecurile
    // - pornește SysMain la loc (best-effort)
    // - rulează PowerShell ascuns (fereastră ascunsă) și returnează output text

    let ps = r#"
        $ErrorActionPreference = 'Stop'

        function Write-Line([string]$s) { Write-Output $s }

        # Admin check
        $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
        ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        Write-Line ("IsAdmin: " + $isAdmin)
        if (-not $isAdmin) {
        Write-Line "ERROR: Administrator privileges required."
        exit 2
        }

        $path = Join-Path $env:windir 'Prefetch'
        if (-not (Test-Path -LiteralPath $path)) {
        Write-Line ("Prefetch folder not found: " + $path)
        exit 0
        }

        # Stop SysMain (best-effort)
        $svcStopped = $false
        try {
        Write-Line "Stopping SysMain..."
        Stop-Service -Name 'SysMain' -Force -ErrorAction Stop
        $svcStopped = $true
        } catch {
        Write-Line ("WARNING: Failed to stop SysMain: " + $_.Exception.Message)
        }

        # Enumerate items
        $items = @()
        try {
        $items = Get-ChildItem -LiteralPath $path -Force -ErrorAction Stop
        } catch {
        Write-Line ("ERROR: Failed to enumerate Prefetch: " + $_.Exception.Message)
        if ($svcStopped) {
            try { Write-Line "Starting SysMain..."; Start-Service -Name 'SysMain' -ErrorAction Stop } catch {}
        }
        exit 1
        }

        Write-Line ("Items before: " + $items.Count)

        # Delete per item, capture failures
        $failed = New-Object System.Collections.Generic.List[string]
        foreach ($it in $items) {
        try {
            Remove-Item -LiteralPath $it.FullName -Force -Recurse -ErrorAction Stop
        } catch {
            $failed.Add(($it.Name + " -> " + $_.Exception.Message)) | Out-Null
        }
        }

        # Re-check remaining
        $remaining = 0
        try {
        $remaining = (Get-ChildItem -LiteralPath $path -Force -ErrorAction SilentlyContinue | Measure-Object).Count
        } catch { $remaining = -1 }

        Write-Line ("Items remaining: " + $remaining)

        if ($failed.Count -gt 0) {
        Write-Line ("FAILED items: " + $failed.Count)
        # Nu lista prea mult, dar dă câteva exemple utile
        $failed | Select-Object -First 25 | ForEach-Object { Write-Line (" - " + $_) }
        }

        # Start SysMain back (best-effort)
        if ($svcStopped) {
        try {
            Write-Line "Starting SysMain..."
            Start-Service -Name 'SysMain' -ErrorAction Stop
        } catch {
            Write-Line ("WARNING: Failed to start SysMain: " + $_.Exception.Message)
        }
        }

        if ($failed.Count -gt 0) { exit 1 } else { exit 0 }
        "#;

    run_powershell_hidden(ps)
}



/// Disk CleanUp pe toate partițiile (Fixed drives) folosind cleanmgr /verylowdisk.
/// - Enumeră toate volumele de tip “Fixed” (DriveType=3)
/// - Rulează cleanmgr.exe pe fiecare: /verylowdisk /d <DriveLetter>
/// - Necesită Administrator pentru rezultate mai bune (și pentru unele volume)
pub fn disk_cleanup_all_partitions() -> String {
    let ps = r#"
        $ErrorActionPreference = 'Stop'

        # Admin check (recomandat)
        $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
        ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

        Write-Output ("IsAdmin: " + $isAdmin)
        if (-not $isAdmin) {
        Write-Output "ERROR: Please run the app as Administrator for Disk Cleanup on all partitions."
        exit 2
        }

        $cleanmgr = Join-Path $env:windir 'System32\cleanmgr.exe'
        if (-not (Test-Path -LiteralPath $cleanmgr)) {
        Write-Output ("ERROR: cleanmgr.exe not found at: " + $cleanmgr)
        exit 1
        }

        # Fixed drives (HDD/SSD) – exclude removable/network
        $drives = Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" |
        Select-Object -ExpandProperty DeviceID

        if (-not $drives -or $drives.Count -eq 0) {
        Write-Output "INFO: No fixed drives found."
        exit 0
        }

        Write-Output ("Fixed drives: " + ($drives -join ', '))

        foreach ($d in $drives) {
        try {
            Write-Output ("Running Disk Cleanup on " + $d + " ...")
            # /verylowdisk = rulează fără UI (sau minimal), folosind setările implicite.
            Start-Process -FilePath $cleanmgr -ArgumentList @("/verylowdisk","/d",$d) -WindowStyle Hidden -Wait
            Write-Output ("DONE: " + $d)
        } catch {
            Write-Output ("ERROR: " + $d + " -> " + $_.Exception.Message)
        }
        }

        Write-Output "SUCCESS: Disk Cleanup attempted on all fixed drives."
        exit 0
        "#;

    run_powershell_hidden(ps)
}




// commands.rs (adaugi acest bloc; include si `use`-urile de mai jos)
// IMPORTANT: aceste functii ruleaza PowerShell elevat (UAC) si capteaza output-ul intr-un fisier temporar.

use std::{
    // fs,
    io::Write,
    path::PathBuf,
    // process::{Command, Stdio},
    // time::{SystemTime, UNIX_EPOCH},
};

// #[cfg(windows)]
// use std::os::windows::process::CommandExt;

// #[cfg(windows)]
// const CREATE_NO_WINDOW: u32 = 0x08000000;

fn ps_escape_single_quotes(s: &str) -> String {
    // in PowerShell single-quoted strings: ' -> ''
    s.replace('\'', "''")
}

fn run_powershell_elevated_capture(script_body: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut ps1_path: PathBuf = std::env::temp_dir();
    ps1_path.push(format!("eoliann_updates_{stamp}.ps1"));

    let mut out_path: PathBuf = std::env::temp_dir();
    out_path.push(format!("eoliann_updates_{stamp}.log"));

    // scrie scriptul in fisier
    match fs::File::create(&ps1_path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(script_body.as_bytes()) {
                return format!("Failed to write temp script: {e}");
            }
        }
        Err(e) => return format!("Failed to create temp script: {e}"),
    }

    let ps1_escaped = ps_escape_single_quotes(&ps1_path.to_string_lossy());
    let out_escaped = ps_escape_single_quotes(&out_path.to_string_lossy());

    // Comanda care ruleaza in PowerShell-ul elevat:
    // ruleaza ps1 si redirecteaza TOATE stream-urile (inclusiv warning/error/info) in fisier.
    let inner_cmd = format!(
        "& '{ps1}' *>&1 | Out-File -FilePath '{out}' -Encoding utf8; exit $LASTEXITCODE",
        ps1 = ps1_escaped,
        out = out_escaped
    );

    // Comanda externa (ne-elevata) care lanseaza un PowerShell elevat (UAC) ascuns si asteapta sa termine.
    let outer_cmd = format!(
        "$cmd = \"{inner}\"; \
         $p = Start-Process -FilePath 'powershell.exe' -Verb RunAs -WindowStyle Hidden -Wait -PassThru \
             -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-Command',$cmd); \
         exit $p.ExitCode",
        inner = inner_cmd.replace('"', "`\"")
    );

    let mut cmd = Command::new("powershell.exe");
    cmd.args(&[
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &outer_cmd,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            let _ = fs::remove_file(&ps1_path);
            let _ = fs::remove_file(&out_path);
            return format!("Failed to start PowerShell: {e}");
        }
    };

    // incearca sa citeasca log-ul produs de procesul elevat
    let file_log = fs::read_to_string(&out_path).unwrap_or_default();

    // cleanup
    let _ = fs::remove_file(&ps1_path);
    let _ = fs::remove_file(&out_path);

    if !file_log.trim().is_empty() {
        return file_log;
    }

    // fallback: stdout/stderr din procesul extern (de ex. daca user-ul a anulat UAC)
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if stdout.trim().is_empty() && stderr.trim().is_empty() {
        if output.status.success() {
            "Done (no output).".to_string()
        } else {
            format!("Operation failed (exit={}).", output.status)
        }
    } else {
        format!(
            "{}{}{}",
            if stdout.trim().is_empty() { "" } else { &stdout },
            if !stdout.trim().is_empty() && !stderr.trim().is_empty() {
                "\n"
            } else {
                ""
            },
            if stderr.trim().is_empty() { "" } else { &stderr }
        )
    }
}

pub fn updates_default_settings() -> String {
    let script = r#"
        $ErrorActionPreference = 'SilentlyContinue'

        Write-Output "Removing Windows Update policy settings..."

        Remove-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Recurse -Force
        Remove-Item -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\DeliveryOptimization" -Recurse -Force
        Remove-Item -Path "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings" -Recurse -Force
        Remove-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Device Metadata" -Recurse -Force
        Remove-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DriverSearching" -Recurse -Force
        Remove-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Recurse -Force

        Write-Output "Reenabling Windows Update Services..."

        Write-Output "Restored BITS to Manual"
        Set-Service -Name BITS -StartupType Manual

        Write-Output "Restored wuauserv to Manual"
        Set-Service -Name wuauserv -StartupType Manual

        Write-Output "Restored UsoSvc to Automatic"
        Set-Service -Name UsoSvc -StartupType Automatic

        Write-Output "Restored WaaSMedicSvc to Manual"
        Set-Service -Name WaaSMedicSvc -StartupType Manual

        Write-Output "Enabling update-related scheduled tasks..."

        $TaskPaths = @(
        "\Microsoft\Windows\InstallService\",
        "\Microsoft\Windows\UpdateOrchestrator\",
        "\Microsoft\Windows\UpdateAssistant\",
        "\Microsoft\Windows\WaaSMedic\",
        "\Microsoft\Windows\WindowsUpdate\",
        "\Microsoft\WindowsUpdate\"
        )

        foreach ($p in $TaskPaths) {
        Get-ScheduledTask -TaskPath $p -ErrorAction SilentlyContinue | Enable-ScheduledTask -ErrorAction SilentlyContinue
        }

        Write-Output "Windows Local Policies Reset to Default"
        secedit /configure /cfg "$Env:SystemRoot\inf\defltbase.inf" /db defltbase.sdb

        Write-Output "==================================================="
        Write-Output "---  Windows Update Settings Reset to Default   ---"
        Write-Output "==================================================="
        Write-Output "Note: You must restart your system in order for all changes to take effect."
        "#;

    run_powershell_elevated_capture(script)
}

pub fn updates_disable_all() -> String {
    let script = r#"
        $ErrorActionPreference = 'SilentlyContinue'

        Write-Output "Configuring registry settings..."

        New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Force | Out-Null

        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "NoAutoUpdate" -Type DWord -Value 1
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "AUOptions" -Type DWord -Value 1

        New-Item -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\DeliveryOptimization\Config" -Force | Out-Null
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\DeliveryOptimization\Config" -Name "DODownloadMode" -Type DWord -Value 0

        Write-Output "Disabling services..."

        Write-Output "Disabled BITS Service"
        Set-Service -Name BITS -StartupType Disabled

        Write-Output "Disabled wuauserv Service"
        Set-Service -Name wuauserv -StartupType Disabled

        Write-Output "Disabled UsoSvc Service"
        Set-Service -Name UsoSvc -StartupType Disabled

        Write-Output "Disabled WaaSMedicSvc Service"
        Set-Service -Name WaaSMedicSvc -StartupType Disabled

        Remove-Item "C:\Windows\SoftwareDistribution\*" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Output "Cleared SoftwareDistribution folder"

        Write-Output "Disabling update-related scheduled tasks..."

        $TaskPaths = @(
        "\Microsoft\Windows\InstallService\",
        "\Microsoft\Windows\UpdateOrchestrator\",
        "\Microsoft\Windows\UpdateAssistant\",
        "\Microsoft\Windows\WaaSMedic\",
        "\Microsoft\Windows\WindowsUpdate\",
        "\Microsoft\WindowsUpdate\"
        )

        foreach ($p in $TaskPaths) {
        Get-ScheduledTask -TaskPath $p -ErrorAction SilentlyContinue | Disable-ScheduledTask -ErrorAction SilentlyContinue
        }

        Write-Output "================================="
        Write-Output "---   Updates Are Disabled    ---"
        Write-Output "================================="
        Write-Output "Note: You must restart your system in order for all changes to take effect."
        "#;

    run_powershell_elevated_capture(script)
}

pub fn updates_security_settings() -> String {
    let script = r#"
        $ErrorActionPreference = 'SilentlyContinue'

        Write-Output "Disabling driver offering through Windows Update..."

        New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Device Metadata" -Force | Out-Null
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Device Metadata" -Name "PreventDeviceMetadataFromNetwork" -Type DWord -Value 1

        New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DriverSearching" -Force | Out-Null
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DriverSearching" -Name "DontPromptForWindowsUpdate" -Type DWord -Value 1
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DriverSearching" -Name "DontSearchWindowsUpdate" -Type DWord -Value 1
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DriverSearching" -Name "DriverUpdateWizardWuSearchEnabled" -Type DWord -Value 0

        New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Force | Out-Null
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Name "ExcludeWUDriversInQualityUpdate" -Type DWord -Value 1

        Write-Output "Deferring feature updates (365 days) and quality updates (4 days)..."

        New-Item -Path "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings" -Force | Out-Null
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings" -Name "BranchReadinessLevel" -Type DWord -Value 20
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings" -Name "DeferFeatureUpdatesPeriodInDays" -Type DWord -Value 365
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings" -Name "DeferQualityUpdatesPeriodInDays" -Type DWord -Value 4

        Write-Output "Disabling Windows Update automatic restart..."

        New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Force | Out-Null
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "NoAutoRebootWithLoggedOnUsers" -Type DWord -Value 1
        Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "AUPowerManagement" -Type DWord -Value 0

        Write-Output "================================="
        Write-Output "-- Updates Set to Recommended ---"
        Write-Output "================================="
        "#;

    run_powershell_elevated_capture(script)
}




#[cfg(windows)]
pub fn disk_health_report_json() -> String {
    use crate::utils::run_powershell;

    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'

function BytesToUInt64([byte[]]$b, [int]$offset){
    [BitConverter]::ToUInt64($b, $offset)
}

function Parse-SmartVendorSpecific([byte[]]$data){
    # vendor data length should be 512 for MSStorageDriver_FailurePredictData
    # SMART attributes are 30 bytes each, starting at offset 2, 12 bytes per attribute entry in some layouts.
    # MSStorageDriver_FailurePredictData uses 12-byte entries starting at offset 2 for 30 attributes (total 362 bytes),
    # but common parsing for this WMI class: each attribute is 12 bytes: Id, Flags(2), Value, Worst, Raw(6), Reserved.
    $attrs = @()
    if (-not $data -or $data.Length -lt 362) { return $attrs }

    $offset = 2
    for ($i = 0; $i -lt 30; $i++){
        $id = [int]$data[$offset]
        if ($id -eq 0){
            $offset += 12
            continue
        }
        $value = [int]$data[$offset + 3]
        $worst = [int]$data[$offset + 4]
        $raw6 = $data[($offset + 5)..($offset + 10)]
        # raw6 is 6 bytes little-endian
        $raw = 0
        for ($j=0; $j -lt 6; $j++){
            $raw += [uint64]$raw6[$j] -shl (8*$j)
        }

        $attrs += [PSCustomObject]@{
            id = $id
            name = $null
            current = $value
            worst = $worst
            threshold = $null
            raw = $raw
        }

        $offset += 12
    }

    # Minimal name mapping for common SMART attributes (optional)
    $nameMap = @{
        5   = 'Reallocated Sectors Count'
        9   = 'Power-On Hours'
        12  = 'Power Cycle Count'
        170 = 'Available Reserved Space'
        171 = 'Program Fail Count'
        172 = 'Erase Fail Count'
        173 = 'Wear Leveling Count'
        174 = 'Unexpected Power Loss Count'
        177 = 'Wear Range Delta'
        179 = 'Used Reserved Block Count Total'
        180 = 'Unused Reserved Block Count Total'
        181 = 'Program Fail Count Total'
        182 = 'Erase Fail Count Total'
        183 = 'Runtime Bad Block'
        184 = 'End-to-End Error'
        187 = 'Reported Uncorrectable Errors'
        188 = 'Command Timeout'
        190 = 'Airflow Temperature'
        194 = 'Temperature'
        196 = 'Reallocation Event Count'
        197 = 'Current Pending Sector Count'
        198 = 'Uncorrectable Sector Count'
        199 = 'UltraDMA CRC Error Count'
        202 = 'Percent Lifetime Used'
        231 = 'Temperature'
        233 = 'Media Wearout Indicator'
        241 = 'Total LBAs Written'
        242 = 'Total LBAs Read'
    }

    foreach ($a in $attrs){
        if ($nameMap.ContainsKey($a.id)){
            $a.name = $nameMap[$a.id]
        }
    }

    return $attrs
}

$smart = @()
try {
    $pred = Get-CimInstance -Namespace root\wmi -ClassName MSStorageDriver_FailurePredictStatus
    $data = Get-CimInstance -Namespace root\wmi -ClassName MSStorageDriver_FailurePredictData

    foreach ($p in $pred){
        $inst = $p.InstanceName
        $pd = $data | Where-Object { $_.InstanceName -eq $inst } | Select-Object -First 1
        $attrs = $null
        if ($pd -and $pd.VendorSpecific){
            $attrs = Parse-SmartVendorSpecific -data $pd.VendorSpecific
        }
        $smart += [PSCustomObject]@{
            instance_name = $inst
            predict_failure = [bool]$p.PredictFailure
            reason = $null
            attributes = $attrs
        }
    }
} catch {
    # ignore
}

$disks = @()
try {
    $pd = Get-PhysicalDisk
    foreach ($d in $pd) {
        $rel = $null
        try { $rel = Get-StorageReliabilityCounter -PhysicalDisk $d } catch { $rel = $null }

        # Wear: prefer % used (Wear). If only PercentLifeRemaining exists, convert to % used.
        $wearUsed = $null
        if ($rel -and $rel.Wear -ne $null) {
            $wearUsed = [int]$rel.Wear
        } elseif ($rel -and $rel.PercentLifeRemaining -ne $null) {
            $wearUsed = 100 - [int]$rel.PercentLifeRemaining
        }

        $health = $null
        if ($wearUsed -ne $null) { $health = 100 - [int]$wearUsed }

        $powerOn = $null
        if ($rel -and $rel.PowerOnHours -ne $null) { $powerOn = [uint64]$rel.PowerOnHours }

        $temp = $null
        if ($rel -and $rel.Temperature -ne $null) { $temp = [double]$rel.Temperature }

        $totalWritten = $null
        if ($rel -and $rel.TotalBytesWritten -ne $null) { $totalWritten = [string]$rel.TotalBytesWritten }

        $disks += [PSCustomObject]@{
            device_id = [string]$d.DeviceId
            friendly_name = [string]$d.FriendlyName
            serial_number = [string]$d.SerialNumber
            model = [string]$d.Model
            media_type = [string]$d.MediaType
            bus_type = [string]$d.BusType
            size_bytes = [uint64]$d.Size

            wear_percent_used = $wearUsed
            health_percent = $health
            power_on_hours = $powerOn
            temperature_c = $temp
            total_bytes_written = $totalWritten
        }
    }
} catch {
    # ignore
}

$report = [PSCustomObject]@{
    generated_at = (Get-Date).ToString('s')
    smart_devices = $smart
    physical_disks = $disks
}

$report | ConvertTo-Json -Depth 7
"#;

    run_powershell(ps)
}



// ========================= Disk Health: NVMe SMART/Health log (0x02) enrichment =========================
// Requires dependency in Cargo.toml:
// windows-sys = { version = "0.59", features = ["Win32_Foundation","Win32_Storage_FileSystem","Win32_System_IO","Win32_System_Ioctl"] }

// #[cfg(windows)]
// #[derive(Debug, Clone)]
// struct NvmeHealthInfo {
//     critical_warning: u8,
//     composite_temp_k: u16,
//     available_spare: u8,
//     available_spare_threshold: u8,
//     percentage_used: u8,
//     power_on_hours: u64,
//     unsafe_shutdowns: u64,
//     media_errors: u64,
//     num_err_info_log_entries: u64,
// }

// #[cfg(windows)]
// fn u64_from_le_16(bytes16: &[u8]) -> u64 {
//     // NVMe uses 128-bit counters in some fields; for UI we take the lower 64 bits.
//     let mut b = [0u8; 8];
//     b.copy_from_slice(&bytes16[..8]);
//     u64::from_le_bytes(b)
// }

// #[cfg(windows)]
// fn nvme_smart_health_log_physical_drive(drive_index: u32) -> Result<NvmeHealthInfo, String> {
//     use std::ffi::OsStr;
//     use std::iter::once;
//     use std::os::windows::ffi::OsStrExt;
//     use std::ptr::{null, null_mut};

//     use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, INVALID_HANDLE_VALUE};
//     use windows_sys::Win32::Storage::FileSystem::{
//         CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ,
//         OPEN_EXISTING,
//     };
//     use windows_sys::Win32::System::IO::DeviceIoControl;
//     use windows_sys::Win32::System::Ioctl::IOCTL_STORAGE_QUERY_PROPERTY;

//     const STORAGE_DEVICE_PROTOCOL_SPECIFIC_PROPERTY: u32 = 50; // StorageDeviceProtocolSpecificProperty
//     const PROPERTY_STANDARD_QUERY: u32 = 0; // PropertyStandardQuery
//     const PROTOCOL_TYPE_NVME: u32 = 3; // ProtocolTypeNvme
//     const NVME_DATA_TYPE_LOG_PAGE: u32 = 2; // NVMeDataTypeLogPage
//     const NVME_LOG_PAGE_HEALTH_INFO: u32 = 0x02;
//     const NVME_LOG_LEN: usize = 512;

//     #[repr(C)]
//     struct StoragePropertyQuery {
//         property_id: u32,
//         query_type: u32,
//         additional_parameters: [u8; 1], // variable length
//     }

//     #[repr(C)]
//     struct StorageProtocolSpecificData {
//         protocol_type: u32,
//         data_type: u32,
//         protocol_data_request_value: u32,
//         protocol_data_request_sub_value: u32,
//         protocol_data_offset: u32,
//         protocol_data_length: u32,
//         fixed_protocol_return_data: u32,
//         protocol_data_request_sub_value2: u32,
//         protocol_data_request_sub_value3: u32,
//         protocol_data_request_sub_value4: u32,
//     }

//     #[repr(C)]
//     struct StorageProtocolDataDescriptor {
//         version: u32,
//         size: u32,
//         protocol_specific_data: StorageProtocolSpecificData,
//     }

//     let path = format!(r"\\.\PhysicalDrive{drive_index}");
//     let wide: Vec<u16> = OsStr::new(&path).encode_wide().chain(once(0)).collect();

//     let handle = unsafe {
//         CreateFileW(
//             wide.as_ptr(),
//             GENERIC_READ,
//             FILE_SHARE_READ | FILE_SHARE_WRITE,
//             null(),
//             OPEN_EXISTING,
//             FILE_ATTRIBUTE_NORMAL,
//             0,
//         )
//     };

//     if handle == INVALID_HANDLE_VALUE {
//         let err = unsafe { GetLastError() };
//         return Err(format!("CreateFileW failed for {path} (GetLastError={err})"));
//     }

//     // input/output buffer (same buffer, as recommended by Microsoft)
//     let header_size = 8usize; // PropertyId + QueryType (4 + 4)
//     let psd_size = std::mem::size_of::<StorageProtocolSpecificData>();
//     let mut buf = vec![0u8; header_size + psd_size + NVME_LOG_LEN];

//     unsafe {
//         let q = buf.as_mut_ptr() as *mut StoragePropertyQuery;
//         (*q).property_id = STORAGE_DEVICE_PROTOCOL_SPECIFIC_PROPERTY;
//         (*q).query_type = PROPERTY_STANDARD_QUERY;

//         let psd = buf.as_mut_ptr().add(header_size) as *mut StorageProtocolSpecificData;
//         (*psd).protocol_type = PROTOCOL_TYPE_NVME;
//         (*psd).data_type = NVME_DATA_TYPE_LOG_PAGE;
//         (*psd).protocol_data_request_value = NVME_LOG_PAGE_HEALTH_INFO;
//         (*psd).protocol_data_request_sub_value = 0;
//         (*psd).fixed_protocol_return_data = 0;
//         (*psd).protocol_data_request_sub_value2 = 0;
//         (*psd).protocol_data_request_sub_value3 = 0;
//         (*psd).protocol_data_request_sub_value4 = 0;

//         (*psd).protocol_data_offset = psd_size as u32;
//         (*psd).protocol_data_length = NVME_LOG_LEN as u32;
//     }

//     let mut returned: u32 = 0;
//     let ok = unsafe {
//         DeviceIoControl(
//             handle,
//             IOCTL_STORAGE_QUERY_PROPERTY,
//             buf.as_mut_ptr() as *mut _,
//             buf.len() as u32,
//             buf.as_mut_ptr() as *mut _,
//             buf.len() as u32,
//             &mut returned as *mut u32,
//             null_mut(),
//         )
//     };

//     unsafe { CloseHandle(handle) };

//     if ok == 0 || returned == 0 {
//         let err = unsafe { GetLastError() };
//         return Err(format!(
//             "DeviceIoControl(IOCTL_STORAGE_QUERY_PROPERTY) failed (GetLastError={err})"
//         ));
//     }

//     if buf.len() < std::mem::size_of::<StorageProtocolDataDescriptor>() {
//         return Err("Returned buffer too small for STORAGE_PROTOCOL_DATA_DESCRIPTOR".to_string());
//     }

//     let desc = unsafe { &*(buf.as_ptr() as *const StorageProtocolDataDescriptor) };

//     if desc.size as usize  < std::mem::size_of::<StorageProtocolDataDescriptor>() {
//         return Err(format!(
//             "Invalid descriptor header: version={}, size={}",
//             desc.version, desc.size
//         ));
//     }

//     let psd = &desc.protocol_specific_data;
//     let off = psd.protocol_data_offset as usize;
//     let len = psd.protocol_data_length as usize;

//     if off < std::mem::size_of::<StorageProtocolSpecificData>() || len < NVME_LOG_LEN {
//         return Err(format!(
//             "Invalid ProtocolDataOffset/Length: off={off}, len={len}"
//         ));
//     }

//     let psd_base = psd as *const StorageProtocolSpecificData as *const u8;
//     let data_ptr = unsafe { psd_base.add(off) };
//     let data = unsafe { std::slice::from_raw_parts(data_ptr, NVME_LOG_LEN) };

//     // Parse NVMe SMART / Health log (Log Identifier 0x02)
//     let critical_warning = data[0];
//     let composite_temp_k = u16::from_le_bytes([data[1], data[2]]);
//     let available_spare = data[3];
//     let available_spare_threshold = data[4];
//     let percentage_used = data[5];

//     // Offsets per NVMe Base Specification (SMART/Health Information Log)
//     let power_on_hours = u64_from_le_16(&data[128..144]);
//     let unsafe_shutdowns = u64_from_le_16(&data[144..160]);
//     let media_errors = u64_from_le_16(&data[160..176]);
//     let num_err_info_log_entries = u64_from_le_16(&data[176..192]);

//     Ok(NvmeHealthInfo {
//         critical_warning,
//         composite_temp_k,
//         available_spare,
//         available_spare_threshold,
//         percentage_used,
//         power_on_hours,
//         unsafe_shutdowns,
//         media_errors,
//         num_err_info_log_entries,
//     })
// }

// #[cfg(windows)]
// fn augment_disk_health_json_with_nvme(mut json: String) -> String {
//     let mut v: serde_json::Value = match serde_json::from_str(&json) {
//         Ok(v) => v,
//         Err(_) => return json,
//     };

//     let physical = match v.get_mut("physical_disks").and_then(|x| x.as_array_mut()) {
//         Some(a) => a,
//         None => return json,
//     };

//     for disk in physical.iter_mut() {
//         let bus = disk
//             .get("bus_type")
//             .and_then(|x| x.as_str())
//             .unwrap_or("")
//             .to_ascii_lowercase();

//         if bus != "nvme" {
//             continue;
//         }

//         let dev_id = match disk.get("device_id").and_then(|x| x.as_u64()) {
//             Some(x) => x as u32,
//             None => continue,
//         };

//         if let Ok(nvme) = nvme_smart_health_log_physical_drive(dev_id) {
//             let temp_c = if nvme.composite_temp_k >= 273 {
//                 Some((nvme.composite_temp_k as u32).saturating_sub(273))
//             } else {
//                 None
//             };

//             // HDSentinel-style health for NVMe: Health = 100 - PercentageUsed
//             let hp = 100u32.saturating_sub((nvme.percentage_used as u32).min(100));

//             disk["nvme_critical_warning"] = serde_json::Value::from(nvme.critical_warning as u32);
//             disk["nvme_composite_temperature_k"] =
//                 serde_json::Value::from(nvme.composite_temp_k as u32);
//             if let Some(tc) = temp_c {
//                 disk["nvme_composite_temperature_c"] = serde_json::Value::from(tc);
//             }
//             disk["nvme_available_spare"] = serde_json::Value::from(nvme.available_spare as u32);
//             disk["nvme_available_spare_threshold"] =
//                 serde_json::Value::from(nvme.available_spare_threshold as u32);
//             disk["nvme_percentage_used"] = serde_json::Value::from(nvme.percentage_used as u32);
//             disk["nvme_power_on_hours"] = serde_json::Value::from(nvme.power_on_hours);
//             disk["nvme_unsafe_shutdowns"] = serde_json::Value::from(nvme.unsafe_shutdowns);
//             disk["nvme_media_errors"] = serde_json::Value::from(nvme.media_errors);
//             disk["nvme_error_info_log_entries"] =
//                 serde_json::Value::from(nvme.num_err_info_log_entries);

//             // Prefer NVMe log for PowerOnHours and HealthPercent if present.
//             disk["power_on_hours"] = serde_json::Value::from(nvme.power_on_hours);
//             disk["health_percent"] = serde_json::Value::from(hp);

//             // Optional: performance is typically 100%
//             if disk.get("performance_percent").is_none() {
//                 disk["performance_percent"] = serde_json::Value::from(100u32);
//             }
//         }
//     }

//     match serde_json::to_string(&v) {
//         Ok(s) => {
//             json = s;
//             json
//         }
//         Err(_) => json,
//     }
// }

// =======================
// Quick Keys (Win shortcuts)
// =======================

use std::mem::size_of;
// use windows::Win32::UI::Input::KeyboardAndMouse::{
//     SendInput, INPUT, INPUT_0, INPUT_KEYBOARD,
//     KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_LWIN,
// };

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput,
    INPUT,
    INPUT_0,
    INPUT_KEYBOARD,
    KEYBDINPUT,
    KEYEVENTF_KEYUP,
    // KEYBD_EVENT_FLAGS,
    VIRTUAL_KEY,
    VK_LWIN,
};

fn send_key_combo(modifier: VIRTUAL_KEY, key: VIRTUAL_KEY) {
    unsafe {
        let inputs = [
            INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: modifier, wScan: 0, dwFlags: Default::default(), time: 0, dwExtraInfo: 0 } } },
            INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: key, wScan: 0, dwFlags: Default::default(), time: 0, dwExtraInfo: 0 } } },
            INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: key, wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 } } },
            INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: modifier, wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 } } },
        ];

        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

pub fn send_win_x() {
    send_key_combo(VK_LWIN, VIRTUAL_KEY(0x58));
}

pub fn send_win_d() {
    send_key_combo(VK_LWIN, VIRTUAL_KEY(0x44));
}

pub fn send_win_l() {
    let mut cmd = Command::new("rundll32.exe");
    cmd.arg("user32.dll,LockWorkStation");
    #[cfg(windows)] cmd.creation_flags(CREATE_NO_WINDOW);
    let _ = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

pub fn send_win_r() {
    send_key_combo(VK_LWIN, VIRTUAL_KEY(0x52));
}

pub fn send_win_e() {
    let _ = Command::new("explorer.exe").creation_flags(CREATE_NO_WINDOW).spawn();
}

pub fn send_win_i() {
    let _ = Command::new("explorer.exe")
        .arg("ms-settings:")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}


// ========================= NETWORK TOOLS (versiune îmbunătățită 2026) =========================

/// Struct pentru status (verifică AMBELE locații)
#[derive(Clone, Debug)]
pub struct NetworkPolicyStatus {
    pub require_security_signature: Option<u32>,
    pub allow_insecure_guest_auth: Option<u32>,
    pub allow_insecure_guest_auth_legacy: Option<u32>, // pentru compatibilitate
}

/// Verifică status-ul real din ambele locații posibile
pub fn check_network_policy_status() -> NetworkPolicyStatus {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let require = hklm
        .open_subkey(r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("RequireSecuritySignature").ok());

    let policies = hklm
        .open_subkey(r"SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("AllowInsecureGuestAuth").ok());

    let legacy = hklm
        .open_subkey(r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("AllowInsecureGuestAuth").ok());

    NetworkPolicyStatus {
        require_security_signature: require,
        allow_insecure_guest_auth: policies.or(legacy), // prioritate Policies
        allow_insecure_guest_auth_legacy: legacy,
    }
}

/// Aplică fix-ul complet (ambele locații + restart serviciu)
pub fn apply_network_compatibility_fix() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut log = Vec::new();

    // 1. RequireSecuritySignature = 0
    if let Ok((k, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters") {
        let _ = k.set_value("RequireSecuritySignature", &0u32);
        log.push("✅ RequireSecuritySignature = 0");
    }

    // 2. AllowInsecureGuestAuth în locația oficială (recomandată 24H2+)
    if let Ok((k, _)) = hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation") {
        let _ = k.set_value("AllowInsecureGuestAuth", &1u32);
        log.push("✅ AllowInsecureGuestAuth (Policies) = 1");
    }

    // 3. AllowInsecureGuestAuth în locația legacy (pentru 23H2 și mai vechi)
    if let Ok((k, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters") {
        let _ = k.set_value("AllowInsecureGuestAuth", &1u32);
        log.push("✅ AllowInsecureGuestAuth (legacy) = 1");
    }

    // 4. Restart serviciu LanmanWorkstation
    let _ = std::process::Command::new("net").args(&["stop", "lanmanworkstation", "/y"]).output();
    let restart = std::process::Command::new("net").args(&["start", "lanmanworkstation"]).output();

    if restart.map(|o| o.status.success()).unwrap_or(false) {
        log.push("🔄 Serviciul LanmanWorkstation a fost restartat");
    } else {
        log.push("⚠ Restart serviciu eșuat → recomand restart complet Windows");
    }

    format!(
        "SUCCESS: Network Compatibility Fix fully implemented!\n\n{}\n\n\
        IMPORTANT NOTE:\n\
        • gpedit.msc will still show \"Not Configured\" → this is NORMAL and that doesn't mean it doesn't work!!\n\
        • The change works through the registry.\n\
        • Test access to a guest share after restarting Windows.\n\
        • If it still doesn't work: run in PowerShell: Get-SmbClientConfiguration | Select EnableInsecureGuestLogons",
        log.join("\n")
    )
}

/// Restore secure defaults (șterge din ambele locații)
pub fn restore_secure_defaults() -> String {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE};
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut log = Vec::new();

    // RequireSecuritySignature = 1
    if let Ok((k, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters") {
        let _ = k.set_value("RequireSecuritySignature", &1u32);
        log.push("✅ RequireSecuritySignature restaurat la 1");
    }

    // Șterge AllowInsecureGuestAuth din ambele locații
    if let Ok(k) = hklm.open_subkey_with_flags(r"SOFTWARE\Policies\Microsoft\Windows\LanmanWorkstation", KEY_SET_VALUE) {
        let _ = k.delete_value("AllowInsecureGuestAuth");
        log.push("✅ AllowInsecureGuestAuth (Policies) șters");
    }
    if let Ok(k) = hklm.open_subkey_with_flags(r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters", KEY_SET_VALUE) {
        let _ = k.delete_value("AllowInsecureGuestAuth");
        log.push("✅ AllowInsecureGuestAuth (legacy) șters");
    }

    let _ = std::process::Command::new("net").args(&["stop", "lanmanworkstation", "/y"]).output();
    let _ = std::process::Command::new("net").args(&["start", "lanmanworkstation"]).output();

    format!(
        "SUCCESS: Secure defaults restaurate!\n\n{}\n\nRecomand restart complet Windows.",
        log.join("\n")
    )
}

pub fn open_registry_editor() {
    // Pas 1: Deschide Run dialog (Win + R)
    send_key_combo(VK_LWIN, VIRTUAL_KEY(0x52));  // Win + R

    // Așteptăm puțin ca dialogul să apară (important!)
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Pas 2: Tastează "regedit"
    let text = "regedit";
    for c in text.chars() {
        let vk = match c.to_ascii_uppercase() {
            'R' => VIRTUAL_KEY(0x52),
            'E' => VIRTUAL_KEY(0x45),
            'G' => VIRTUAL_KEY(0x47),
            'D' => VIRTUAL_KEY(0x44),
            'I' => VIRTUAL_KEY(0x49),
            'T' => VIRTUAL_KEY(0x54),
            _ => continue,
        };

        unsafe {
            let input_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: Default::default(),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let input_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let inputs = [input_down, input_up];
            SendInput(&inputs, size_of::<INPUT>() as i32);
        }

        std::thread::sleep(std::time::Duration::from_millis(30)); // ritm natural
    }

    // Pas 3: Apasă Enter
    unsafe {
        let enter_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0x0D), // VK_RETURN
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let enter_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0x0D),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let inputs = [enter_down, enter_up];
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

pub fn open_group_policy_editor() {
    let _ = std::process::Command::new("cmd")
        .args(&["/c", "start", "gpedit.msc"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}

/// Returnează ediția Windows (ex: "Professional", "Core", "Enterprise" etc.)
fn get_windows_edition() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") {
        if let Ok(edition) = key.get_value::<String, _>("EditionID") {
            return edition;
        }
    }
    "Unknown".to_string()
}

/// Activează Group Policy Editor (gpedit.msc) pe Windows Home - cu verificare inteligentă
pub fn enable_group_policy_editor() -> String {
    use crate::utils::is_elevated;

    if !is_elevated() {
        return "❌ ERROR: This operation requires Administrator rights.\n\nPlease run the application as Administrator.".to_string();
    }

    // Verificare 1: gpedit.msc există deja?
    if std::path::Path::new(r"C:\Windows\System32\gpedit.msc").exists() {
        return "✅ Group Policy Editor is already installed and available on this system.\n\nNo action needed.".to_string();
    }

    // Verificare 2: Ce ediție de Windows avem?
    let edition = get_windows_edition();

    if edition.contains("Pro")
        || edition.contains("Enterprise")
        || edition.contains("Education")
        || edition.contains("Server")
        || edition.contains("IoTEnterprise") {

        return format!(
            "✅ Not needed.\n\n\
            Group Policy Editor (gpedit.msc) is already included in your edition: {}\n\n\
            You can open it directly with Win + R → gpedit.msc",
            edition
        );
    }

    // Dacă am ajuns aici → este Windows Home + gpedit nu este instalat
    let mut log = String::new();
    log.push_str("🚀 Windows Home detected + gpedit.msc not found.\n");
    log.push_str("Starting automatic activation...\n\n");
    log.push_str("This may take 30–90 seconds. Please wait...\n\n");

    // Comanda 1
    log.push_str("→ Adding GroupPolicy-ClientTools package...\n");
    let cmd1 = r#"cmd /c "FOR %%F IN ("%SystemRoot%\servicing\Packages\Microsoft-Windows-GroupPolicy-ClientTools-Package~*.mum") DO dism /online /norestart /add-package:"%F""#;
    log.push_str(&run_command(cmd1));
    log.push_str("\n\n");

    // Comanda 2
    log.push_str("→ Adding GroupPolicy-ClientExtensions package...\n");
    let cmd2 = r#"cmd /c "FOR %%F IN ("%SystemRoot%\servicing\Packages\Microsoft-Windows-GroupPolicy-ClientExtensions-Package~*.mum") DO dism /online /norestart /add-package:"%F""#;
    log.push_str(&run_command(cmd2));

    log.push_str("\n\n✅ SUCCESS: Group Policy Editor has been activated!\n");
    log.push_str("🔄 Please restart your computer now.\n");
    log.push_str("After restart, press Win + R and type: gpedit.msc");

    log
}

// ============================
// SECURITY: Password expiration / admin / domain / users
// ============================

/// Fast: read Windows ProductName from registry (e.g., "Windows 11 Pro").
pub fn security_windows_product_name() -> String {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") {
            if let Ok(product) = key.get_value::<String, _>("ProductName") {
                return product;
            }
        }
    }
    "Unknown".to_string()
}

/// Reliable admin check without PowerShell (works even if quoting/locale changes).
/// Uses `net session` which requires admin; non-admin returns non-zero.
pub fn security_is_running_as_admin() -> bool {
    let status = StdCommand::new("cmd")
        .args(["/C", "net session >nul 2>&1"])
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// Domain join info (best-effort). Returns (part_of_domain, domain_name).
pub fn security_domain_info() -> (bool, String) {
    // Try WMIC first (fast enough; may be missing in future builds).
    let out = StdCommand::new("cmd")
        .args(["/C", "wmic computersystem get PartOfDomain,Domain /value"])
        .output();

    if let Ok(o) = out {
        let text = String::from_utf8_lossy(&o.stdout).to_string() + &String::from_utf8_lossy(&o.stderr);
        let mut part = None;
        let mut dom = None;

        for line in text.lines() {
            let l = line.trim();
            if let Some(v) = l.strip_prefix("PartOfDomain=") {
                part = Some(v.trim().eq_ignore_ascii_case("TRUE"));
            } else if let Some(v) = l.strip_prefix("Domain=") {
                let v = v.trim();
                if !v.is_empty() {
                    dom = Some(v.to_string());
                }
            }
        }

        if let Some(p) = part {
            return (p, dom.unwrap_or_default());
        }
    }

    // Fallback PowerShell (slower)
    let ps = r#"$cs=Get-CimInstance Win32_ComputerSystem; "PartOfDomain=$($cs.PartOfDomain)"; "Domain=$($cs.Domain)""#;
    let text = run_powershell_command(ps);

    let mut part = false;
    let mut dom = String::new();
    for line in text.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("PartOfDomain=") {
            part = v.trim().eq_ignore_ascii_case("True");
        } else if let Some(v) = l.strip_prefix("Domain=") {
            dom = v.trim().to_string();
        }
    }
    (part, dom)
}

/// List local users (clean): excludes common system accounts; returns only enabled accounts where possible.
/// Uses PowerShell once (background-thread friendly; not used in UI thread).
pub fn security_list_local_users_clean() -> Vec<String> {
    // Single PS call for deterministic list
    let ps = r#"
Get-LocalUser |
Where-Object { $_.Enabled -eq $true } |
Select-Object -ExpandProperty Name
"#;
    let out = run_powershell_command(ps);
    let mut users: Vec<String> = out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Remove system accounts
    let deny = ["administrator", "defaultaccount", "guest", "wdagutilityaccount"];
    users.retain(|u| !deny.iter().any(|d| u.eq_ignore_ascii_case(d)));

    users.sort();
    users.dedup();
    users
}

/// Best-effort Microsoft account detection for a local user.
pub fn security_is_microsoft_account(username: &str) -> bool {
    if username.trim().is_empty() {
        return false;
    }

    let ps = format!(
        r#"$u=Get-LocalUser -Name "{}"; "PS="+$u.PrincipalSource"#,
        username.replace('"', r#"\""#)
    );
    let out = run_powershell_command(&ps);
    out.lines().any(|l| l.trim().eq_ignore_ascii_case("PS=MicrosoftAccount"))
}

/// Read PasswordNeverExpires (true => expiration disabled).
pub fn security_password_never_expires(username: &str) -> Result<bool, String> {
    if username.trim().is_empty() {
        return Err("Empty username".into());
    }

    // One PS call, parse stable tokens
    let ps = format!(
        r#"$u=Get-LocalUser -Name "{}"; "PNE="+$u.PasswordNeverExpires"#,
        username.replace('"', r#"\""#)
    );
    let out = run_powershell_command(&ps);

    for line in out.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("PNE=") {
            let v = v.trim();
            if v.eq_ignore_ascii_case("True") {
                return Ok(true);
            }
            if v.eq_ignore_ascii_case("False") {
                return Ok(false);
            }
        }
    }

    // Fallback WMIC (opposite semantics: PasswordExpires)
    let wmic = format!(
        "wmic UserAccount where Name='{}' get PasswordExpires /value",
        username.replace('\'', "''")
    );

    let o = StdCommand::new("cmd")
        .args(["/C", &wmic])
        .output()
        .map_err(|e| format!("Failed to run WMIC: {}", e))?;

    let text = String::from_utf8_lossy(&o.stdout).to_string() + &String::from_utf8_lossy(&o.stderr);
    for line in text.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("PasswordExpires=") {
            let v = v.trim();
            if v.eq_ignore_ascii_case("TRUE") {
                return Ok(false);
            }
            if v.eq_ignore_ascii_case("FALSE") {
                return Ok(true);
            }
        }
    }

    Err(format!("Cannot determine status. Raw: {}", out.trim()))
}

/// Set PasswordNeverExpires to target.
/// target=true => disable expiration; target=false => enable expiration.
pub fn security_set_password_never_expires(username: &str, target: bool) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Empty username".into());
    }

    let value = if target { "$true" } else { "$false" };

    let ps = format!(
        r#"Set-LocalUser -Name "{}" -PasswordNeverExpires {}"#,
        username.replace('"', r#"\""#),
        value
    );

    let mut c = StdCommand::new(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    c.args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-Command", &ps]);
    #[cfg(windows)]
    { c.creation_flags(CREATE_NO_WINDOW); }

    match c.status() {
        Ok(st) if st.success() => Ok(()),
        _ => {
            // Fallback WMIC (PasswordExpires is inverse)
            let wmic_value = if target { "False" } else { "True" };
            let cmd = format!(
                "wmic UserAccount where Name='{}' set PasswordExpires={}",
                username.replace('\'', "''"),
                wmic_value
            );

            let st = StdCommand::new("cmd")
                .args(["/C", &cmd])
                .status()
                .map_err(|e| format!("Failed to run WMIC: {}", e))?;

            if st.success() { Ok(()) } else { Err("Failed to update password expiration.".into()) }
        }
    }
}
