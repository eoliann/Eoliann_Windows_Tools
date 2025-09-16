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
