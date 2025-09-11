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
                if !s.ends_with('\n') { s.push('\n'); }
                s.push_str(&err);
            }
            s
        }
        Err(e) => format!("Exec error: {e}"),
    }
}

pub fn run_powershell(script: &str) -> String {
    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", script])
        .creation_flags(0x08000000) // ascunde CMD
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
                result.push_str("\n⚠ ERROR: ");
                result.push_str(&String::from_utf8_lossy(&o.stderr));
            }
            if result.trim().is_empty() {
                "⚠ No output returned.".to_string()
            } else {
                result
            }
        }
        Err(e) => format!("❌ Execution error: {}", e),
    }
}
