![Stars](https://img.shields.io/github/stars/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/Eoliann_Windows_Tools?style=flat-square)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE.md)
![Release Date](https://img.shields.io/github/release-date/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Total Downloads](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/total?style=flat-square)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/total?style=flat-square)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)

![OS](https://badgen.net/badge/icon/windows?icon=windows&label=OS)
![Lang](https://badgen.net/static/Lang/Rust/orange)
![Stars](https://badgen.net/github/stars/eoliann/Eoliann_Windows_Tools/)
![Watchers](https://badgen.net/github/watchers/eoliann/Eoliann_Windows_Tools/)
[![Releases](https://badgen.net/github/releases/eoliann/Eoliann_Windows_Tools)](https://github.com/eoliann/Eoliann_Windows_Tools/releases)
![Last Release](https://badgen.net/github/tag/eoliann/Eoliann_Windows_Tools)
[![Downloads latest](https://badgen.net/github/assets-dl/eoliann/Eoliann_Windows_Tools)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)
[![Donate](https://badgen.net/static/Donate/PayPal/orange)](https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G)
[![Donate](https://badgen.net/static/Donate/Revolut/f2a)](http://revolut.me/adriannm9)




# 🟢 Eoliann Windows Tools

A quick toolkit for Windows 11, written in Rust + egui, with a retro hacker fluorescent green UI 🟩.  
Includes useful administration commands and tweaks for Windows 11.


Now install it on your system!

![ewt](https://github.com/user-attachments/assets/b8d5978f-84e3-4db8-b7ae-c563181eaf05)


---

<img width="456" height="403" alt="EWT_1" src="https://github.com/user-attachments/assets/2b430312-846a-433f-9559-e73c44bb264f" />

## 🚀 Features

### INFO
<img width="1402" height="932" alt="EWT_info_update" src="https://github.com/user-attachments/assets/ee3a0082-3255-4a33-9985-97faf3795297" />
<img width="1402" height="932" alt="info-1 1 6" src="https://github.com/user-attachments/assets/2c9c5547-1b69-407d-bf4b-a69e9f1275c8" />

- `whoami` → displays the current user
- `ipconfig` → display network settings
- `systeminfo` → system details
- `tasklist` → active processes

### TOOLS
<img width="1754" height="932" alt="tools-1 1 6" src="https://github.com/user-attachments/assets/4caa7544-0a58-43aa-b568-42044a3f0ad9" />

#### Context menu
- `Toggle Context Menu` → instantly switch between the **Win11 ↔ Classic** context menu (with Explorer restart)
#### Maintenance
- `Disk Cleanup` → start disk cleanup
- `Empty Recycle Bin` → initiates emptying of the trash bin
- `Clean Temporary Files` → launches temporary file cleanup
- `Network Reset` → completely reset the network (Winsock + IP)
- `Verify System Integrity (SFC + DSIM)` → checks whether the system integrity is correct and, if not, repairs corrupted files
- `Reset Windows Update` → Attempts to repair Windows Update. Aggressive mode runs chkdsk, SFC and DISM and may take a long time.
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
<img width="1402" height="987" alt="EWT_install" src="https://github.com/user-attachments/assets/92dd36bc-35c8-4242-8613-2506daa263e7" />
<img width="1402" height="932" alt="EWT_install_2" src="https://github.com/user-attachments/assets/d4f47a68-8650-4338-b873-e61c44349f25" />
<img width="1402" height="932" alt="EWT_install_3" src="https://github.com/user-attachments/assets/b059bb25-24a0-4825-a377-e6a78177f8e4" />
<img width="1402" height="932" alt="EWT_install_4" src="https://github.com/user-attachments/assets/ed990efd-4c0d-4bdf-ab64-8e361ef8ddd3" />
<img width="1402" height="932" alt="EWT_install_5" src="https://github.com/user-attachments/assets/527c9273-22db-4479-bb74-dafee9799530" />
<img width="1402" height="932" alt="EWT_install_6" src="https://github.com/user-attachments/assets/e0eca63a-9952-450c-a77f-76c3be858257" />
<img width="1402" height="1027" alt="install-1 1 6" src="https://github.com/user-attachments/assets/83c8507d-98c6-48ea-a2d3-172ed6593961" />

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
<img width="1402" height="1027" alt="winappremoval-1 1 6" src="https://github.com/user-attachments/assets/93aeb49d-42e9-48e7-980a-3a840534c6f5" />

- `Communication` → remove default applications that come with Windows installation, such as: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar, and others.
- `Media & Creativity` → removes default applications that come with Windows installation, such as: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Print 3D, Mixed Reality Portal
- `Microsoft Apps` → removes default applications that come with Windows installation, such as: Office Hub, OneNote, Sway, Sticky Notes, Family Safety
- `Bing Apps` → removes default applications that come with Windows installation, such as: Bing Weather, Bing Sports, Bing Finance, Bing News
- `Games` → removes default applications that come with Windows installation, such as: Solitaire Collection, Minecraft for Windows
- `System & Misc` → remove default applications that come with Windows installation, such as: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder

### SETTINGS
<img width="1402" height="1027" alt="settings-1 1 6" src="https://github.com/user-attachments/assets/c1d1435d-feec-496b-9bdc-dc452940ca7d" />

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

## 🛠 Bug-uri i erori / Bugs and Errors
- If you discover any bugs or errors, please send them to my channel at **[Discord](https://discord.com/channels/977086560782663680/1416056622069055580)**
- Documentation <a href="https://discord.com/channels/977086560782663680/1416056737047253156" target="_blank"><img src="https://img.shields.io/badge/documentation-available-blue?logo=readthedocs" alt="Documentation"/></a>

---

## 💖 Support
- To morally and mentally support the project, make sure to leave a ⭐️!
- Support this project with a donation on:
  - Revolut [![Donate](https://img.shields.io/badge/Donate-Revolut-purple)](http://revolut.me/adriannm9)
  - PayPal [![Donate](https://img.shields.io/badge/Donate-PayPal-blue)](https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G)

