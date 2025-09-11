use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use egui::{self, ProgressBar};

// use crate::commands::winget_ensure_ready; // Removed: no such function in commands

/// Un element instalabil via winget.
struct AppItem {
    name: &'static str,
    winget_id: &'static str,
    category: &'static str,
}

/// ✅ Catalog de start (poți adăuga ușor mai multe din lista ta)
static APP_CATALOG: &[AppItem] = &[
    // Browsers
    AppItem { name: "Brave", winget_id: "Brave.Brave", category: "Browsers" },
    AppItem { name: "Chrome", winget_id: "Google.Chrome", category: "Browsers" },
    AppItem { name: "Chromium", winget_id: "Chromium.Chromium", category: "Browsers" },
    AppItem { name: "Firefox", winget_id: "Mozilla.Firefox", category: "Browsers" },
    AppItem { name: "Floorp", winget_id: "Floorp.Floorp", category: "Browsers" },
    AppItem { name: "Zen Browser", winget_id: "SRWare.ZenBrowser", category: "Browsers" },
    AppItem { name: "Waterfox", winget_id: "Waterfox.Waterfox", category: "Browsers" },
    AppItem { name: "Edge", winget_id: "Microsoft.Edge", category: "Browsers" },
    AppItem { name: "Vivaldi", winget_id: "Vivaldi.Vivaldi", category: "Browsers" },
    AppItem { name: "Opera", winget_id: "Opera.Opera", category: "Browsers" },
    AppItem { name: "Opera GX", winget_id: "Opera.OperaGX", category: "Browsers" },
    AppItem { name: "Tor Browser", winget_id: "TheTorProject.TorBrowser", category: "Browsers" },
    AppItem { name: "Yandex", winget_id: "Yandex.YandexBrowser", category: "Browsers" },
    AppItem { name: "Iridium", winget_id: "Iridium.Iridium", category: "Browsers" },
    AppItem { name: "Pale Moon", winget_id: "MoonchildProductions.PaleMoon", category: "Browsers" },
    AppItem { name: "Epic Privacy Browser", winget_id: "Epic.EpicPrivacyBrowser", category: "Browsers" },
    AppItem { name: "Maxthon", winget_id: "Maxthon.Maxthon", category: "Browsers" },
    AppItem { name: "Kometa", winget_id: "Kometa.Kometa", category: "Browsers" },
    AppItem { name: "Torch Browser", winget_id: "Torch.Torch", category: "Browsers" },
    AppItem { name: "SRWare Iron", winget_id: "SRWare.SRWareIron", category: "Browsers" },
    AppItem { name: "Slimjet", winget_id: "FlashPeak.Slimjet", category: "Browsers" },
    AppItem { name: "UC Browser", winget_id: "UCWeb.UCBrowser", category: "Browsers" },
    AppItem { name: "Bing AI", winget_id: "Microsoft.BingAI", category: "Browsers" },
    AppItem { name: "Cent Browser", winget_id: "CentBrowser.CentBrowser", category: "Browsers" },

    // Utilities
    AppItem { name: "1Password", winget_id: "AgileBits.1Password", category: "Utilities" },
    AppItem { name: "7-Zip", winget_id: "7zip.7zip", category: "Utilities" },
    AppItem { name: "NanaZip", winget_id: "M2Team.NanaZip", category: "Utilities" },
    AppItem { name: "Everything Search", winget_id: "voidtools.Everything", category: "Utilities" },
    AppItem { name: "Ditto (Clipboard)", winget_id: "Ditto.Ditto", category: "Utilities" },
    AppItem { name: "PowerToys", winget_id: "Microsoft.PowerToys", category: "Utilities" },
    AppItem { name: "Rufus", winget_id: "Rufus.Rufus", category: "Utilities" },
    AppItem { name: "WinRAR", winget_id: "RARLab.WinRAR", category: "Utilities" },
    AppItem { name: "WinZip", winget_id: "WinZip.WinZip", category: "Utilities" },
    AppItem { name: "f.lux", winget_id: "FluxSoftware.Flux", category: "Utilities" },
    AppItem { name: "ShareX", winget_id: "ShareX.ShareX", category: "Utilities" },
    AppItem { name: "Greenshot", winget_id: "Greenshot.Greenshot", category: "Utilities" },
    AppItem { name: "IrfanView", winget_id: "IrfanSkiljan.IrfanView", category: "Utilities" },
    AppItem { name: "Lightshot", winget_id: "Skillbrains.Lightshot", category: "Utilities" },
    AppItem { name: "Notepad++", winget_id: "Notepad++.Notepad++", category: "Utilities" },
    AppItem { name: "VeraCrypt", winget_id: "IDRIX.VeraCrypt", category: "Utilities" },
    AppItem { name: "AutoHotkey", winget_id: "AutoHotkey.AutoHotkey", category: "Utilities" },
    AppItem { name: "fman", winget_id: "fman.fman", category: "Utilities" },
    AppItem { name: "Q-Dir", winget_id: "SoftwareOK.Q-Dir", category: "Utilities" },
    AppItem { name: "TreeSize Free", winget_id: "JAMSoftware.TreeSizeFree", category: "Utilities" },
    AppItem { name: "WinDirStat", winget_id: "WinDirStat.WinDirStat", category: "Utilities" },
    AppItem { name: "Everything", winget_id: "voidtools.Everything", category: "Utilities" },
    AppItem { name: "ClipClip", winget_id: "ClipClip.ClipClip", category: "Utilities" },
    AppItem { name: "Bulk Rename Utility", winget_id: "TGRM.BulkRenameUtility", category: "Utilities" },
    AppItem { name: "FileZilla", winget_id: "FileZilla.FileZilla", category: "Utilities" },
    AppItem { name: "WinSCP", winget_id: "WinSCP.WinSCP", category: "Utilities" },
    AppItem { name: "PuTTY", winget_id: "SimonTatham.PuTTY", category: "Utilities" },
    AppItem { name: "IObit Uninstaller", winget_id: "IObit.IObitUninstaller", category: "Utilities" },
    AppItem { name: "IObit Driver Booster", winget_id: "IObit.DriverBooster", category: "Utilities" },
    AppItem { name: "IObit Smart Defrag", winget_id: "IObit.SmartDefrag", category: "Utilities" },
    AppItem { name: "IObit Advanced SystemCare", winget_id: "IObit.AdvancedSystemCare", category: "Utilities" },

    // Communications
    AppItem { name: "Discord", winget_id: "Discord.Discord", category: "Communications" },
    AppItem { name: "Telegram", winget_id: "Telegram.TelegramDesktop", category: "Communications" },
    AppItem { name: "Signal", winget_id: "OpenWhisperSystems.Signal", category: "Communications" },
    AppItem { name: "Zoom", winget_id: "Zoom.Zoom", category: "Communications" },
    AppItem { name: "Slack", winget_id: "SlackTechnologies.Slack", category: "Communications" },
    AppItem { name: "Skype", winget_id: "Microsoft.Skype", category: "Communications" },
    AppItem { name: "Microsoft Teams", winget_id: "Microsoft.Teams", category: "Communications" },
    AppItem { name: "WhatsApp", winget_id: "WhatsApp.WhatsApp", category: "Communications" },
    AppItem { name: "Viber", winget_id: "Viber.Viber", category: "Communications" },
    AppItem { name: "Zoom", winget_id: "Zoom.Zoom", category: "Communications" },
    AppItem { name: "Google Meet", winget_id: "Google.GoogleMeet", category: "Communications" },
    AppItem { name: "Microsoft Outlook", winget_id: "Microsoft.Outlook", category: "Communications" },
    AppItem { name: "Thunderbird", winget_id: "Mozilla.Thunderbird", category: "Communications" },
    AppItem { name: "Proton Mail Bridge", winget_id: "ProtonMail.ProtonMailBridge", category: "Communications" },
    AppItem { name: "eM Client", winget_id: "eMClient.eMClient", category: "Communications" },
    AppItem { name: "Mailbird", winget_id: "Mailbird.Mailbird", category: "Communications" },
    AppItem { name: "BlueMail", winget_id: "BlueMail.BlueMail", category: "Communications" },
    AppItem { name: "Hootsuite", winget_id: "Hootsuite.Hootsuite", category: "Communications" },
    AppItem { name: "Tweeten", winget_id: "Tweeten.Tweeten", category: "Communications" },
    AppItem { name: "Franz", winget_id: "MeetFranz.Franz", category: "Communications" },
    AppItem { name: "Rambox", winget_id: "Rambox.Rambox", category: "Communications" },
    AppItem { name: "Pidgin", winget_id: "Pidgin.Pidgin", category: "Communications" },
    AppItem { name: "Jitsi Meet", winget_id: "Jitsi.JitsiMeet", category: "Communications" },
    AppItem { name: "Element", winget_id: "Element.Element", category: "Communications" },
    AppItem { name: "Mumble", winget_id: "Mumble.Mumble", category: "Communications" },
    AppItem { name: "TeamSpeak", winget_id: "TeamSpeak.TeamSpeak", category: "Communications" },
    AppItem { name: "Wire", winget_id: "Wire.Wire", category: "Communications" },
    AppItem { name: "Tox", winget_id: "Tox.Tox", category: "Communications" },
    AppItem { name: "Vocal", winget_id: "Vocal.Vocal", category: "Communications" },
    AppItem { name: "Mastodon", winget_id: "Mastodon.Mastodon", category: "Communications" },
    AppItem { name: "Keybase", winget_id: "Keybase.Keybase", category: "Communications" },
    AppItem { name: "Mattermost", winget_id: "Mattermost.Mattermost", category: "Communications" },

    // Development
    AppItem { name: "Git", winget_id: "Git.Git", category: "Development" },
    AppItem { name: "GitHub Desktop", winget_id: "GitHub.GitHubDesktop", category: "Development" },
    AppItem { name: "NodeJS LTS", winget_id: "OpenJS.NodeJS.LTS", category: "Development" },
    AppItem { name: "Python 3", winget_id: "Python.Python.3.13", category: "Development" },
    AppItem { name: "VS Code", winget_id: "Microsoft.VisualStudioCode", category: "Development" },
    AppItem { name: "Sublime Text", winget_id: "SublimeHQ.SublimeText.4", category: "Development" },
    AppItem { name: "Atom", winget_id: "GitHub.Atom", category: "Development" },
    AppItem { name: "Vim", winget_id: "Vim.Vim", category: "Development" },
    AppItem { name: "Emacs", winget_id: "Emacs.Emacs", category: "Development" },
    AppItem { name: "Postman", winget_id: "Postman.Postman", category: "Development" },
    AppItem { name: "Docker Desktop", winget_id: "Docker.DockerDesktop", category: "Development" },
    AppItem { name: "XAMPP", winget_id: "Bitnami.XAMPP", category: "Development" },
    AppItem { name: "WampServer", winget_id: "WampServer.WampServer", category: "Development" },
    AppItem { name: "MAMP", winget_id: "MAMP.MAMP", category: "Development" },
    AppItem { name: "PHPStorm", winget_id: "JetBrains.PhpStorm", category: "Development" },
    AppItem { name: "Android Studio", winget_id: "JetBrains.AndroidStudio", category: "Development" },
    AppItem { name: "IntelliJ IDEA Community", winget_id: "JetBrains.IntelliJIDEA.Community", category: "Development" },
    AppItem { name: "WebStorm", winget_id: "JetBrains.WebStorm", category: "Development" },
    AppItem { name: "GoLand", winget_id: "JetBrains.GoLand", category: "Development" },
    AppItem { name: "CLion", winget_id: "JetBrains.CLion", category: "Development" },
    AppItem { name: "Rider", winget_id: "JetBrains.Rider", category: "Development" },
    AppItem { name: "DataGrip", winget_id: "JetBrains.DataGrip", category: "Development" },
    AppItem { name: "PyCharm", winget_id: "JetBrains.PyCharm", category: "Development" },
    AppItem { name: "RubyMine", winget_id: "JetBrains.RubyMine", category: "Development" },
    AppItem { name: "RStudio", winget_id: "JetBrains.RStudio", category: "Development" },
    AppItem { name: "Eclipse IDE", winget_id: "EclipseFoundation.EclipseIDE", category: "Development" },
    AppItem { name: "NetBeans", winget_id: "Apache.NetBeans", category: "Development" },
    AppItem { name: "BlueJ", winget_id: "BlueJ.BlueJ", category: "Development" },
    AppItem { name: "Code::Blocks", winget_id: "CodeBlocks.CodeBlocks", category: "Development" },
    AppItem { name: "Dev-C++", winget_id: "Embarcadero.Dev-Cpp", category: "Development" },
    AppItem { name: "Arduino IDE", winget_id: "Arduino.ArduinoIDE", category: "Development" },
    AppItem { name: "Processing", winget_id: "ProcessingFoundation.Processing", category: "Development" },
    AppItem { name: "Godot Engine", winget_id: "Godot.Godot", category: "Development" },
    AppItem { name: "Unity", winget_id: "Unity.Unity", category: "Development" },
    AppItem { name: "Unreal Engine", winget_id: "Epic.UnrealEngine", category: "Development" },
    AppItem { name: "CMake", winget_id: "Kitware.CMake", category: "Development" },
    AppItem { name: "MinGW", winget_id: "MinGW.MinGW", category: "Development" },
    AppItem { name: "GCC", winget_id: "GCC.GCC", category: "Development" },
    AppItem { name: "LLVM", winget_id: "LLVM.LLVM", category: "Development" },
    AppItem { name: "Cygwin", winget_id: "Cygwin.Cygwin", category: "Development" },

    // Document
    AppItem { name: "Adobe Acrobat Reader", winget_id: "Adobe.Acrobat.Reader.64-bit", category: "Document" },
    AppItem { name: "Sumatra PDF", winget_id: "SumatraPDF.SumatraPDF", category: "Document" },
    AppItem { name: "LibreOffice", winget_id: "TheDocumentFoundation.LibreOffice", category: "Document" },
    AppItem { name: "Notepad++", winget_id: "Notepad++.Notepad++", category: "Document" },
    AppItem { name: "Foxit Reader", winget_id: "Foxit.FoxitReader", category: "Document" },
    AppItem { name: "PDF-XChange Editor", winget_id: "TrackerSoftware.PDFXChangeEditor", category: "Document" },
    AppItem { name: "Microsoft Word", winget_id: "Microsoft.Word", category: "Document" },
    AppItem { name: "Microsoft Excel", winget_id: "Microsoft.Excel", category: "Document" },
    AppItem { name: "Microsoft PowerPoint", winget_id: "Microsoft.PowerPoint", category: "Document" },
    AppItem { name: "WPS Office", winget_id: "Kingsoft.WPSOffice", category: "Document" },
    AppItem { name: "OnlyOffice", winget_id: "AscensioSystemSIA.ONLYOFFICE", category: "Document" },
    AppItem { name: "PDFsam Basic", winget_id: "PDFsam.PDFsamBasic", category: "Document" },
    AppItem { name: "Calibre", winget_id: "KovidGoyal.Calibre", category: "Document" },
    AppItem { name: "Zotero", winget_id: "Zotero.Zotero", category: "Document" },
    AppItem { name: "Mendeley", winget_id: "Elsevier.Mendeley", category: "Document" },
    AppItem { name: "EndNote", winget_id: "Clarivate.EndNote", category: "Document" },
    AppItem { name: "Scrivener", winget_id: "LiteratureAndLatte.Scrivener", category: "Document" },
    AppItem { name: "Bibisco", winget_id: "bibisco.bibisco", category: "Document" },
    AppItem { name: "FocusWriter", winget_id: "GottCode.FocusWriter", category: "Document" },
    AppItem { name: "Typora", winget_id: "Typora.Typora", category: "Document" },
    AppItem { name: "Obsidian", winget_id: "Obsidian.Obsidian", category: "Document" },
    AppItem { name: "Joplin", winget_id: "Joplin.Joplin", category: "Document" },
    AppItem { name: "Zettlr", winget_id: "Zettlr.Zettlr", category: "Document" },
    AppItem { name: "Atom", winget_id: "GitHub.Atom", category: "Document" },
    AppItem { name: "Typora", winget_id: "Typora.Typora", category: "Document" },
    AppItem { name: "MarkText", winget_id: "MarkText.MarkText", category: "Document" },
    AppItem { name: "Hemingway Editor", winget_id: "HemingwayEditor.HemingwayEditor", category: "Document" },
    AppItem { name: "Grammarly", winget_id: "Grammarly.Grammarly", category: "Document" },
    AppItem { name: "ProWritingAid", winget_id: "ProWritingAid.ProWritingAid", category: "Document" },
    AppItem { name: "Ginger", winget_id: "GingerSoftware.Ginger", category: "Document" },
    AppItem { name: "WhiteSmoke", winget_id: "WhiteSmoke.WhiteSmoke", category: "Document" },
    AppItem { name: "Scribus", winget_id: "Scribus.Scribus", category: "Document" },
    AppItem { name: "LaTeX", winget_id: "MiKTeX.MiKTeX", category: "Document" },
    AppItem { name: "TeXstudio", winget_id: "TeXstudio.TeXstudio", category: "Document" },

    // Multimedia
    AppItem { name: "VLC", winget_id: "VideoLAN.VLC", category: "Multimedia Tools" },
    AppItem { name: "OBS Studio", winget_id: "OBSProject.OBSStudio", category: "Multimedia Tools" },
    AppItem { name: "ShareX", winget_id: "ShareX.ShareX", category: "Multimedia Tools" },
    AppItem { name: "GIMP", winget_id: "GIMP.GIMP.3", category: "Multimedia Tools" },
    AppItem { name: "Krita", winget_id: "KDE.Krita", category: "Multimedia Tools" },
    AppItem { name: "Inkscape", winget_id: "Inkscape.Inkscape", category: "Multimedia Tools" },
    AppItem { name: "Blender", winget_id: "BlenderFoundation.Blender", category: "Multimedia Tools" },
    AppItem { name: "Audacity", winget_id: "Audacity.Audacity", category: "Multimedia Tools" },
    AppItem { name: "Shotcut", winget_id: "Meltytech.Shotcut", category: "Multimedia Tools" },
    AppItem { name: "HandBrake", winget_id: "HandBrake.HandBrake", category: "Multimedia Tools" },
    AppItem { name: "DaVinci Resolve", winget_id: "BlackmagicDesign.DaVinciResolve", category: "Multimedia Tools" },
    AppItem { name: "Lightworks", winget_id: "EditShare.Lightworks", category: "Multimedia Tools" },
    AppItem { name: "VSDC Free Video Editor", winget_id: "FlashIntegro.VSDCFreeVideoEditor", category: "Multimedia Tools" },
    AppItem { name: "Avidemux", winget_id: "Mean.Avidemux", category: "Multimedia Tools" },
    AppItem { name: "Media Player Classic - Home Cinema", winget_id: "MPC-HC.MPC-HC", category: "Multimedia Tools" },
    AppItem { name: "Foobar2000", winget_id: "foobar2000.foobar2000", category: "Multimedia Tools" },
    AppItem { name: "Spotify", winget_id: "Spotify.Spotify", category: "Multimedia Tools" },
    AppItem { name: "iTunes", winget_id: "Apple.iTunes", category: "Multimedia Tools" },
    AppItem { name: "Vox", winget_id: "Coppertino.Vox", category: "Multimedia Tools" },
    AppItem { name: "Adobe Photoshop", winget_id: "Adobe.Photoshop", category: "Multimedia Tools" },
    AppItem { name: "Adobe Lightroom", winget_id: "Adobe.Lightroom", category: "Multimedia Tools" },
    AppItem { name: "Adobe Premiere Pro", winget_id: "Adobe.PremierePro", category: "Multimedia Tools" },
    AppItem { name: "Adobe After Effects", winget_id: "Adobe.AfterEffects", category: "Multimedia Tools" },
    AppItem { name: "Adobe InDesign", winget_id: "Adobe.Indesign", category: "Multimedia Tools" },
    AppItem { name: "Adobe Illustrator", winget_id: "Adobe.Illustrator", category: "Multimedia Tools" },
    AppItem { name: "CorelDRAW", winget_id: "Corel.CorelDRAW", category: "Multimedia Tools" },
    AppItem { name: "Inkscape", winget_id: "Inkscape.Inkscape", category: "Multimedia Tools" },
    AppItem { name: "Affinity Designer", winget_id: "Serif.AffinityDesigner", category: "Multimedia Tools" },
    AppItem { name: "Affinity Photo", winget_id: "Serif.AffinityPhoto", category: "Multimedia Tools" },
    AppItem { name: "Affinity Publisher", winget_id: "Serif.AffinityPublisher", category: "Multimedia Tools" },
    AppItem { name: "Affinity Video", winget_id: "Serif.AffinityVideo", category: "Multimedia Tools" },
    AppItem { name: "Canva", winget_id: "Canva.Canva", category: "Multimedia Tools" },
    AppItem { name: "Corel PaintShop Pro", winget_id: "Corel.PaintShopPro", category: "Multimedia Tools" },
    AppItem { name: "Corel VideoStudio", winget_id: "Corel.VideoStudio", category: "Multimedia Tools" },
    AppItem { name: "CyberLink PowerDirector", winget_id: "CyberLink.PowerDirector", category: "Multimedia Tools" },
    AppItem { name: "Magix Movie Edit Pro", winget_id: "MAGIX.MovieEditPro", category: "Multimedia Tools" },
    AppItem { name: "MAGIX Music Maker", winget_id: "MAGIX.MusicMaker", category: "Multimedia Tools" },
    AppItem { name: "FL Studio", winget_id: "ImageLine.FLStudio", category: "Multimedia Tools" },
    AppItem { name: "Ableton Live", winget_id: "Ableton.AbletonLive", category: "Multimedia Tools" },
    AppItem { name: "Pro Tools", winget_id: "Avid.ProTools", category: "Multimedia Tools" },
    AppItem { name: "Cubase", winget_id: "Steinberg.Cubase", category: "Multimedia Tools" },
    AppItem { name: "Reaper", winget_id: "Cockos.Reaper", category: "Multimedia Tools" },
    AppItem { name: "VirtualDJ", winget_id: "Atomix.VirtualDJ", category: "Multimedia Tools" },
    AppItem { name: "Streamlabs OBS", winget_id: "Streamlabs.StreamlabsOBS", category: "Multimedia Tools" },
    AppItem { name: "XSplit Broadcaster", winget_id: "SplitmediaLabs.XSplitBroadcaster", category: "Multimedia Tools" },
    AppItem { name: "XSplit Gamecaster", winget_id: "SplitmediaLabs.XSplitGamecaster", category: "Multimedia Tools" },

    // Games/platforms
    AppItem { name: "Steam", winget_id: "Valve.Steam", category: "Games" },
    AppItem { name: "Epic Games Launcher", winget_id: "EpicGames.EpicGamesLauncher", category: "Games" },
    AppItem { name: "GOG Galaxy", winget_id: "GOG.Galaxy", category: "Games" },
    AppItem { name: "Origin", winget_id: "ElectronicArts.Origin", category: "Games" },
    AppItem { name: "Ubisoft Connect", winget_id: "Ubisoft.UbisoftConnect", category: "Games" },
    AppItem { name: "Battle.net", winget_id: "Blizzard.Battle.net", category: "Games" },
    AppItem { name: "Minecraft", winget_id: "Microsoft.Minecraft", category: "Games" },
    AppItem { name: "Roblox", winget_id: "Roblox.Roblox", category: "Games" },
    AppItem { name: "League of Legends", winget_id: "RiotGames.LeagueofLegends", category: "Games" },
    AppItem { name: "Valorant", winget_id: "RiotGames.Valorant", category: "Games" },
    AppItem { name: "Fortnite", winget_id: "EpicGames.Fortnite", category: "Games" },
    AppItem { name: "Call of Duty", winget_id: "Activision.CallofDuty", category: "Games" },
    AppItem { name: "Among Us", winget_id: "Innersloth.AmongUs", category: "Games" },
    AppItem { name: "Rocket League", winget_id: "Psyonix.RocketLeague", category: "Games" },
    AppItem { name: "Counter-Strike: Global Offensive", winget_id: "Valve.CounterStrikeGlobalOffensive", category: "Games" },
    AppItem { name: "Dota 2", winget_id: "Valve.Dota2", category: "Games" },
    AppItem { name: "Team Fortress 2", winget_id: "Valve.TeamFortress2", category: "Games" },
    AppItem { name: "The Witcher 3: Wild Hunt", winget_id: "CDProjektRed.TheWitcher3WildHunt", category: "Games" },
    AppItem { name: "Cyberpunk 2077", winget_id: "CDProjektRed.Cyberpunk2077", category: "Games" },
    AppItem { name: "Red Dead Redemption 2", winget_id: "RockstarGames.RedDeadRedemption2", category: "Games" },
    AppItem { name: "Grand Theft Auto V", winget_id: "RockstarGames.GrandTheftAutoV", category: "Games" },
    AppItem { name: "Assassin's Creed Valhalla", winget_id: "Ubisoft.Assassin'sCreedValhalla", category: "Games" },

    // Microsoft tools
    AppItem { name: "Windows Terminal", winget_id: "Microsoft.WindowsTerminal", category: "Microsoft Tools" },
    AppItem { name: "PowerShell 7", winget_id: "Microsoft.PowerShell", category: "Microsoft Tools" },
    AppItem { name: "Visual Studio 2022 Community", winget_id: "Microsoft.VisualStudio.2022.Community", category: "Microsoft Tools" },
    AppItem { name: "Visual Studio Code", winget_id: "Microsoft.VisualStudioCode", category: "Microsoft Tools" },
    AppItem { name: "Microsoft To Do", winget_id: "Microsoft.Todo", category: "Microsoft Tools" },
    AppItem { name: "Microsoft OneDrive", winget_id: "Microsoft.OneDrive", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Edge", winget_id: "Microsoft.Edge", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Office", winget_id: "Microsoft.Office", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Outlook", winget_id: "Microsoft.Outlook", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Word", winget_id: "Microsoft.Word", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Excel", winget_id: "Microsoft.Excel", category: "Microsoft Tools" },
    AppItem { name: "Microsoft PowerPoint", winget_id: "Microsoft.PowerPoint", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Teams", winget_id: "Microsoft.Teams", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Publisher", winget_id: "Microsoft.Publisher", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visio", winget_id: "Microsoft.Visio", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Project", winget_id: "Microsoft.Project", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2015-2022", winget_id: "Microsoft.VCRedist.2015+.x64", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2015-2022 (x86)", winget_id: "Microsoft.VCRedist.2015+.x86", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2013", winget_id: "Microsoft.VCRedist.2013.x64", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2013 (x86)", winget_id: "Microsoft.VCRedist.2013.x86", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2012", winget_id: "Microsoft.VCRedist.2012.x64", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2012 (x86)", winget_id: "Microsoft.VCRedist.2012.x86", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2010", winget_id: "Microsoft.VCRedist.2010.x64", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2010 (x86)", winget_id: "Microsoft.VCRedist.2010.x86", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2008", winget_id: "Microsoft.VCRedist.2008.x64", category: "Microsoft Tools" },
    AppItem { name: "Microsoft Visual C++ Redistributable 2008 (x86)", winget_id: "Microsoft.VCRedist.2008.x86", category: "Microsoft Tools" },

    // Security
    AppItem { name: "Malwarebytes", winget_id: "Malwarebytes.Malwarebytes", category: "Security" },
    AppItem { name: "Bitwarden", winget_id: "Bitwarden.Bitwarden", category: "Security" },
    AppItem { name: "LastPass", winget_id: "LastPass.LastPass", category: "Security" },
    AppItem { name: "Avast Free Antivirus", winget_id: "AvastSoftware.AvastFreeAntivirus", category: "Security" },
    AppItem { name: "Kaspersky Security Cloud", winget_id: "Kaspersky.KasperskySecurityCloud", category: "Security" },
    AppItem { name: "Norton 360", winget_id: "Norton.Norton360", category: "Security" },
    AppItem { name: "McAfee Total Protection", winget_id: "McAfee.TotalProtection", category: "Security" },
    AppItem { name: "ESET NOD32 Antivirus", winget_id: "ESET.ESETNOD32Antivirus", category: "Security" },
    AppItem { name: "Avira Free Security", winget_id: "Avira.AviraFreeSecurity", category: "Security" },
    AppItem { name: "Sophos Home", winget_id: "Sophos.SophosHome", category: "Security" },
    AppItem { name: "ZoneAlarm Free Firewall", winget_id: "CheckPoint.ZoneAlarmFreeFirewall", category: "Security" },
    AppItem { name: "GlassWire", winget_id: "GlassWire.GlassWire", category: "Security" },
    AppItem { name: "Spybot - Search & Destroy", winget_id: "Safer-Networking.Spybot", category: "Security" },
    AppItem { name: "AdwCleaner", winget_id: "Malwarebytes.AdwCleaner", category: "Security" }, // by Malwarebytes
    AppItem { name: "CCleaner", winget_id: "Piriform.CCleaner", category: "Security" },
    AppItem { name: "ZoneAlarm Free Antivirus + Firewall", winget_id: "CheckPoint.ZoneAlarmFreeAntivirusFirewall", category: "Security" },
    AppItem { name: "Immunet", winget_id: "Cisco.Immunet", category: "Security" },
    AppItem { name: "Comodo Free Firewall", winget_id: "Comodo.ComodoFreeFirewall", category: "Security" },
    AppItem { name: "F-Secure SAFE", winget_id: "F-Secure.FSecureSAFE", category: "Security" },
    AppItem { name: "Panda Free Antivirus", winget_id: "PandaSecurity.PandaFreeAntivirus", category: "Security" },
    AppItem { name: "Trend Micro Maximum Security", winget_id: "TrendMicro.TrendMicroMaximumSecurity", category: "Security" },
    AppItem { name: "Webroot SecureAnywhere", winget_id: "Webroot.WebrootSecureAnywhere", category: "Security" },
    AppItem { name: "Bitdefender Antivirus Free", winget_id: "Bitdefender.BitdefenderAntivirusFree", category: "Security" },

    // Pro Tools
    AppItem { name: "Advanced IP Scanner", winget_id: "Famatech.AdvancedIPScanner", category: "Pro Tools" },
    AppItem { name: "Wireshark", winget_id: "WiresharkFoundation.Wireshark", category: "Pro Tools" },
    AppItem { name: "Nmap", winget_id: "Insecure.Nmap", category: "Pro Tools" },
    AppItem { name: "PuTTY", winget_id: "SimonTatham.PuTTY", category: "Pro Tools" },
    AppItem { name: "FileZilla", winget_id: "FileZilla.FileZilla", category: "Pro Tools" },
    AppItem { name: "WinSCP", winget_id: "WinSCP.WinSCP", category: "Pro Tools" },
    AppItem { name: "VLC Media Player", winget_id: "VideoLAN.VLC", category: "Pro Tools" },
    AppItem { name: "IrfanView", winget_id: "IrfanSkiljan.IrfanView", category: "Pro Tools" },
    AppItem { name: "Greenshot", winget_id: "Greenshot.Greenshot", category: "Pro Tools" },
    AppItem { name: "CCleaner", winget_id: "Piriform.CCleaner", category: "Pro Tools" },
    AppItem { name: "HWMonitor", winget_id: "CPUID.HWMonitor", category: "Pro Tools" },
    AppItem { name: "CPU-Z", winget_id: "CPUID.CPU-Z", category: "Pro Tools" },
    AppItem { name: "CrystalDiskInfo", winget_id: "CrystalDeworld.CrystalDiskInfo", category: "Pro Tools" },
    AppItem { name: "Speccy", winget_id: "Piriform.Speccy", category: "Pro Tools" },
    AppItem { name: "Defraggler", winget_id: "Piriform.Defraggler", category: "Pro Tools" },
    AppItem { name: "IObit Uninstaller", winget_id: "IObit.IObitUninstaller", category: "Pro Tools" },
    AppItem { name: "IObit Driver Booster", winget_id: "IObit.DriverBooster", category: "Pro Tools" },
    AppItem { name: "IObit Smart Defrag", winget_id: "IObit.SmartDefrag", category: "Pro Tools" },
    AppItem { name: "IObit Advanced SystemCare", winget_id: "IObit.AdvancedSystemCare", category: "Pro Tools" },
    AppItem { name: "VeraCrypt", winget_id: "IDRIX.VeraCrypt", category: "Pro Tools" },
    AppItem { name: "Audacity", winget_id: "Audacity.Audacity", category: "Pro Tools" },
    AppItem { name: "Krita", winget_id: "Krita.Krita", category: "Pro Tools" },
    AppItem { name: "Inkscape", winget_id: "Inkscape.Inkscape", category: "Pro Tools" },
    AppItem { name: "Blender", winget_id: "BlenderFoundation.Blender", category: "Pro Tools" },
    AppItem { name: "Shotcut", winget_id: "Meltytech.Shotcut", category: "Pro Tools" },
    AppItem { name: "HandBrake", winget_id: "HandBrake.HandBrake", category: "Pro Tools" },
    AppItem { name: "OBS Studio", winget_id: "OBSProject.OBSStudio", category: "Pro Tools" },
    AppItem { name: "Lightworks", winget_id: "EditShare.Lightworks", category: "Pro Tools" },
    AppItem { name: "DaVinci Resolve", winget_id: "BlackmagicDesign.DaVinciResolve", category: "Pro Tools" },
    AppItem { name: "Acronis True Image", winget_id: "Acronis.AcronisTrueImage", category: "Pro Tools" },
    AppItem { name: "AOMEI Backupper Standard", winget_id: "AOMEI.AOMEIBackupperStandard", category: "Pro Tools" },
    AppItem { name: "Macrium Reflect Free", winget_id: "Macrium.ReflectFree", category: "Pro Tools" },
];

#[derive(Debug, Clone, Copy)]
enum DoWhat { Install, Uninstall, Update }

#[derive(Default, Clone)]
struct Progress {
    running: bool,
    current: usize,
    total: usize,
    current_name: String,
}

struct InstallState {
    selected: HashSet<String>,            // set de winget IDs selectate
    progress: Arc<Mutex<Progress>>,       // progresul curent pentru bară
}

impl Default for InstallState {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            progress: Arc::new(Mutex::new(Progress::default())),
        }
    }
}

static INSTALL_STATE: OnceLock<Mutex<InstallState>> = OnceLock::new();
fn state() -> &'static Mutex<InstallState> {
    INSTALL_STATE.get_or_init(|| Mutex::new(InstallState::default()))
}

fn log_line(log: &Arc<Mutex<String>>, s: impl AsRef<str>) {
    if let Ok(mut lg) = log.lock() {
        lg.push_str(s.as_ref());
        if !s.as_ref().ends_with('\n') { lg.push('\n'); }
    }
}

pub fn show_install(ui: &mut egui::Ui, log: &Arc<Mutex<String>>) {
    // Snapshot info
    let (sel_count, running, p_snapshot) = {
        let st = state().lock().unwrap();
        let p = st.progress.lock().unwrap().clone();
        (st.selected.len(), p.running, p)
    };

    // Header mic cu progres
    ui.group(|ui| {
        if running {
            ui.label(format!("Working on: {}", p_snapshot.current_name));
            let frac = if p_snapshot.total > 0 {
                p_snapshot.current as f32 / p_snapshot.total as f32
            } else { 0.0 };
            ui.add(ProgressBar::new(frac).text(format!("{} / {}", p_snapshot.current, p_snapshot.total)));
        } else {
            ui.label(if sel_count > 0 {
                format!("Selected: {} apps", sel_count)
            } else {
                "Select apps then choose an action.".to_owned()
            });
        }
    });

    ui.add_space(6.0);

    // Acțiuni
    ui.horizontal(|ui| {
        let disabled = sel_count == 0 || running;
        if ui.add_enabled(!disabled, egui::Button::new("Install selections")).clicked() {
            spawn_task(DoWhat::Install, log.clone());
        }
        if ui.add_enabled(!disabled, egui::Button::new("Uninstall selections")).clicked() {
            spawn_task(DoWhat::Uninstall, log.clone());
        }
        if ui.add_enabled(!disabled, egui::Button::new("Update selections")).clicked() {
            spawn_task(DoWhat::Update, log.clone());
        }
        if ui.button("Clear selection").clicked() {
            state().lock().unwrap().selected.clear();
        }
    });

    ui.add_space(6.0);

    // Listă pe categorii cu scroll
    let mut by_cat: BTreeMap<&'static str, Vec<&'static AppItem>> = BTreeMap::new();
    for item in APP_CATALOG.iter() {
        by_cat.entry(item.category).or_default().push(item);
    }

    egui::ScrollArea::vertical()
        .id_salt("install_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (category, items) in by_cat.into_iter() {
                ui.collapsing(category, |ui| {
                    for app in items {
                        let mut checked = state().lock().unwrap().selected.contains(app.winget_id);
                        if ui.checkbox(&mut checked, app.name).clicked() {
                            let mut st = state().lock().unwrap();
                            if checked {
                                st.selected.insert(app.winget_id.to_string());
                            } else {
                                st.selected.remove(app.winget_id);
                            }
                        }
                    }
                });
            }
        });
}

fn spawn_task(what: DoWhat, log: Arc<Mutex<String>>) {
    // colectăm selecția + pregătim progresul
    let (list, progress) = {
        let st = state().lock().unwrap();
        let v: Vec<(String, String)> = APP_CATALOG
            .iter()
            .filter(|a| st.selected.contains(a.winget_id))
            .map(|a| (a.name.to_string(), a.winget_id.to_string()))
            .collect();
        (v, st.progress.clone())
    };

    if list.is_empty() {
        return;
    }

    // winget ready?
    if !crate::commands::ensure_winget_ready(log.clone()) {
        return;
    }

    // rulează în fundal
    std::thread::spawn(move || {
        {
            let mut p = progress.lock().unwrap();
            p.running = true;
            p.total = list.len();
            p.current = 0;
            p.current_name.clear();
        }

        for (idx, (name, id)) in list.iter().enumerate() {
            {
                let mut p = progress.lock().unwrap();
                p.current = idx;
                p.current_name = name.clone();
            }

            match what {
                DoWhat::Install => {
                    if crate::commands::winget_is_installed(id) {
                        log_line(&log, format!("ℹ {} already installed. Skipping.", name));
                    } else {
                        log_line(&log, format!("⬇ Installing {}...", name)); //
                        let _ = crate::commands::winget_install(id, log.clone());
                    }
                }
                DoWhat::Uninstall => {
                    if crate::commands::winget_is_installed(id) {
                        log_line(&log, format!("🗑 Uninstalling {}...", name));
                        let _ = crate::commands::winget_uninstall(id, log.clone());
                    } else {
                        log_line(&log, format!("ℹ {} not found. Skipping.", name));
                    }
                }
                DoWhat::Update => {
                    if crate::commands::winget_is_installed(id) {
                        log_line(&log, format!("⤴ Updating {}...", name));
                        let _ = crate::commands::winget_upgrade(id, log.clone());
                    } else {
                        log_line(&log, format!("ℹ {} not installed. Skipping update.", name));
                    }
                }
            }
        }

        {
            let mut p = progress.lock().unwrap();
            p.current = p.total;
            p.current_name = "Done".to_string();
            p.running = false;
        }

        log_line(&log, "✅ Finished processing selection.");

        // Debifează tot după terminare
        state().lock().unwrap().selected.clear();
    });
}
