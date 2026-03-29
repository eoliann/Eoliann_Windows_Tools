[![Group](https://img.shields.io/badge/Group-Telegram-blue?style=plastic)](https://t.me/eoliannwindowstool)
![Followers](https://img.shields.io/github/followers/eoliann?style=plastic&color=green)
![Watchers](https://img.shields.io/github/watchers/eoliann/wup-web?style=plastic)
![Stars](https://img.shields.io/github/stars/eoliann/Eoliann_Windows_Tools?style=plastic)
[![Donate](https://img.shields.io/badge/Donate-PayPal-blue?style=plastic)](https://www.paypal.com/donate/?hosted_button_id=PTH2EXUDS423S)
[![Donate](https://img.shields.io/badge/Donate-Revolut-8A2BE2?style=plastic)](http://revolut.me/adriannm9?style=plastic)

![Release Date](https://img.shields.io/github/release-date/eoliann/Eoliann_Windows_Tools?style=plastic)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/Eoliann_Windows_Tools?style=plastic)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=plastic)](LICENSE.md)
![OS](https://img.shields.io/badge/OS-Windows-blue?style=plastic)
![Lang](https://img.shields.io/badge/Lang-Rust-magenta?style=plastic)

![Total Downloads](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/total?style=plastic)
![](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/eoliann_windows_tools_w11.exe?displayAssetName=true&style=plastic&color=green)
![](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/Eoliann_Windows_Tools_Installer.exe?displayAssetName=true&style=plastic&color=red)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/total?style=plastic)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)

# 🟢 Eoliann Windows Tools

A quick toolkit for Windows 11, written in Rust + egui, with a retro hacker fluorescent green UI 🟩.  
Includes useful administration commands and tweaks for Windows 11.

#### New design:
- Yellow color in titles
- Icons in the menu

Now install it on your system!

![ewt](https://github.com/user-attachments/assets/b8d5978f-84e3-4db8-b7ae-c563181eaf05)


---

<img width="456" height="403" alt="EWT_1" src="https://github.com/user-attachments/assets/2b430312-846a-433f-9559-e73c44bb264f" />

## 🚀 Features

#### Update to new version
<img width="1402" height="932" alt="update-1 1 7-to-1 1 8" src="https://github.com/user-attachments/assets/4d442ac3-679d-4c51-9f00-8f6383eea4b6" />

### INFO
<img width="1920" height="1032" alt="1 0" src="https://github.com/user-attachments/assets/75c2c5f9-de30-4f02-a370-8b190e9cc8db" />
<img width="1920" height="1032" alt="1 1" src="https://github.com/user-attachments/assets/8f183082-a31e-40f3-9eeb-961619b42b0f" />

- `whoami` → displays the current user
- `ipconfig` → display network settings
- `systeminfo` → system details
- `tasklist` → active processes
- `System Information` → information about the entire system: PC Name, Windows Edition, Processor, RAM, Storage, Graphics Card, Installed apps, Open processes, Services running, and Network.
- `About` → all about application: developer, version, website

### TOOLS
<img width="1920" height="1032" alt="2 1" src="https://github.com/user-attachments/assets/e4c64496-afcc-4b92-b322-2a0479213984" />
<img width="1920" height="1032" alt="2 2" src="https://github.com/user-attachments/assets/086568ad-b1d2-4597-92ce-0fb77a85bb47" />

#### Context menu
- `Toggle Context Menu` → instantly switch between the **Win11 ↔ Classic** context menu (with Explorer restart)
#### Maintenance
- `Disk Cleanup C:` → start disk cleanup only C: partition
- `Disk CleanUp all Partitions` → start disk cleanup for all partitions
- `Empty Recycle Bin` → initiates emptying of the trash bin
- `Clean Temporary Files` → launches temporary file cleanup
- `Empty Prefetch files` → initiates emptying prefetch files
- `Network Reset` → completely reset the network (Winsock + IP)
- `Verify System Integrity (SFC + DSIM)` → checks whether the system integrity is correct and, if not, repairs corrupted files
#### Essential Tweaks
- `Disable ConsumerFeatures` → prevents automatic installation of Store apps/games for the signed-in user
- `Enable ConsumerFeatures` → restores policy to allow Store consumer features 
- `Disable Telemetry` → disables various telemetry options, popups, and other annoyances in Edge
- `Disable Location Tracking` → disables Location Tracking
- `Disable Wifi-Sense` → Wifi Sense is a spying service that phones home all nearby scanned wifi networks and your current geo location
- `Enable End Task With Right Click` → enables option to end task when right clicking a program in the taskbar
- `Undo End Task With Right Click` → disables option to end task when right clicking a program in the taskbar
- `Disable Recall` → disables MS Recall built into Windows since 24H2
- `Enable Recall` → enables MS Recall built into Windows since 24H2
- `Debloat Edge` → disables various telemetry options, popups, and other annoyances in Edge
- `Create Restore Point` → creates a restore point at runtime in case a revert is needed from Eoliann Windows Tools modifications
- `Disable Activity History` → erases recent docs, clipboard, and run history; disables Activity History features
- `Enable Activity History` → restores Activity History policies to allow activity collection
- `Disable Storage Sense` → prevents Storage Sense from automatically deleting temporary files for the current user
- `Enable Storage Sense` → restores Storage Sense automatic cleanup for the current user
- `Show/Hide Hidden Files` → Toggles visibility of hidden files and folders in Explorer.
- `Show File Extensions` → Toggle showing file extensions for known file types.
#### Advanced Tweaks
- `Adobe Network Block` → reduce user interruptions by selectively blocking connections to Adobe’s activation and telemetry servers. Credit: Ruddernation-Designs
- `Adobe Debloat` → manages Adobe Services, Adobe Desktop Service, and Acrobat Updates
- `Disable Microsoft Copilot` → disables MS Copilot AI built into Windows since 23H2
- `Set Display for Performance` → sets the system preferences to performance. You can do this manually with sysdm.cpl as well
- `Set Time to UTC (Dual Boot)` → essential for dual-boot systems: syncs Windows with Linux hardware clock (UTC)
- `Restore Time to Local` → restores Windows default: hardware clock treated as local time
- `Remove OneDrive` → moves OneDrive files to default home folders and uninstalls OneDrive
- `Install OneDrive (Restore)` → installs OneDrive using winget (undo)
- `Toggle Explorer Tabs` → enabling and disabling tabs in Explorer (My Computer)
- `Run OO Shutup 10` → downloads and launches OO Shutup 10 (third-party executable)
- `Set DNS` → set the DNS for the network card

#### Network Tools
- `Set up insecure guest logins + disable SMB signing` → RequireSecuritySignature = 0 & AllowInsecureGuestAuth = 1 (in both registry locations)

#### Power Plans
- `High Performance` → maximizes performance at the cost of higher power usage
- `Balanced` → default Windows plan (best for most users)
- `Power Saver` → reduces system performance to save battery life
- `Set Hibernation as default (laptops)` → most laptops with connected standby drain battery, this sets hibernation as default
- `Restore Hibernation defaults` → restores registry Attributes and turns hibernation off
#### Power Tweaks
- `Disable Sleep` → prevents Windows from going into sleep mode
- `Disable HDD/SSD Timeout` → prevents hard drives and SSDs from powering down after inactivity
- `Disable Monitor Timeout` → prevents the monitor from turning off automatically
#### Updates Settings
- `Default Settings`  → resets Windows Update settings to default
- `Security Settings`  → sets Windows Update to recommended/security-focused settings
- `Disable All Updates`  → disables Windows Update (advanced users only)

#### Group Policy Editor (Windows Home)
- `Enable gpedit.msc on Windows Home`  → Check your Windows version and, if it is Home, activate gpedit.msc in the system if it is not already activated.

#### Security
- `Password Expire`  → Enable/Disable Password Expire on Windows 10/11 Pro & Windows 10/11 Home.

### DISK HEALTH
<img width="1920" height="1032" alt="3 0" src="https://github.com/user-attachments/assets/287f6e4a-ac04-4fc9-911f-a11d978d593f" />
<img width="1920" height="1032" alt="3 1" src="https://github.com/user-attachments/assets/11c29734-d0e6-429e-a83b-c02ef008efbe" />

Displays storage memory information such as: temperature, health, performance, Power on time (total number of hours of use), Estimated remaining lifetime, Lifetime writes in GB and bytes.

### INSTALL
<img width="1920" height="1032" alt="4 0" src="https://github.com/user-attachments/assets/2a8c2ed2-ca82-4ee2-8e5e-2c6eebbb72f6" />
<img width="1920" height="1032" alt="4 1" src="https://github.com/user-attachments/assets/7d088e7e-da94-45b1-95c2-83d4d31e035f" />

#### Install application
![install-app](https://github.com/user-attachments/assets/1d50fdc2-eea6-4b72-863f-e3e7f44a330f)

#### Uninstall installed application
![uninstall-app](https://github.com/user-attachments/assets/55bccd07-4b97-4c57-bca0-858f9cf31e0b)

- `Install selections, Uninstall selections, Update selections, Clear selections, Upgrade All Applications, Reinstall winget & search field to filter apps`
- `Browsers`
- `Communications`
- `Development`
- `Document`
- `Games`
- `Microsoft Tools`
- `Multimedia Tools`
- `Pro Tools`
- `Utilities`

For Google Chrome users:
- `Install/Reinstall Google Chrome` → if the browser is not in the system
- `Install Chrome extensions` → install Chrome extensions from the Chrome Web Store
  
### WINDOWS APP REMOVAL
<img width="1920" height="1032" alt="5 0" src="https://github.com/user-attachments/assets/341ddf32-3b31-4407-8176-443d809ade9c" />
<img width="1920" height="1032" alt="5 1" src="https://github.com/user-attachments/assets/b8100f5d-7e22-4ad3-9e40-082c29987a87" />
<img width="1920" height="1032" alt="5 2" src="https://github.com/user-attachments/assets/3188e785-2cbd-4bb0-8038-4143e4b82561" />
<img width="1920" height="1032" alt="5 3" src="https://github.com/user-attachments/assets/eba29c8c-c281-4d84-b5a8-c666bfb0c8fe" />

- `Communication` → remove default applications that come with Windows installation, such as: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar, and others.
- `Media & Creativity` → removes default applications that come with Windows installation, such as: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Mixed Reality Portal
- `Microsoft Apps` → removes default applications that come with Windows installation, such as: Office Hub, OneNote, Sway, Sticky Notes, Family Safety
- `Bing Apps` → removes default applications that come with Windows installation, such as: Bing Weather, Bing Sports, Bing Finance, Bing News
- `Games` → removes default applications that come with Windows installation, such as: Solitaire Collection, Minecraft for Windows
- `System & Misc` → remove default applications that come with Windows installation, such as: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder
- `Other` → remove default applications that come with Windows installation, such as: Print 3D, Office-related

### CUSTOMIZE PREFERENCES
<img width="1920" height="1032" alt="6 0" src="https://github.com/user-attachments/assets/8df00031-6e01-4f4f-a759-c5bae0912c50" />

- `General Preferences` → Start with Windows, Enable advanced tooltips, Auto-check for updates
- `Features / Tweaks` → Mouse Acceleration, NumLock on startup, Taskbar Search Button, Taskbar Widgets, Snap Windows on startup, Sticky Keys on startup, Task View button, Verbose Logon Messages (system), BitLocker protection
- `Bitlocker on/off all partition` → activate/deactivate Bitlocker for one or all partition

### HEALT
#### Power, cleanup and diagnostics.
<img width="1920" height="1032" alt="7 0" src="https://github.com/user-attachments/assets/b2288622-9082-49d6-b786-32f7a5185add" />

- `Hibernation` → Enable or disable Windows hibernation.
- `Cleanup` → Storage settings and System Restore cleanup.
- `System restore` → Remove restore points created
- `Battery report` → Generate the Windows battery report and save it to Documents.
- `Memory diagnostic` → Launch Windows Memory Diagnostic.

### PERFORMANCE
#### Power, graphics and startup settings.
<img width="1920" height="1032" alt="8 0" src="https://github.com/user-attachments/assets/b8ba27fd-e918-4bff-a655-7e89c7a5c0bd" />

- `Ultimate performance power plan` → Switch between the app-managed Ultimate plan and Balanced.
- `HAGS (hardware-accelerated GPU scheduling)` → May reduce latency and CPU overhead. Restart usually required.
- `VBS (virtualization-based security)` → Turns VBS / memory-integrity-related registry flags on or off. Restart required.
- `Startup apps` → Manage classic Run startup entries.
- `Relaunch apps` → Automatically save restartable apps and relaunch them after sign-in.
- `Background apps` → Global background-app permission switch for the current user.
- `Activity history` → Uses the existing Activity History policies already present in the app.
- `Visual settings`:
  - `Optimize visual effects for performance`: Uses the existing performance visual profile from the app.;
  - `Transparency`: Windows transparency effects for the current user.
- `Gaming settings`:
  - `Game mode`: Windows Game Mode.
  - `Windowed mode optimizations`: Optimizations for windowed games. Restart the game after changes.
  - `Background recording`: Xbox Game Bar / Game DVR background capture.
  - `Superfetch`: Controls the SysMain service.

- `Search indexing` → Controls the Windows Search indexing service.
- `Delivery optimization` → Controls Delivery Optimization sharing behavior.
- `Network adapter onboard processor` → Controls TCP task offload.

### QUICK KEYS
<img width="1920" height="1032" alt="9 0" src="https://github.com/user-attachments/assets/17e313e7-111c-4207-9f0b-b05cea12f6eb" />
<img width="1920" height="1032" alt="9 1" src="https://github.com/user-attachments/assets/a1c7f9df-635a-4de6-ae74-c073a913dc2d" />

- `Shortcuts` → Shortcuts for some important functions such as: Win + X, Win + R, Win + I, Regedit\Registry Editor, gpedit.msc\Group Policy

### SETTINGS
<img width="1920" height="1032" alt="10 0" src="https://github.com/user-attachments/assets/a9d7663e-cf5a-4138-83fb-8ff8cde08570" />
<img width="1920" height="1032" alt="10 1" src="https://github.com/user-attachments/assets/98c2ad52-7bae-48a3-bc4c-155d977453a1" />

- `Switch to Light Mode / Switch to Dark Mode` → switch between Windows Light and Dark themes
- `Display Current alignment` → Display Current alignment of taskbar elements (Center or Left)
- `Center Taskbar Items` → center the taskbar items (Windows 11)
- `Left Taskbar Items` → align the taskbar items to the left (classic)
- `Create new local user account` - create local account for new user
- `About` → version information + GitHub link

---

## 🎨 Design
- Visual UI with tabs **INFO, TOOLS, WINAPP REMOVAL SETTINGS**
- **Black** background
- Text and buttons **fluorescent green (#39FF14)**
- ASCII Retro Hacker Logo 🟩 on start (JetBrains Mono font)

---

## 🛠️ Build and run

### Clone repo
```powershell
git clone https://github.com/eoliann/Eoliann_Windows_Tools
cd Eoliann_Windows_Tools
```

---

## 🛠 Bugs and Errors
- If you discover any bugs or errors, please send them to my channel at **[Discord](https://discord.com/channels/977086560782663680/1416056622069055580)**
- Documentation <a href="https://discord.com/channels/977086560782663680/1416056737047253156" target="_blank"><img src="https://img.shields.io/badge/documentation-available-blue?logo=readthedocs" alt="Documentation"/></a>

---

## 💖 Support
- To morally and mentally support the project, make sure to leave a ⭐️!
- Support this project with a donation on:
  - Revolut [![Donate](https://img.shields.io/badge/Donate-Revolut-purple)](http://revolut.me/adriannm9)
  - PayPal [![Donate](https://img.shields.io/badge/Donate-PayPal-blue)](https://www.paypal.com/donate/?hosted_button_id=PTH2EXUDS423S)

