#[cfg(target_os = "windows")]
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(target_os = "windows")]
use winreg::{enums::*, RegKey};

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
};

#[cfg(target_os = "windows")]
const VC_REDIST_URL: &str = "https://aka.ms/vc14/vc_redist.x64.exe";

#[cfg(target_os = "windows")]
pub fn ensure_vc_runtime_x64() {
    if !cfg!(target_arch = "x86_64") {
        return;
    }

    if is_vc_runtime_present() {
        return;
    }

    show_info(
        "Eoliann Windows Tools",
        "Microsoft Visual C++ Runtime is missing.\n\n\
         Eoliann Windows Tools will download and install the official Microsoft Visual C++ Redistributable x64.",
    );

    match download_verify_install_vcredist() {
        Ok(()) => {
            if is_vc_runtime_present() {
                show_info(
                    "Eoliann Windows Tools",
                    "Microsoft Visual C++ Runtime was installed successfully.\n\n\
                     The application will continue to start.",
                );
            } else {
                show_error(
                    "Eoliann Windows Tools",
                    "The Microsoft Visual C++ Redistributable installer finished, \
                     but the runtime could not be confirmed.\n\n\
                     Please restart Windows and try again.",
                );
            }
        }
        Err(err) => {
            show_error(
                "Eoliann Windows Tools",
                &format!(
                    "Microsoft Visual C++ Runtime is missing and could not be installed automatically.\n\n\
                     Error:\n{}\n\n\
                     You can install it manually from Microsoft:\n\
                     https://aka.ms/vc14/vc_redist.x64.exe",
                    err
                ),
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_vc_runtime_x64() {}

#[cfg(target_os = "windows")]
fn is_vc_runtime_present() -> bool {
    is_vc_runtime_installed_registry() || vcruntime_dll_exists()
}

#[cfg(target_os = "windows")]
fn vcruntime_dll_exists() -> bool {
    let system_root = match env::var_os("SystemRoot") {
        Some(value) => PathBuf::from(value),
        None => return false,
    };

    system_root
        .join("System32")
        .join("VCRUNTIME140.dll")
        .exists()
}

#[cfg(target_os = "windows")]
fn is_vc_runtime_installed_registry() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let paths = [
        r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
        r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
    ];

    for path in paths {
        if let Ok(key) = hklm.open_subkey(path) {
            let installed: Result<u32, _> = key.get_value("Installed");
            if installed.unwrap_or(0) == 1 {
                return true;
            }

            let version: Result<String, _> = key.get_value("Version");
            if version.is_ok() {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "windows")]
fn download_verify_install_vcredist() -> Result<(), String> {
    let download_path = env::temp_dir().join("vc_redist.x64.exe");

    if download_path.exists() {
        let _ = fs::remove_file(&download_path);
    }

    download_vcredist(&download_path)?;
    verify_microsoft_signature(&download_path)?;
    install_vcredist(&download_path)?;

    let _ = fs::remove_file(&download_path);

    Ok(())
}

#[cfg(target_os = "windows")]
fn download_vcredist(path: &Path) -> Result<(), String> {
    let out = ps_quote(&path_to_string(path)?);

    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
         Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
        VC_REDIST_URL, out
    );

    let code = run_powershell(&script)?;

    if code == 0 {
        Ok(())
    } else {
        Err(format!("Download failed. PowerShell exit code: {}", code))
    }
}

#[cfg(target_os = "windows")]
fn verify_microsoft_signature(path: &Path) -> Result<(), String> {
    let file = ps_quote(&path_to_string(path)?);

    let script = format!(
        "$sig = Get-AuthenticodeSignature -FilePath '{}'; \
         if ($sig.Status -ne 'Valid') {{ exit 20 }}; \
         if ($null -eq $sig.SignerCertificate) {{ exit 21 }}; \
         if ($sig.SignerCertificate.Subject -notmatch 'Microsoft') {{ exit 22 }}; \
         exit 0",
        file
    );

    let code = run_powershell(&script)?;

    if code == 0 {
        Ok(())
    } else {
        Err(format!(
            "Downloaded installer failed Microsoft Authenticode verification. Exit code: {}",
            code
        ))
    }
}

#[cfg(target_os = "windows")]
fn install_vcredist(path: &Path) -> Result<(), String> {
    let file = ps_quote(&path_to_string(path)?);

    let script = format!(
        "$p = Start-Process -FilePath '{}' \
         -ArgumentList '/install','/passive','/norestart' \
         -Wait -PassThru; \
         exit $p.ExitCode",
        file
    );

    let code = run_powershell(&script)?;

    match code {
        0 | 3010 | 1638 => Ok(()),
        _ => Err(format!(
            "VC++ Redistributable installer failed. Exit code: {}",
            code
        )),
    }
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<i32, String> {
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .status()
        .map_err(|err| format!("Failed to start PowerShell: {}", err))?;

    status
        .code()
        .ok_or_else(|| "PowerShell ended without an exit code.".to_string())
}

#[cfg(target_os = "windows")]
fn path_to_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(|value| value.to_string())
        .ok_or_else(|| "Invalid temporary file path.".to_string())
}

#[cfg(target_os = "windows")]
fn ps_quote(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(target_os = "windows")]
fn show_info(title: &str, message: &str) {
    show_message(title, message, MB_ICONINFORMATION);
}

#[cfg(target_os = "windows")]
fn show_error(title: &str, message: &str) {
    show_message(title, message, MB_ICONERROR);
}

#[cfg(target_os = "windows")]
fn show_message(title: &str, message: &str, icon: u32) {
    let title_w = to_wide(title);
    let message_w = to_wide(message);

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message_w.as_ptr(),
            title_w.as_ptr(),
            MB_OK | icon,
        );
    }
}

#[cfg(target_os = "windows")]
fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}