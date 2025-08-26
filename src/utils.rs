use std::process::{Command, Stdio};
use std::os::windows::process::CommandExt; // necesar pentru creation_flags

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run_command(cmd: &str) -> String {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW) // 🔥 ascunde CMD
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.trim().is_empty() {
                if stdout.trim().is_empty() {
                    "✅ Command executed successfully (no output)".to_string()
                } else {
                    stdout
                }
            } else {
                format!("{}\n{}", stdout, stderr)
            }
        }
        Err(e) => format!("❌ Failed to execute command: {}", e),
    }
}
