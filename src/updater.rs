//! Update check and in-app installation from GitHub Releases.
//!
//! The check runs on a background thread (it used to block application start-up). Installing
//! downloads the Inno Setup installer of the latest release, verifies its SHA-256 against the
//! digest GitHub publishes for every release asset, then runs it silently and closes the app.

use semver::Version;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const LATEST_RELEASE_API: &str = "https://api.github.com/repos/eoliann/Eoliann_Windows_Tools/releases/latest";
pub const RELEASES_PAGE: &str = "https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest";
const USER_AGENT: &str = "EoliannWindowsTools-Updater";

#[derive(Debug, Clone)]
pub struct GithubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub installer_url: Option<String>,
    pub installer_sha256: Option<String>,
}

#[derive(Clone, Default)]
pub enum Phase {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(GithubRelease),
    Installing(String),
    Failed(String),
}

#[derive(Default)]
struct State {
    phase: Phase,
    /// A newly found release that the UI has not picked up yet.
    pending: Option<GithubRelease>,
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn state() -> &'static Mutex<State> {
    STATE.get_or_init(|| Mutex::new(State::default()))
}

pub fn phase() -> Phase {
    state().lock().map(|s| s.phase.clone()).unwrap_or_default()
}

fn set_phase(phase: Phase) {
    if let Ok(mut s) = state().lock() {
        s.phase = phase;
    }
}

/// Returns a release found by a check since the last call (used to open the update window).
pub fn take_new_release() -> Option<GithubRelease> {
    state().lock().ok().and_then(|mut s| s.pending.take())
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

fn describe(error: reqwest::Error) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(inner) = source {
        text.push_str(&format!(" ({inner})"));
        source = inner.source();
    }
    text
}

pub fn is_update_available(current: &str, latest: &str) -> bool {
    let cur = Version::parse(current.trim_start_matches('v')).unwrap_or_else(|_| Version::new(0, 0, 0));
    let lat = Version::parse(latest.trim_start_matches('v')).unwrap_or_else(|_| Version::new(0, 0, 0));
    cur < lat
}

fn fetch_latest() -> Result<GithubRelease, String> {
    let response = client()?
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(20))
        .send()
        .map_err(describe)?;
    if !response.status().is_success() {
        return Err(format!("GitHub returned HTTP {}", response.status()));
    }
    let v: Value = response.json().map_err(describe)?;
    let tag_name = v.get("tag_name").and_then(Value::as_str).ok_or("Release has no tag")?.to_string();
    let html_url = v.get("html_url").and_then(Value::as_str).unwrap_or(RELEASES_PAGE).to_string();

    let installer = v.get("assets").and_then(Value::as_array).and_then(|assets| {
        assets.iter().find(|asset| {
            asset
                .get("name")
                .and_then(Value::as_str)
                .map(|name| {
                    let name = name.to_ascii_lowercase();
                    name.ends_with(".exe") && name.contains("installer")
                })
                .unwrap_or(false)
        })
    });
    let installer_url = installer.and_then(|a| a.get("browser_download_url")).and_then(Value::as_str).map(str::to_string);
    let installer_sha256 = installer
        .and_then(|a| a.get("digest"))
        .and_then(Value::as_str)
        .and_then(|d| d.strip_prefix("sha256:"))
        .map(|d| d.to_ascii_lowercase());

    Ok(GithubRelease { tag_name, html_url, installer_url, installer_sha256 })
}

/// Starts a background check. `manual` checks also report "up to date" and errors.
pub fn start_check(manual: bool) {
    {
        let Ok(mut s) = state().lock() else { return };
        if matches!(s.phase, Phase::Checking | Phase::Installing(_)) {
            return;
        }
        s.phase = Phase::Checking;
    }
    std::thread::spawn(move || {
        let current = env!("CARGO_PKG_VERSION");
        let result = fetch_latest();
        let Ok(mut s) = state().lock() else { return };
        s.phase = match result {
            Ok(release) if is_update_available(current, &release.tag_name) => {
                s.pending = Some(release.clone());
                Phase::Available(release)
            }
            Ok(_) => Phase::UpToDate,
            Err(error) if manual => Phase::Failed(format!("Update check failed: {error}")),
            Err(_) => Phase::Idle,
        };
    });
}

fn download_and_verify(release: &GithubRelease) -> Result<std::path::PathBuf, String> {
    let url = release.installer_url.as_deref().ok_or("This release has no installer asset. Download it manually from GitHub.")?;
    let expected = release
        .installer_sha256
        .as_deref()
        .ok_or("GitHub did not publish a checksum for the installer, so it cannot be verified. Download it manually.")?;

    let mut response = client()?.get(url).send().map_err(describe)?;
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }
    let total = response.content_length().unwrap_or(0);
    let mut data = Vec::with_capacity(total as usize);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = response.read(&mut buffer).map_err(|e| format!("Download interrupted: {e}"))?;
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..read]);
        if total > 0 {
            set_phase(Phase::Installing(format!("Downloading update... {}%", data.len() as u64 * 100 / total)));
        }
    }

    let actual: String = Sha256::digest(&data).iter().map(|b| format!("{b:02x}")).collect();
    if actual != expected {
        return Err("The downloaded installer does not match the checksum published on GitHub. Update aborted.".into());
    }

    let folder = std::env::temp_dir().join("EWT-update");
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create {}: {e}", folder.display()))?;
    let path = folder.join(format!("Eoliann_Windows_Tools_Installer_{}.exe", release.tag_name.trim_start_matches('v')));
    std::fs::write(&path, &data).map_err(|e| format!("Could not save the installer: {e}"))?;
    Ok(path)
}

/// Downloads, verifies and runs the installer, then exits so files can be replaced.
pub fn start_install(release: GithubRelease) {
    {
        let Ok(mut s) = state().lock() else { return };
        if matches!(s.phase, Phase::Installing(_)) {
            return;
        }
        s.phase = Phase::Installing("Downloading update...".into());
    }
    std::thread::spawn(move || {
        let installer = match download_and_verify(&release) {
            Ok(path) => path,
            Err(error) => {
                set_phase(Phase::Failed(error));
                return;
            }
        };
        set_phase(Phase::Installing("Checksum verified. Starting installer...".into()));
        let spawned = std::process::Command::new(&installer)
            .args(["/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CLOSEAPPLICATIONS"])
            .spawn();
        match spawned {
            Ok(_) => {
                std::thread::sleep(Duration::from_millis(800));
                std::process::exit(0);
            }
            Err(error) => set_phase(Phase::Failed(format!("Could not start the installer: {error}"))),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::is_update_available;

    #[test]
    fn compares_versions() {
        assert!(is_update_available("1.3.5", "v1.4.0"));
        assert!(is_update_available("1.3.5", "1.3.10"));
        assert!(!is_update_available("1.4.0", "v1.4.0"));
        assert!(!is_update_available("1.4.0", "v1.3.9"));
    }
}
