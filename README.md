# 🟢 Eoliann Windows Tools

Un toolkit rapid pentru Windows 11, scris în Rust + egui, cu UI verde fluorescent retro hacker 🟩.  
Include comenzi utile de administrare și tweak-uri pentru Windows 11.

---

## 🚀 Funcționalități

### INFO
- `whoami` → afișează utilizatorul curent
- `ipconfig` → afișează setările de rețea
- `systeminfo` → detalii despre sistem
- `tasklist` → procese active
- `About` → versiune + link GitHub

### TOOLS
- `Toggle Context Menu` → schimbă instant între context menu **Win11 ↔ Classic** (cu restart Explorer)
- `Disk Cleanup` → lansează curățarea de disc
- `Quick Settings` → shortcut-uri pentru panouri din Settings
- `Network Reset` → resetează complet rețeaua (Winsock + IP)
- `Power Plan Switcher` → schimbă rapid între High Performance, Balanced, Power Saver

### SETTINGS
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
git clone https://github.com/eoliann/Eoliann_Windows_Tools_W11
cd Eoliann_Windows_Tools_W11
