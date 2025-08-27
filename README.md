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
<img width="1006" height="736" alt="Screenshot 2025-08-26 124331" src="https://github.com/user-attachments/assets/703be1cc-5cbc-4890-8f23-f56bc6286b50" />

- `whoami` → afișează utilizatorul curent
- `ipconfig` → afișează setările de rețea
- `systeminfo` → detalii despre sistem
- `tasklist` → procese active
- `About` → versiune + link GitHub

### TOOLS
<img width="1006" height="736" alt="Screenshot 2025-08-26 124337" src="https://github.com/user-attachments/assets/b70d68cc-1412-4283-b270-b13530bb1e55" />

- `Toggle Context Menu` → schimbă instant între context menu **Win11 ↔ Classic** (cu restart Explorer)
- `Disk Cleanup` → lansează curățarea de disc
- `Quick Settings` → shortcut-uri pentru panouri din Settings
- `Network Reset` → resetează complet rețeaua (Winsock + IP)
- `Power Plan Switcher` → schimbă rapid între High Performance, Balanced, Power Saver

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
