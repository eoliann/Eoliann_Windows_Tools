use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Rulează un cmd/sh ascuns și întoarce stdout + stderr concatenate.
pub fn run_command(cmdline: &str) -> String {
    let out = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", cmdline])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    } else {
        Command::new("sh")
            .args(["-c", cmdline])
            .output()
    };

    match out {
        Ok(o) => {
            let mut s = String::new();
            s.push_str(&String::from_utf8_lossy(&o.stdout));

            let err = String::from_utf8_lossy(&o.stderr);
            if !err.trim().is_empty() {
                if !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str(&err);
            }

            s
        }
        Err(e) => format!("Exec error: {e}"),
    }
}

/// Rulează un script PowerShell ascuns și întoarce stdout + stderr.
pub fn run_powershell(script: &str) -> String {
    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW) // ascunde fereastra CMD
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let mut result = String::new();

            if !o.stdout.is_empty() {
                result.push_str(&String::from_utf8_lossy(&o.stdout));
            }
            if !o.stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str("⚠ ERROR: ");
                result.push_str(&String::from_utf8_lossy(&o.stderr));
            }

            if result.trim().is_empty() {
                "⚠ No output returned.".to_string()
            } else {
                result
            }
        }
        Err(e) => format!("❌ Execution error: {e}"),
    }
}

use reqwest::blocking::Client;
use serde::Deserialize;
#[allow(unused_imports)]
use semver::Version;

#[derive(Deserialize, Debug)]
pub struct GithubRelease {
    pub tag_name: String,
    #[allow(dead_code)]
    pub html_url: String,
}

/// Verifică ultima versiune disponibilă pe GitHub Releases.
pub fn check_latest_version() -> Option<GithubRelease> {
    let url = "https://api.github.com/repos/eoliann/Eoliann_Windows_Tools/releases/latest";
    let client = Client::new();

    let resp = client
        .get(url)
        .header("User-Agent", "EoliannWindowsTools")
        .send()
        .ok()?;

    let release: GithubRelease = resp.json().ok()?;
    Some(release)
}

/// Compară versiunea curentă cu ultima versiune.
/// Returnează `true` dacă există o versiune mai nouă pe GitHub.
pub fn is_update_available(current: &str, latest: &str) -> bool {
    let cur = Version::parse(current).unwrap_or_else(|_| Version::new(0, 0, 0));
    let lat = Version::parse(latest.trim_start_matches('v'))
        .unwrap_or_else(|_| Version::new(0, 0, 0));
    cur < lat
}


// src/utils.rs (sau adaugă în utils existent)
#[cfg(target_os = "windows")]
pub fn is_elevated() -> bool {
    use std::mem::size_of;
    use std::ptr::null_mut;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::processthreadsapi::OpenProcessToken;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use winapi::um::securitybaseapi::GetTokenInformation;

    unsafe {
        let mut token_handle = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_size: DWORD = 0;
        let res = GetTokenInformation(
            token_handle,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_size,
        );
        if res == 0 {
            return false;
        }
        elevation.TokenIsElevated != 0
    }
}

#[cfg(target_os = "windows")]
pub fn relaunch_as_admin() -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOWNORMAL;

    // Path to current executable
    let exe = std::env::current_exe().map_err(|e| format!("current_exe error: {}", e))?;
    let exe_w: Vec<u16> = OsStr::new(&exe)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let verb: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
    let lp_file = exe_w.as_ptr();
    let lp_verb = verb.as_ptr();

    unsafe {
        let res = ShellExecuteW(
            null_mut(),
            lp_verb,
            lp_file,
            null_mut(),
            null_mut(),
            SW_SHOWNORMAL,
        );
        // ShellExecuteW returns a value > 32 on success
        if (res as isize) <= 32 {
            Err(format!("ShellExecuteW failed (code {})", res as isize))
        } else {
            Ok(())
        }
    }
}

// For non-windows builds, provide stubs
#[cfg(not(target_os = "windows"))]
pub fn is_elevated() -> bool { true }

#[cfg(not(target_os = "windows"))]
pub fn relaunch_as_admin() -> Result<(), String> { Ok(()) }
