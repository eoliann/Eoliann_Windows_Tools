![Stars](https://img.shields.io/github/stars/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/Eoliann_Windows_Tools?style=flat-square)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE.md)
![Release Date](https://img.shields.io/github/release-date/eoliann/Eoliann_Windows_Tools?style=flat-square)
![Total Downloads](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/total?style=flat-square)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/Eoliann_Windows_Tools/latest/total?style=flat-square)](https://github.com/eoliann/Eoliann_Windows_Tools/releases/latest/download/eoliann_windows_tools_w11.exe)





# 🟢 Eoliann Windows Tools

Un toolkit rapid pentru Windows 11, scris în Rust + egui, cu UI verde fluorescent retro hacker 🟩.  
Include comenzi utile de administrare și tweak-uri pentru Windows 11.

---
<img width="460" height="495" alt="Screenshot 2025-08-26 124326" src="https://github.com/user-attachments/assets/9b195b58-4f9f-44a2-a7f9-85168b0a3634" />

## 🚀 Funcționalități

### INFO
<img width="1002" height="732" alt="Screenshot 2025-08-27 175721" src="https://github.com/user-attachments/assets/a8517a1d-d6cc-44fb-a9a8-1ca73c39bb2e" />

- `whoami` → afișează utilizatorul curent
- `ipconfig` → afișează setările de rețea
- `systeminfo` → detalii despre sistem
- `tasklist` → procese active
- `About` → versiune + link GitHub

### TOOLS
<img width="1006" height="736" alt="Screenshot 2025-08-27 124652" src="https://github.com/user-attachments/assets/7c25051f-e169-466d-994e-54719aa2c75b" />

- `Toggle Context Menu` → schimbă instant între context menu **Win11 ↔ Classic** (cu restart Explorer)
- `Disk Cleanup` → lansează curățarea de disc
- `Quick Settings` → shortcut-uri pentru panouri din Settings
- `Network Reset` → resetează complet rețeaua (Winsock + IP)
- `Power Plan Switcher` → schimbă rapid între High Performance, Balanced, Power Saver
- `Power Tweaks` → Disable Sleep (fără standby/sleep/hibernate), Disable HDD/SSD turn off, Disable Monitor turn off și funcționează atât pe baterie (DC) cât și la priză (AC).

### WINDOWS APP REMOVAL
<img width="1002" height="949" alt="Screenshot 2025-08-27 175736" src="https://github.com/user-attachments/assets/e61609e5-1668-4f43-9010-64d144e28b31" />

- `Communication` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Outlook for Windows, Skype, Teams, GroupMe, To-Do, Your Phone, CommsPhone, Messaging, Mail & Calendar
- `Media & Creativity` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Climpchamp, Camera, MS Paint, 3D Builder, 3D Viewer, Print 3D, Mixed Reality Portal
- `Microsoft Apps` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Office Hub, OneNote, Sway, Sticky Notes, Family Safety
- `Bing Apps` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Bing Weather, Bing Sports, Bing Finance, Bing News
- `Games` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: Solitaire Collection, Minecraft for Windows
- `System & Misc` → elimină aplicațiile implicite care vin cu instalarea Windows, precum: People, Maps, Wallet, Get Started, Feedback Hub, Alarms & Clock, OneConnect, Windows Phone, Voice Recorder

### SETTINGS
<img width="1006" height="736" alt="Screenshot 2025-08-26 124340" src="https://github.com/user-attachments/assets/146e63dc-2fb7-4150-91bf-96d9518d3aa9" />

- `Dark / Light Theme` → schimbă între tema Windows

---

## 🎨 Design
- UI vizual cu tab-uri **INFO, TOOLS, SETTINGS**
- Fundal **negru**
- Text și butoane **verde fluorescent (#39FF14)**
- ASCII Logo retro hacker 🟩 la pornire (JetBrains Mono font)

---

## 🛠️ Build și rulare

### 1. Clonează repo
```powershell
git clone https://github.com/eoliann/Eoliann_Windows_Tools
cd Eoliann_Windows_Tools
