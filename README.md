![Stars](https://img.shields.io/github/stars/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/Eoliann_Windows_Tools?style=flat-square)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE.md)
![Release Date](https://img.shields.io/github/release-date/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Total Downloads](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/total?style=flat-square)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/total?style=flat-square)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)





# 🟢 Eoliann Windows Tools

[RO]
Un toolkit rapid pentru Windows 11, scris în Rust + egui, cu UI verde fluorescent retro hacker 🟩.  
Include comenzi utile de administrare și tweak-uri pentru Windows 11.

[EN]
A quick toolkit for Windows 11, written in Rust + egui, with a retro hacker fluorescent green UI 🟩.  
Includes useful administration commands and tweaks for Windows 11.

---

<img width="456" height="403" alt="EWT_1" src="https://github.com/user-attachments/assets/2b430312-846a-433f-9559-e73c44bb264f" />

## 🚀 Funcționalități / Features

### INFO
<img width="1002" height="732" alt="1" src="https://github.com/user-attachments/assets/eafdb955-e54c-4016-9e83-a2c995b2baaa" />

- `whoami` → afișează utilizatorul curent / displays the current user
- `ipconfig` → afișează setările de rețea / display network settings
- `systeminfo` → detalii despre sistem / system details
- `tasklist` → procese active / active processes

### TOOLS
<img width="1252" height="914" alt="Screenshot 2025-09-11 170250" src="https://github.com/user-attachments/assets/4825115b-dc2d-4365-b574-f17b97953275" />

- `Toggle Context Menu` → schimbă instant între context menu **Win11 ↔ Classic** (cu restart Explorer) / Instantly switch between the **Win11 ↔ Classic** context menu (with Explorer restart)
- `Disk Cleanup` → lansează curățarea de disc / start disk cleanup
- `Empty Recycle Bin` → lansează golirea coșului de gunoi / initiates emptying of the trash bin
- `Clean Temporary Files` → lansează curățarea fișierelor temporare / launches temporary file cleanup
- `Network Reset` → resetează complet rețeaua (Winsock + IP) / completely reset the network (Winsock + IP)
- `Verify System Integrity (SFC + DSIM)` → verifică dacă integritatea sistemului este corectă și, dacă nu, repară fișierele corupte / checks whether the system integrity is correct and, if not, repairs corrupted files
- `Disable Telemetry` → dezactivează diverse opțiuni de telemetrie, ferestre pop-up și alte elemente deranjante din Edge / disables various telemetry options, popups, and other annoyances in Edge
- `Disable Location Tracking` → dezactivează urmărirea locației / disables Location Tracking
- `Disable Wifi-Sense` → Wifi Sense este un serviciu de spionaj care transmite către producător toate rețelele wifi scanate din apropiere și locația dvs. geografică actuală / Wifi Sense is a spying service that phones home all nearby scanned wifi networks and your current geo location
- `Enable End Task With Right Click` → activează opțiunea de a închide sarcina atunci când faceți clic dreapta pe un program din bara de sarcini / enables option to end task when right clicking a program in the taskbar
- `Undo End Task With Right Click` → dezactivează opțiunea de a închide sarcina atunci când faceți clic dreapta pe un program din bara de sarcini / disables option to end task when right clicking a program in the taskbar
- `Disable Recall` → dezactivează MS Recall integrat în Windows începând cu versiunea 24H2 / disables MS Recall built into Windows since 24H2
- `Enable Recall` → activează MS Recall integrat în Windows începând cu versiunea 24H2 / enables MS Recall built into Windows since 24H2
- `Debloat Edge` → dezactivează diverse opțiuni de telemetrie, ferestre pop-up și alte elemente deranjante din Edge / disables various telemetry options, popups, and other annoyances in Edge
- `Adobe Network Block` → reduceți întreruperile utilizatorilor prin blocarea selectivă a conexiunilor la serverele de activare și telemetrie Adobe. Credit: Ruddernation-Designs
/ reduce user interruptions by selectively blocking connections to Adobe’s activation and telemetry servers. Credit: Ruddernation-Designs
- `Adobe Debloat` → gestionează serviciile Adobe, serviciul Adobe Desktop și actualizările Acrobat / manages Adobe Services, Adobe Desktop Service, and Acrobat Updates
- `Disable Microsoft Copilot` → dezactivează MS Copilot AI integrat în Windows începând cu versiunea 23H2 / disables MS Copilot AI built into Windows since 23H2
- `Set Display for Performance` → setează preferințele sistemului pentru performanță. Puteți face acest lucru manual și cu sysdm.cpl / sets the system preferences to performance. You can do this manually with sysdm.cpl as well
- `Power Plan Switcher` → schimbă rapid între High Performance, Balanced, Power Saver / quickly switch between High Performance, Balanced, and Power Saver modes
- `Power Tweaks` → Disable Sleep (fără standby/sleep/hibernate), Disable HDD/SSD turn off, Disable Monitor turn off și funcționează atât pe baterie (DC) cât și la priză (AC) / Disable Sleep (no standby/sleep/hibernate), Disable HDD/SSD turn off, Disable Monitor turn off, and works on both battery (DC) and mains power (AC)

### INSTALL
<img width="1002" height="732" alt="2" src="https://github.com/user-attachments/assets/d26ed7ea-be21-41f6-93a2-b6ce629a9012" />

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
<img width="1006" height="766" alt="EWT_winapp_removal" src="https://github.com/user-attachments/assets/a1026b80-a125-4f77-9650-8558d88ffb35" />

- `Communication` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar și altele / remove default applications that come with Windows installation, such as: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar, and others.
- `Media & Creativity` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Print 3D, Mixed Reality Portal / removes default applications that come with Windows installation, such as: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Print 3D, Mixed Reality Portal
- `Microsoft Apps` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Office Hub, OneNote, Sway, Sticky Notes, Family Safety / removes default applications that come with Windows installation, such as: Office Hub, OneNote, Sway, Sticky Notes, Family Safety
- `Bing Apps` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Bing Weather, Bing Sports, Bing Finance, Bing News / removes default applications that come with Windows installation, such as: Bing Weather, Bing Sports, Bing Finance, Bing News
- `Games` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Solitaire Collection, Minecraft for Windows / removes default applications that come with Windows installation, such as: Solitaire Collection, Minecraft for Windows
- `System & Misc` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder / remove default applications that come with Windows installation, such as: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder

### SETTINGS
<img width="1252" height="914" alt="3" src="https://github.com/user-attachments/assets/ee3963ac-f848-4741-8b7a-24e3b85b824c" />

- `Dark / Light Theme` → schimbă între temele Light și Dark ale Windows / switch between Windows Light and Dark themes
- `Open Display Settings` → deschide fereastra cu opțiunile de afișare ale sistemului / opens the window with the system display options
- `About` → informații despre versiune + link GitHub / version information + GitHub link

---

## 🎨 Design
- UI vizual cu tab-uri **INFO, TOOLS, WINAPP REMOVAL SETTINGS** / Visual UI with tabs **INFO, TOOLS, WINAPP REMOVAL SETTINGS**
- Fundal **negru** / **Black** background
- Text și butoane **verde fluorescent (#39FF14)** / Text and buttons **fluorescent green (#39FF14)**
- ASCII Logo retro hacker 🟩 la pornire (JetBrains Mono font) / ASCII Retro Hacker Logo 🟩 la pornire (JetBrains Mono font)

---

## 🛠️ Build și rulare / Build and run

### Clonează repo / Clone repo
```powershell
git clone https://github.com/eoliann/Eoliann_Windows_Tools
cd Eoliann_Windows_Tools
```

---

## 🛠 Bug-uri i erori / Bugs and Errors
- În cazul în care descoperiți bug-uri sau erori, vă rog să le trimiteți pe canalul meu de **[Discord](https://discord.com/channels/977086560782663680/1416050555612172418)** / If you discover any bugs or errors, please send them to my channel at **[Discord](https://discord.com/channels/977086560782663680/1416050555612172418)**
- Documentație / Documentation <a href="https://discord.com/channels/977086560782663680/1416050270139322388" target="_blank"><img src="https://img.shields.io/badge/documentation-available-blue?logo=readthedocs" alt="Documentation"/></a>




---
