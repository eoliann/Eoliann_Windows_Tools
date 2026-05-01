; EWT.iss - Eoliann Windows Tools Installer
; Version: 1.3.5

#define MyAppName "Eoliann Windows Tools"
#define MyAppVersion "1.3.5"
#define MyAppPublisher "eoliann"
#define MyAppExeName "eoliann_windows_tools_w11.exe"
#define MyAppIconName "icon.ico"

[Setup]
AppId={{7E6F0C4D-2C2A-4B16-9B8D-EOLIANN135}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\Eoliann Windows Tools
DefaultGroupName=Eoliann Windows Tools
Compression=lzma
SolidCompression=yes
OutputBaseFilename=Eoliann_Windows_Tools_Installer_v{#MyAppVersion}
DisableProgramGroupPage=false
UninstallDisplayIcon={app}\{#MyAppExeName}
ShowLanguageDialog=false
LicenseFile=..\LICENSE.md
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
RestartIfNeededByRun=no
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"
; Optional. Not recommended if the app requires admin/UAC at startup.
; Name: "startwithwin"; Description: "Run Eoliann Windows Tools at Windows startup"; GroupDescription: "Additional tasks:"; Flags: unchecked

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\eoliann_windows_tools_w11.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "scripts\install_vcredist_if_missing.ps1"; DestDir: "{tmp}"; Flags: deleteafterinstall

[Icons]
Name: "{group}\Eoliann Windows Tools"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; IconFilename: "{app}\{#MyAppIconName}"
Name: "{userdesktop}\Eoliann Windows Tools"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon; IconFilename: "{app}\{#MyAppIconName}"

[Run]
Filename: "powershell.exe"; \
    Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{tmp}\install_vcredist_if_missing.ps1"""; \
    StatusMsg: "Checking Microsoft Visual C++ Runtime..."; \
    Flags: waituntilterminated

Filename: "{app}\{#MyAppExeName}"; \
    Description: "{cm:LaunchProgram,Eoliann Windows Tools}"; \
    Flags: shellexec nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"