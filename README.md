![Stars](https://img.shields.io/github/stars/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/Eoliann_Windows_Tools?style=flat-square)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE.md)
![Release Date](https://img.shields.io/github/release-date/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Total Downloads](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/total?style=flat-square)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/total?style=flat-square)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)





# 🟢 Eoliann Windows Tools

A quick toolkit for Windows 11, written in Rust + egui, with a retro hacker fluorescent green UI 🟩.  
Includes useful administration commands and tweaks for Windows 11.

---

<img width="456" height="403" alt="EWT_1" src="https://github.com/user-attachments/assets/2b430312-846a-433f-9559-e73c44bb264f" />

## 🚀 Features

### INFO
<img width="1402" height="932" alt="1 0 9_info" src="https://github.com/user-attachments/assets/6997729f-bcf7-42e1-a669-0450507bc6b8" />

- `whoami` → displays the current user
- `ipconfig` → display network settings
- `systeminfo` → system details
- `tasklist` → active processes

### TOOLS
<img width="1402" height="932" alt="1 0 9_tools" src="https://github.com/user-attachments/assets/e86dce7f-e46b-4054-b678-19b1c5cc3dc7" />

- `Toggle Context Menu` → instantly switch between the **Win11 ↔ Classic** context menu (with Explorer restart)
- `Disk Cleanup` → start disk cleanup
- `Empty Recycle Bin` → initiates emptying of the trash bin
- `Clean Temporary Files` → launches temporary file cleanup
- `Network Reset` → completely reset the network (Winsock + IP)
- `Verify System Integrity (SFC + DSIM)` → checks whether the system integrity is correct and, if not, repairs corrupted files
- `Disable Telemetry` → disables various telemetry options, popups, and other annoyances in Edge
- `Disable Location Tracking` → disables Location Tracking
- `Disable Wifi-Sense` → Wifi Sense is a spying service that phones home all nearby scanned wifi networks and your current geo location
- `Enable End Task With Right Click` → enables option to end task when right clicking a program in the taskbar
- `Undo End Task With Right Click` → disables option to end task when right clicking a program in the taskbar
- `Disable Recall` → disables MS Recall built into Windows since 24H2
- `Enable Recall` → enables MS Recall built into Windows since 24H2
- `Debloat Edge` → disables various telemetry options, popups, and other annoyances in Edge
- `Adobe Network Block` → reduce user interruptions by selectively blocking connections to Adobe’s activation and telemetry servers. Credit: Ruddernation-Designs
- `Adobe Debloat` → manages Adobe Services, Adobe Desktop Service, and Acrobat Updates
- `Disable Microsoft Copilot` → disables MS Copilot AI built into Windows since 23H2
- `Set Display for Performance` → sets the system preferences to performance. You can do this manually with sysdm.cpl as well
- `Set DNS` → set the DNS for the network card
- `Power Plan Switcher` → quickly switch between High Performance, Balanced, and Power Saver modes
- `Power Tweaks` → Disable Sleep (no standby/sleep/hibernate), Disable HDD/SSD turn off, Disable Monitor turn off, and works on both battery (DC) and mains power (AC)

### INSTALL
<img width="1402" height="932" alt="1 0 9_install" src="https://github.com/user-attachments/assets/b8334a1b-faae-487e-969a-3a81ec369bc1" />
<img width="1402" height="932" alt="1 0 9_install_1" src="https://github.com/user-attachments/assets/f517e296-1e35-4687-bc12-d7a8896832ba" />

- `Install selections, Uninstall selections, Update selections, Clear selections, Upgrade All Applications, Reinstall winget`
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
<img width="1402" height="932" alt="1 0 9_winapp_remover" src="https://github.com/user-attachments/assets/b5e01c69-22ef-40a0-b344-2d355601df6a" />

- `Communication` → remove default applications that come with Windows installation, such as: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar, and others.
- `Media & Creativity` → removes default applications that come with Windows installation, such as: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Print 3D, Mixed Reality Portal
- `Microsoft Apps` → removes default applications that come with Windows installation, such as: Office Hub, OneNote, Sway, Sticky Notes, Family Safety
- `Bing Apps` → removes default applications that come with Windows installation, such as: Bing Weather, Bing Sports, Bing Finance, Bing News
- `Games` → removes default applications that come with Windows installation, such as: Solitaire Collection, Minecraft for Windows
- `System & Misc` → remove default applications that come with Windows installation, such as: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder

### SETTINGS
<img width="1402" height="932" alt="1 0 9_settings" src="https://github.com/user-attachments/assets/58f7c75c-dc01-4428-ab1d-68b27524ed28" />

- `Dark / Light Theme` → switch between Windows Light and Dark themes
- `Open Display Settings` → opens the window with the system display options
- `About` → version information + GitHub link

---

## 🎨 Design
- Visual UI with tabs **INFO, TOOLS, WINAPP REMOVAL SETTINGS**
- **Black** background
- Text and buttons **fluorescent green (#39FF14)**
- ASCII Retro Hacker Logo 🟩 la pornire (JetBrains Mono font)

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
- Support this project with a donation
  - Revolut [![Donate](https://img.shields.io/badge/Donate-Revolut-purple)](http://revolut.me/adriannm9)
  - PayPal [![Donate](https://img.shields.io/badge/Donate-PayPal-blue)](https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G)

