![Followers](https://img.shields.io/github/followers/eoliann?style=plastic&color=green)
![Watchers](https://img.shields.io/github/watchers/eoliann/wup-web?style=plastic)
![Stars](https://img.shields.io/github/stars/eoliann/Eoliann_Windows_Tools?style=plastic)

![Release Date](https://img.shields.io/github/release-date/eoliann/Eoliann_Windows_Tools?style=plastic)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/Eoliann_Windows_Tools?style=plastic)

![Total Downloads](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/total?style=plastic)
![](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/eoliann_windows_tools_w11.exe?displayAssetName=true&style=plastic&color=green)
![](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/Eoliann_Windows_Tools_Installer.exe?displayAssetName=true&style=plastic&color=red)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/total?style=plastic)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=plastic)](LICENSE.md)
![OS](https://img.shields.io/badge/OS-Windows-blue?style=plastic)
![Lang](https://img.shields.io/badge/Lang-Rust-magenta?style=plastic)

[![Donate](https://img.shields.io/badge/Donate-PayPal-blue?style=plastic)](https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G?style=plastic)
[![Donate](https://img.shields.io/badge/Donate-Revolut-8A2BE2?style=plastic)](http://revolut.me/adriannm9?style=plastic)



# 🟢 Eoliann Windows Tools

A quick toolkit for Windows 11, written in Rust + egui, with a retro hacker fluorescent green UI 🟩.  
Includes useful administration commands and tweaks for Windows 11.


Now install it on your system!

![ewt](https://github.com/user-attachments/assets/b8d5978f-84e3-4db8-b7ae-c563181eaf05)


---

<img width="456" height="403" alt="EWT_1" src="https://github.com/user-attachments/assets/2b430312-846a-433f-9559-e73c44bb264f" />

## 🚀 Features

### Update to new version
<img width="1402" height="932" alt="update-1 1 7-to-1 1 8" src="https://github.com/user-attachments/assets/0b2e984e-4f61-4f86-84f3-fdb6c870479b" />


### INFO
<img width="1402" height="932" alt="info-1 1 8" src="https://github.com/user-attachments/assets/e76e2d3a-2b71-472f-b0ff-ebf1c1c0c136" />

- `whoami` → displays the current user
- `ipconfig` → display network settings
- `systeminfo` → system details
- `tasklist` → active processes


### TOOLS
<img width="1402" height="932" alt="tools-1 1 8" src="https://github.com/user-attachments/assets/2b1194ea-279c-4cf1-b838-4032de85fa3b" />

#### Context menu
- `Toggle Context Menu` → instantly switch between the **Win11 ↔ Classic** context menu (with Explorer restart)
#### Maintenance
- `Disk Cleanup` → start disk cleanup
- `Empty Recycle Bin` → initiates emptying of the trash bin
- `Clean Temporary Files` → launches temporary file cleanup
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
- `Run OO Shutup 10` → downloads and launches OO Shutup 10 (third-party executable)
- `Set DNS` → set the DNS for the network card
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

### INSTALL
<img width="1402" height="932" alt="install-1 1 8" src="https://github.com/user-attachments/assets/fadca38c-1a76-4538-a0d4-0f1e72b7fbf5" />


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
  
### WINDOWS APP REMOVAL
<img width="1402" height="932" alt="winapp-removal-1 1 8" src="https://github.com/user-attachments/assets/1928d8ec-b782-4e04-b6c0-4028cf0a0b2a" />

- `Communication` → remove default applications that come with Windows installation, such as: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar, and others.
- `Media & Creativity` → removes default applications that come with Windows installation, such as: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Print 3D, Mixed Reality Portal
- `Microsoft Apps` → removes default applications that come with Windows installation, such as: Office Hub, OneNote, Sway, Sticky Notes, Family Safety
- `Bing Apps` → removes default applications that come with Windows installation, such as: Bing Weather, Bing Sports, Bing Finance, Bing News
- `Games` → removes default applications that come with Windows installation, such as: Solitaire Collection, Minecraft for Windows
- `System & Misc` → remove default applications that come with Windows installation, such as: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder

### Customiza Preferences
<img width="1402" height="932" alt="customize-preferences-1 1 8" src="https://github.com/user-attachments/assets/60349f14-609e-4425-96a9-4113292d76e4" />
- `General Preferences` → Start with Windows, Enable advanced tooltips, Auto-check for updates
- `Features` → Mouse Acceleration, NumLock on startup, Taskbar Search Button, Taskbar Widgets, Snap Windows on startup, Sticky Keys on startup, Task View button, Verbose Logon Messages (system), BitLocker protection


### SETTINGS
<img width="1402" height="932" alt="settings-1 1 8" src="https://github.com/user-attachments/assets/16c8c393-9863-435f-8741-73d26fdac263" />

- `Switch to Light Mode / Switch to Dark Mode` → switch between Windows Light and Dark themes
- `Display Current alignment` → Display Current alignment of taskbar elements (Center or Left)
- `Center Taskbar Items` → center the taskbar items (Windows 11)
- `Left Taskbar Items` → align the taskbar items to the left (classic)
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
  - PayPal [![Donate](https://img.shields.io/badge/Donate-PayPal-blue)](https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G)

