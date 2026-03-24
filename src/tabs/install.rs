// src/tabs/install.rs
// Rewritten install tab (full file). Focus: remove unwanted vertical gaps in the Chrome panel
// and provide a compact, predictable layout for the right column (Chrome panel).
//
// Notes:
// - Expects `crate::commands` to export winget helpers used below
//   (ensure_winget_ready, winget_is_installed, winget_install, winget_uninstall, winget_upgrade, upgrade_all_apps_with_log).
// - Uses standard egui layout techniques: temporarily reduces `item_spacing` around
//   the button + separator and ensures spacing is restored after the compact blocks.

use std::collections::{BTreeMap, HashSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};
use std::path::PathBuf;
use std::process::Command;
// use crate::app::App;
// use crate::app::App;
use crate::commands;
use eframe::egui::{self, Align2};

/// Un element instalabil via winget.
struct AppItem {
    name: &'static str,
    winget_id: &'static str,
    category: &'static str,
    desc: &'static str,
}

/// ORIGINAL COMPLETE APP_CATALOG (restored)
static APP_CATALOG: &[AppItem] = &[
    // Browsers
    AppItem { name: "Brave", winget_id: "Brave.Brave", category: "Browsers", desc: "Brave is a privacy-focused web browser that blocks ads and trackers, offering a faster and safer browsing experience." },
    AppItem { name: "Chrome", winget_id: "Google.Chrome", category: "Browsers", desc: "Google Chrome is a widely used web browser known for its speed, simplicity, and seamless integration with Google services." },
    AppItem { name: "Chromium", winget_id: "Hibbiki.Chromium", category: "Browsers", desc: "Chromium is the open-source project that serves as the foundation for various web browsers, including Chrome." },
    AppItem { name: "Edge", winget_id: "Microsoft.Edge", category: "Browsers", desc: "Microsoft Edge is a modern web browser built on Chromium, offering performance, security, and integration with Microsoft services." },
    AppItem { name: "Falkon", winget_id: "KDE.Falkon", category: "Browsers", desc: "Falkon is a lightweight and fast web browser with a focus on user privacy and efficiency." },
    AppItem { name: "Firefox", winget_id: "Mozilla.Firefox", category: "Browsers", desc: "Mozilla Firefox is an open-source web browser known for its customization options, privacy features, and extensions." },
    AppItem { name: "Firefox ESR", winget_id: "Mozilla.Firefox.ESR", category: "Browsers", desc: "Mozilla Firefox is an open-source web browser known for its customization options, privacy features, and extensions. Firefox ESR (Extended Support Release) receives major updates every 42 weeks with minor updates such as crash fixes, security fixes and policy updates as needed, but at least every four weeks." },
    AppItem { name: "Floorp", winget_id: "Ablaze.Floorp", category: "Browsers", desc: "Floorp is an open-source web browser project that aims to provide a simple and fast browsing experience." },
    AppItem { name: "LibreWolf", winget_id: "LibreWolf.LibreWolf", category: "Browsers", desc: "LibreWolf is a privacy-focused web browser based on Firefox, with additional privacy and security enhancements." },
    AppItem { name: "Mullvad Browser", winget_id: "MullvadVPN.MullvadBrowser", category: "Browsers", desc: "Mullvad Browser is a privacy-focused web browser, developed in partnership with the Tor Project." },
    AppItem { name: "Opera", winget_id: "Opera.Opera", category: "Browsers", desc: "Opera is a feature-rich web browser known for its built-in ad blocker, VPN, and various productivity tools." },
    AppItem { name: "Opera GX", winget_id: "Opera.OperaGX", category: "Browsers", desc: "Opera GX is a gaming-focused web browser that offers features like CPU, RAM, and network limiters to optimize gaming performance while browsing." },
    AppItem { name: "PaleMoon", winget_id: "MoonchildProductions.PaleMoon", category: "Browsers", desc: "PaleMoon is an open-source web browser based on Firefox, focusing on customization and user control. PaleMoon is an Open Source, Goanna-based web browser available for Microsoft Windows and Linux (with other operating systems in development), focusing on efficiency and ease of use." },
    AppItem { name: "Thorium Browser AVX2", winget_id: "Alex313031.Thorium.AVX2", category: "Browsers", desc: "Thorium Browser is a privacy-focused web browser based on Chromium, designed to provide a secure and efficient browsing experience. Built for speed over vanilla chromium. It is built with AVX2 optimizations and is the fastest browser on the market." },
    AppItem { name: "Tor Browser", winget_id: "TorProject.TorBrowser", category: "Browsers", desc: "Tor Browser is designed for anonymous web browsing, utilizing the Tor network to protect user privacy and security." },
    AppItem { name: "UnGoogled Chromium", winget_id: "eloston.ungoogled-chromium", category: "Browsers", desc: "Ungoogled Chromium is a version of Chromium without Google's integration for enhanced privacy and control." },
    AppItem { name: "Vivaldi", winget_id: "Vivaldi.Vivaldi", category: "Browsers", desc: "Vivaldi is a highly customizable web browser with a focus on user personalization and productivity features." },
    AppItem { name: "Waterfox", winget_id: "Waterfox.Waterfox", category: "Browsers", desc: "Waterfox is a privacy-focused web browser based on Firefox, designed to provide a fast and secure browsing experience." },
    AppItem { name: "Zen Browser", winget_id: "Zen-Team.Zen-Browser", category: "Browsers", desc: "Zen Browser is a modern privacy-focused, performance-driven web browser built on Firefox, designed to provide a secure and efficient browsing experience." },

    // Utilities
    AppItem { name: "1Password", winget_id: "AgileBits.1Password", category: "Utilities", desc: "1Password is a password manager that allows you to store and manage your passwords securely." },
    AppItem { name: "7-Zip", winget_id: "7zip.7zip", category: "Utilities", desc: "7-Zip is a free and open-source file archiver utility. It supports several compression formats and provides a high compression ratio, making it a popular choice for file compression." },
    AppItem { name: "Advanced Renamer", winget_id: "HulubuluSoftware.AdvancedRenamer", category: "Utilities", desc: "Advanced Renamer is a program for renaming multiple files and folders at once. By configuring renaming methods the names can be manipulated in various ways." },
    AppItem { name: "AIDA64", winget_id: "FinalWire.AIDA64.Business", category: "Utilities", desc: "AIDA64 is a system information, diagnostics, and benchmarking solution for Windows PCs. It provides detailed information about hardware and software components, as well as tools for monitoring system performance." },
    AppItem { name: "AOMEI Backupper", winget_id: "AOMEI.Backupper.Standard", category: "Utilities", desc: "AOMEI Backupper is a backup and recovery software that allows you to create backups of your system, files, and disks, as well as restore them when needed." },
    AppItem { name: "AOMEI Partition Assistant", winget_id: "AOMEI.PartitionAssistant", category: "Utilities", desc: "AOMEI Partition Assistant is a disk partition management software that allows you to create, resize, move, merge, and split partitions on your hard drive." },
    AppItem { name: "AnyDesk", winget_id: "AnyDesk.AnyDesk", category: "Utilities", desc: "AnyDesk is a remote desktop software that enables users to access and control computers remotely. It is known for its fast connection and low latency." },
    AppItem { name: "AutoHotkey", winget_id: "AutoHotkey.AutoHotkey", category: "Utilities", desc: "AutoHotkey is a scripting language for Windows that allows users to create custom automation scripts and macros. It is often used for automating repetitive tasks and customizing keyboard shortcuts." },
    AppItem { name: "Bitwarden", winget_id: "Bitwarden.Bitwarden", category: "Utilities", desc: "Bitwarden is an open-source password management solution. It allows users to store and manage their passwords in a secure and encrypted vault, accessible across multiple devices." },
    AppItem { name: "BleachBit", winget_id: "BleachBit.BleachBit", category: "Utilities", desc: "BleachBit is a free and open-source disk space cleaner and privacy manager. It helps users free up disk space by deleting unnecessary files and also protects privacy by cleaning up browsing history and other sensitive data." },
    AppItem { name: "Bulk Crap Uninstaller", winget_id: "Klocman.BulkCrapUninstaller", category: "Utilities", desc: "Bulk Crap Uninstaller (BCUninstaller) is a free and open-source program uninstaller for Windows. It allows users to uninstall multiple programs at once, as well as remove leftover files and registry entries." },
    AppItem { name: "Bulk Rename Utility", winget_id: "TGRMNSoftware.BulkRenameUtility", category: "Utilities", desc: "Bulk Rename Utility allows you to easily rename files and folders recursively based upon find-replace, character place, fields, sequences, regular expressions, EXIF data, and more." },
    AppItem { name: "CCleaner", winget_id: "Piriform.CCleaner", category: "Utilities", desc: "CCleaner is a utility program used to clean potentially unwanted files and invalid Windows Registry entries from a computer. It helps improve system performance and free up disk space." },
    AppItem { name: "CPU-Z", winget_id: "CPUID.CPU-Z", category: "Utilities", desc: "CPU-Z is a system monitoring and diagnostic tool for Windows. It provides detailed information about the computer's hardware components, including the CPU, memory, and motherboard." },
    AppItem { name: "CrystalDiskInfo", winget_id: "CrystalDewWorld.CrystalDiskInfo", category: "Utilities", desc: "Crystal Disk Info is a disk health monitoring tool that provides information about the status and performance of hard drives. It helps users anticipate potential issues and monitor drive health." },
    AppItem { name: "CrystalDiskMark", winget_id: "CrystalDewWorld.CrystalDiskMark", category: "Utilities", desc: "Crystal Disk Mark is a disk benchmarking tool that measures the read and write speeds of storage devices. It helps users assess the performance of their hard drives and SSDs." },
    AppItem { name: "DevToys", winget_id: "DevToys-app.DevToys", category: "Utilities", desc: "DevToys is a collection of development-related utilities and tools for Windows. It includes tools for file management, code formatting, and productivity enhancements for developers." },
    AppItem { name: "Dual Monitor Tools", winget_id: "GNE.DualMonitorTools", category: "Utilities", desc: "Dual Monitor Tools is a set of utilities designed to enhance the experience of using multiple monitors. It includes features for managing windows, wallpapers, and cursor movement across multiple screens." },
    AppItem { name: "Everything Search", winget_id: "voidtools.Everything", category: "Utilities", desc: "Everything is a fast file search utility for Windows. It indexes all files and folders on the computer, allowing users to quickly locate files by name or other attributes." },
    AppItem { name: "ExifCleaner", winget_id: "szTheory.exifcleaner", category: "Utilities", desc: "ExifCleaner is a tool for removing EXIF metadata from image files. It helps protect user privacy by stripping out sensitive information such as location data and camera settings." },
    AppItem { name: "File-Converter", winget_id: "AdrienAllard.FileConverter", category: "Utilities", desc: "File Converter is a context menu extension for Windows that allows users to quickly convert files between different formats. It supports a wide range of file types, including images, documents, and audio files." },
    AppItem { name: "F.lux", winget_id: "flux.flux", category: "Utilities", desc: "f.lux is a software application that adjusts the color temperature of your computer screen based on the time of day. It helps reduce eye strain and improve sleep quality by reducing blue light exposure in the evening." },
    AppItem { name: "Google Drive", winget_id: "Google.GoogleDrive", category: "Utilities", desc: "Google Drive is a cloud storage service that allows users to store and access files online. It offers file synchronization, sharing, and collaboration features." },
    AppItem { name: "GPU-Z", winget_id: "TechPowerUp.GPU-Z", category: "Utilities", desc: "GPU-Z is a lightweight utility that provides detailed information about the graphics card and GPU. It helps users monitor GPU performance and diagnose issues." },
    AppItem { name: "KDE Connect", winget_id: "KDE.KDEConnect", category: "Utilities", desc: "KDE Connect is a tool that allows seamless integration between your computer and mobile devices. It enables features like file sharing, notifications, and remote control." },
    AppItem { name: "LockHunter", winget_id: "CrystalRich.LockHunter", category: "Utilities", desc: "LockHunter is a file unlocker utility that helps users delete, move, or rename files that are locked by other processes. It provides a simple interface for managing locked files." },
    AppItem { name: "Malwarebytes", winget_id: "Malwarebytes.Malwarebytes", category: "Utilities", desc: "Malwarebytes is an anti-malware software that detects and removes malware, spyware, and other malicious software from your computer. It provides real-time protection and regular scans to keep your system secure." },
    AppItem { name: "Meld", winget_id: "Meld.Meld", category: "Utilities", desc: "Meld is a visual diff and merge tool that helps users compare files, directories, and version-controlled projects. It provides an intuitive interface for identifying differences and resolving conflicts." },
    AppItem { name: "NanaZip", winget_id: "M2Team.NanaZip", category: "Utilities", desc: "NanaZip is a free and open-source file archiver utility for Windows. It supports various compression formats and provides a user-friendly interface for managing archives." },
    AppItem { name: "Nextcloud Desktop Client", winget_id: "Nextcloud.NextcloudDesktop", category: "Utilities", desc: "Nextcloud Desktop Client is a synchronization tool that allows users to sync files between their local computer and a Nextcloud server. It provides seamless access to files and collaboration features." },
    AppItem { name: "Nilesoft Shell", winget_id: "Nilesoft.Shell", category: "Utilities", desc: "Nilesoft Shell is a customizable file explorer and shell replacement for Windows. It offers advanced features for file management and organization." },
    AppItem { name: "Notion", winget_id: "Notion.Notion", category: "Utilities", desc: "Notion is a productivity and note-taking application that combines notes, tasks, databases, and collaboration features into a single platform." },
    AppItem { name: "Nushell", winget_id: "Nushell.Nushell", category: "Utilities", desc: "Nushell is a modern shell and scripting language that combines the power of traditional shells with the flexibility of modern programming languages. It provides a rich set of features for data manipulation and automation." },
    AppItem { name: "OFGB (Oh Frick Go Back)", winget_id: "xM4ddy.OFGB", category: "Utilities", desc: "OFGB (Oh Frick Go Back) is a lightweight utility that allows users to quickly navigate back to the previous folder in File Explorer. It provides a simple and convenient way to improve file navigation. GUI Tool to remove ads from various places around Windows 11" },
    AppItem { name: "Oracle VirtualBox", winget_id: "Oracle.VirtualBox", category: "Utilities", desc: "Oracle VirtualBox is a powerful x86 and AMD64/Intel64 virtualization product for enterprise as well as home use. It allows users to run multiple operating systems simultaneously on a single physical machine." },
    AppItem { name: "ownCloud Desktop Client", winget_id: "ownCloud.ownCloudDesktop", category: "Utilities", desc: "ownCloud Desktop Client is a synchronization tool that allows users to sync files between their local computer and an ownCloud server. It provides seamless access to files and collaboration features." },
    AppItem { name: "PowerToys", winget_id: "Microsoft.PowerToys", category: "Utilities", desc: "Microsoft PowerToys is a set of utilities for power users to enhance productivity and customize the Windows experience. It includes tools like FancyZones, PowerRename, and File Explorer add-ons." },
    AppItem { name: "qBittorrent", winget_id: "qBittorrent.qBittorrent", category: "Utilities", desc: "qBittorrent is a free and open-source BitTorrent client that provides a user-friendly interface for downloading and managing torrent files. It offers features like sequential downloading, bandwidth scheduling, and RSS feed support." },
    AppItem { name: "Revo Uninstaller", winget_id: "RevoUninstaller.RevoUninstaller", category: "Utilities", desc: "Revo Uninstaller is a software uninstaller that helps users remove unwanted programs and clean up leftover files and registry entries. It provides advanced features for thorough uninstallation." },
    AppItem { name: "Rufus Imager", winget_id: "Rufus.Rufus", category: "Utilities", desc: "Rufus Imager is a tool for creating bootable USB drives from ISO images. It supports various operating systems and provides a simple interface for creating installation media." },
    AppItem { name: "TeamViewer", winget_id: "TeamViewer.TeamViewer", category: "Utilities", desc: "TeamViewer is a remote access and remote control software that allows users to connect to and control computers remotely. It is commonly used for remote support and collaboration." },
    AppItem { name: "Teracopy", winget_id: "CodeSector.TeraCopy", category: "Utilities", desc: "TeraCopy is a file transfer utility that enhances the speed and reliability of file copying and moving operations. It provides features like pause and resume, error recovery, and file verification." },
    AppItem { name: "TotalCommander", winget_id: "Ghisler.TotalCommander", category: "Utilities", desc: "Total Commander is a file manager for Windows that provides a dual-pane interface for efficient file management. It offers advanced features like file comparison, archive handling, and FTP support." },
    AppItem { name: "Transmission", winget_id: "Transmission.Transmission", category: "Utilities", desc: "Transmission is a lightweight and open-source BitTorrent client that provides a simple and user-friendly interface for downloading and managing torrent files." },
    AppItem { name: "UniGetUI", winget_id: "MartiCliment.UniGetUI", category: "Utilities", desc: "UniGetUI is a graphical user interface for the winget package manager. It allows users to easily search for, install, and manage applications from the winget repository." },
    AppItem { name: "WinRAR", winget_id: "RARLab.WinRAR", category: "Utilities", desc: "WinRAR is a popular file archiver utility that supports various compression formats, including RAR and ZIP. It provides a user-friendly interface for creating and managing archives." },
    AppItem { name: "WinZip", winget_id: "Corel.WinZip", category: "Utilities", desc: "WinZip is a file compression and archiving utility that allows users to create, manage, and share compressed files. It supports various formats and provides features like encryption and cloud integration." },

    AppItem { name: "", winget_id: "", category: "", desc: "" },

    // Communications
    AppItem { name: "Betterbird", winget_id: "Betterbird.Betterbird", category: "Communications", desc: "Betterbird is a fork of Mozilla Thunderbird with additional features and bugfixes."},
    AppItem { name: "BlueMail", winget_id: "Blix.BlueMail", category: "Communications", desc: "BlueMail is a versatile email client that supports multiple email accounts and provides features like unified inbox, customizable interface, and advanced security options." },
    AppItem { name: "Discord", winget_id: "Discord.Discord", category: "Communications", desc: "Discord is a popular communication platform designed for creating communities. It offers text, voice, and video chat features, making it ideal for gamers, hobbyists, and various interest groups." },
    AppItem { name: "Franz", winget_id: "StefanMalzner.Franz", category: "Communications", desc: "Franz is a messaging app that combines multiple chat and messaging services into a single interface. It supports platforms like WhatsApp, Facebook Messenger, Slack, and more." },
    AppItem { name: "Hexchat", winget_id: "HexChat.HexChat", category: "Communications", desc: "HexChat is an open-source IRC (Internet Relay Chat) client that allows users to connect to IRC networks and participate in chat rooms. It provides a customizable interface and supports various plugins and scripts." },
    AppItem { name: "Mumble (client)", winget_id: "Mumble.Mumble.Client", category: "Communications", desc: "Mumble is an open-source, low-latency, high-quality voice chat software primarily intended for use while gaming. It provides secure communication with features like positional audio and echo cancellation." },
    AppItem { name: "Mumble (server)", winget_id: "Mumble.Mumble.Server", category: "Communications", desc: "Mumble is an open-source, low-latency, high-quality voice chat software primarily intended for use while gaming. It provides secure communication with features like positional audio and echo cancellation." },
    AppItem { name: "Revolt", winget_id: "Revolt.RevoltDesktop", category: "Communications", desc: "Revolt is an open-source, privacy-focused chat platform that offers features like end-to-end encryption, group chats, and file sharing. It aims to provide a secure and user-friendly communication experience." },
    AppItem { name: "Signal", winget_id: "OpenWhisperSystems.Signal", category: "Communications", desc: "Signal is a secure messaging app that provides end-to-end encryption for text messages, voice calls, and video calls. It is known for its strong privacy features and commitment to user security." },
    AppItem { name: "Slack", winget_id: "SlackTechnologies.Slack", category: "Communications", desc: "Slack is a collaboration platform designed for teams and workplaces. It offers channels for organized communication, direct messaging, file sharing, and integration with various productivity tools." },
    AppItem { name: "Microsoft Teams", winget_id: "Microsoft.Teams", category: "Communications", desc: "Microsoft Teams is a collaboration platform that integrates with Microsoft 365. It provides chat, video conferencing, file sharing, and collaboration features for teams and organizations." },
    AppItem { name: "Pidgin", winget_id: "Pidgin.Pidgin", category: "Communications", desc: "Pidgin is a free and open-source instant messaging client that supports multiple chat networks, allowing users to connect to various messaging services in one application." },
    AppItem { name: "Proton Mail", winget_id: "Proton.ProtonMail", category: "Communications", desc: "Proton Mail is a secure email service that provides end-to-end encryption for email communication. It is designed to protect user privacy and offers features like self-destructing messages and anonymous email accounts." },
    AppItem { name: "Telegram", winget_id: "Telegram.TelegramDesktop", category: "Communications", desc: "Telegram is a cloud-based messaging app that offers fast and secure communication. It supports text messages, voice calls, video calls, and file sharing, with a focus on privacy and user control." },
    AppItem { name: "Thunderbird", winget_id: "Mozilla.Thunderbird", category: "Communications", desc: "Thunderbird is a free and open-source email client developed by Mozilla. It provides features like email management, calendar integration, and support for multiple email accounts." },
    AppItem { name: "Viber", winget_id: "Rakuten.Viber", category: "Communications", desc: "Viber is a messaging and voice-over-IP (VoIP) app that allows users to send text messages, make voice and video calls, and share multimedia files. It offers end-to-end encryption for secure communication." },
    AppItem { name: "Zoom", winget_id: "Zoom.Zoom", category: "Communications", desc: "Zoom is a video conferencing platform that enables users to host and join virtual meetings, webinars, and online events. It offers features like screen sharing, breakout rooms, and recording capabilities." },

    // Development
    AppItem { name: "Aegisub", winget_id: "Aegisub.Aegisub", category: "Development", desc: "Aegisub is a free, cross-platform, open-source and powerful tool for creating and modifying subtitles. Makes it quick and easy to time subtitles to audio."},
    AppItem { name: "Android Studio", winget_id: "Google.AndroidStudio", category: "Development", desc: "Android Studio is the official integrated development environment (IDE) for Google's Android operating system. It provides tools for building, testing, and debugging Android applications." },
    AppItem { name: "Arduino IDE", winget_id: "ArduinoSA.IDE.stable", category: "Development", desc: "The Arduino IDE is an open-source software platform used for programming Arduino microcontroller boards. It provides a simple and user-friendly interface for writing, compiling, and uploading code to Arduino devices." },
    AppItem { name: "Atom", winget_id: "GitHub.Atom", category: "Development", desc: "Atom is a free and open-source text editor developed by GitHub. It is highly customizable and supports various programming languages and extensions." },
    AppItem { name: "AutoIt", winget_id: "AutoIt.AutoIt", category: "Development", desc: "AutoIt is a scripting language designed for automating the Windows GUI and general scripting. It allows users to create scripts for automating repetitive tasks and creating custom applications." },
    AppItem { name: "Brackets", winget_id: "Adobe.Brackets", category: "Development", desc: "Brackets is a modern, open-source text editor designed for web development. It offers features like live preview, inline editing, and preprocessor support." },
    AppItem { name: "CMake", winget_id: "Kitware.CMake", category: "Development", desc: "CMake is an open-source, cross-platform family of tools designed to build, test, and package software. It is used to control the software compilation process using simple platform and compiler-independent configuration files." },
    AppItem { name: "Code::Blocks with MinGW", winget_id: "CodeBlocks.CodeBlocks.MinGW", category: "Development", desc: "Code::Blocks is a free, open-source cross-platform IDE that supports multiple compilers, including MinGW. It provides a user-friendly interface for C, C++, and Fortran development." },
    AppItem { name: "Docker Desktop", winget_id: "Docker.DockerDesktop", category: "Development", desc: "Docker Desktop is an application that enables developers to build, share, and run containerized applications on their local machine. It provides a user-friendly interface for managing Docker containers and images." },
    AppItem { name: "Eclipse IDE for c/C++ Dev", winget_id: "EclipseFoundation.Eclipse.CPP", category: "Development", desc: "Eclipse IDE for C/C++ Developers is a powerful, open-source integrated development environment (IDE) for C and C++ programming. It provides tools for code editing, debugging, and project management." },
    AppItem { name: "Eclipse IDE for Java Dev", winget_id: "EclipseFoundation.Eclipse.Java", category: "Development", desc: "Eclipse IDE for Java Developers is a powerful, open-source integrated development environment (IDE) for Java programming. It provides tools for code editing, debugging, and project management." },
    AppItem { name: "Eclipse IDE for Java and Web Dev", winget_id: "EclipseFoundation.Eclipse.JEE", category: "Development", desc: "Eclipse IDE for Java EE Developers is a powerful, open-source integrated development environment (IDE) for Java Enterprise Edition (EE) programming. It provides tools for code editing, debugging, and project management." },
    AppItem { name: "Eclipse IDE for PHP Dev", winget_id: "EclipseFoundation.Eclipse.PHP", category: "Development", desc: "Eclipse IDE for PHP Developers is a powerful, open-source integrated development environment (IDE) for PHP programming. It provides tools for code editing, debugging, and project management." },
    AppItem { name: "Fork", winget_id: "Fork.Fork", category: "Development", desc: "Fork is a fast and friendly Git client for both Mac and Windows. It provides a visual interface for managing Git repositories, making it easier to perform version control tasks." },
    AppItem { name: "Git", winget_id: "Git.Git", category: "Development", desc: "Git is a distributed version control system that allows developers to track changes in source code during software development. It enables collaboration and helps manage code history." },
    AppItem { name: "Git Butler", winget_id: "GitButler.GitButler", category: "Development", desc: "Git Butler is a graphical user interface (GUI) for Git that simplifies version control tasks. It provides an intuitive interface for managing repositories, branches, and commits." },
    AppItem { name: "Git Extensions", winget_id: "GitExtensionsTeam.GitExtensions", category: "Development", desc: "Git Extensions is a graphical user interface (GUI) for Git that provides a visual way to manage Git repositories. It offers features like commit history visualization, branch management, and integration with external tools." },
    AppItem { name: "GitHub CLI", winget_id: "GitHub.cli", category: "Development", desc: "GitHub CLI is a command-line interface for GitHub that allows users to interact with GitHub repositories and perform various tasks directly from the terminal." },
    AppItem { name: "GitHub Desktop", winget_id: "GitHub.GitHubDesktop", category: "Development", desc: "GitHub Desktop is a graphical user interface (GUI) for Git that simplifies version control tasks. It provides an intuitive interface for managing repositories, branches, and commits." },
    AppItem { name: "Gitify", winget_id: "Gitify.Gitify", category: "Development", desc: "Gitify is a desktop application that provides notifications and quick access to GitHub repositories. It helps developers stay updated on repository activity and manage their projects efficiently." },
    AppItem { name: "IntelliJ IDEA Community", winget_id: "JetBrains.IntelliJIDEA.Community", category: "Development", desc: "IntelliJ IDEA Community Edition is a free and open-source IDE for Java, Kotlin, Groovy, Scala, and Android development. It offers a range of features for code editing, debugging, and version control." },
    AppItem { name: "Jetbrains Toolbox", winget_id: "JetBrains.Toolbox", category: "Development", desc: "JetBrains Toolbox is a management tool that allows users to install, update, and manage JetBrains development tools and IDEs from a single interface." },
    AppItem { name: "NetBeans", winget_id: "Apache.NetBeans", category: "Development", desc: "Apache NetBeans is a free and open-source integrated development environment (IDE) for Java, JavaScript, PHP, and other programming languages. It provides tools for code editing, debugging, and project management." },
    AppItem { name: "Neovim", winget_id: "Neovim.Neovim", category: "Development", desc: "Neovim is a highly extensible text editor and an improvement over the original Vim editor." },
    AppItem { name: "NodeJS Current", winget_id: "OpenJS.NodeJS", category: "Development", desc: "Node.js is a JavaScript runtime built on Chrome's V8 JavaScript engine. It allows developers to build scalable and high-performance server-side applications using JavaScript." },
    AppItem { name: "NodeJS LTS", winget_id: "OpenJS.NodeJS.LTS", category: "Development", desc: "Node.js is a JavaScript runtime built on Chrome's V8 JavaScript engine. It allows developers to build scalable and high-performance server-side applications using JavaScript. The LTS (Long-Term Support) version is recommended for most users as it provides stability and extended support." },
    AppItem { name: "Node Version Manager (nvm)", winget_id: "CoreyButler.NVMforWindows", category: "Development", desc: "Node Version Manager (nvm) is a version manager for Node.js that allows users to easily switch between different versions of Node.js on their system." },
    AppItem { name: "Oh My Posh (Prompt)", winget_id: "JanDeDobbeleer.OhMyPosh", category: "Development", desc: "Oh My Posh is a prompt theme engine for PowerShell and other shells. It allows users to customize their command prompt with various themes and styles." },
    AppItem { name: "PHPStorm", winget_id: "JetBrains.PhpStorm", category: "Development", desc: "PhpStorm is a commercial, cross-platform IDE for PHP, built by JetBrains. It provides tools and features for efficient PHP development, including code completion, debugging, and version control integration." },
    AppItem { name: "Postman", winget_id: "Postman.Postman", category: "Development", desc: "Postman is a collaboration platform for API development. It provides tools for designing, testing, and documenting APIs, making it easier for developers to work with APIs." },
    AppItem { name: "Pulsar", winget_id: "Pulsar-Edit.Pulsar", category: "Development", desc: "A Community-led Hyper-Hackable Text Editor, Forked from Atom, built on Electron. Designed to be deeply customizable, but still approachable using the default configuration." },
    AppItem { name: "PyCharm", winget_id: "JetBrains.PyCharm", category: "Development", desc: "PyCharm is an integrated development environment (IDE) used for programming in Python. It provides code analysis, a graphical debugger, an integrated unit tester, and supports web development with Django." },
    AppItem { name: "Python 3", winget_id: "Python.Python.3.13", category: "Development", desc: "Python is a high-level, interpreted programming language known for its simplicity and readability. It is widely used for web development, data analysis, artificial intelligence, and scientific computing." },
    AppItem { name: "RubyMine", winget_id: "JetBrains.RubyMine", category: "Development", desc: "RubyMine is a cross-platform IDE for Ruby and Ruby on Rails development, built by JetBrains. It provides tools and features for efficient Ruby development, including code completion, debugging, and version control integration." },
    AppItem { name: "Rust", winget_id: "Rustlang.Rust.MSVC", category: "Development", desc: "Rust is a systems programming language that focuses on safety, speed, and concurrency. It is designed to provide memory safety without sacrificing performance." },
    AppItem { name: "Sublime Text", winget_id: "SublimeHQ.SublimeText.4", category: "Development", desc: "Sublime Text is a sophisticated text editor for code, markup, and prose. It offers a sleek interface, powerful features, and a wide range of plugins to enhance productivity." },
    AppItem { name: "Thonny Python IDE", winget_id: "AivarAnnamaa.Thonny", category: "Development", desc: "Thony Python IDE is a Python IDE for beginners" },
    AppItem { name: "Vim", winget_id: "vim.vim", category: "Development", desc: "Vim is a highly configurable text editor built to enable efficient text editing. It is an improved version of the vi editor and is widely used by programmers and system administrators." },
    AppItem { name: "Visual Studio 2022", winget_id: "Microsoft.VisualStudio.2022.Community", category: "Development", desc: "Visual Studio 2022 is an integrated development environment (IDE) from Microsoft. It provides a comprehensive set of tools for developing applications across various platforms, including web, mobile, and desktop." },
    AppItem { name: "VS Code", winget_id: "Microsoft.VisualStudioCode", category: "Development", desc: "Visual Studio Code (VS Code) is a lightweight, open-source code editor developed by Microsoft. It supports various programming languages and provides features like debugging, version control, and extensions." },
    AppItem { name: "VS Codium", winget_id: "VSCodium.VSCodium", category: "Development", desc: "VSCodium is a community-driven, freely-licensed binary distribution of Microsoft's VS Code editor. It removes telemetry and branding, providing a more privacy-focused alternative." },
    AppItem { name: "Zed", winget_id: "Zed.Zed", category: "Development", desc: "Zed is a modern, high-performance code editor designed from the ground for and collabotation." },
    AppItem { name: "XAMPP 8.2", winget_id: "ApacheFriends.Xampp.8.2", category: "Development", desc: "XAMPP is a free and open-source cross-platform web server solution stack package developed by Apache Friends, consisting mainly of the Apache HTTP Server, MariaDB database, and interpreters for scripts written in the PHP and Perl programming languages." },
    AppItem { name: "WampServer", winget_id: "Wampserver.Wampserver", category: "Development", desc: "WampServer is a Windows web development environment that allows you to create web applications with Apache2, PHP, and a MySQL database. It also comes with PHPMyAdmin and SQLiteManager to manage your databases." },
    AppItem { name: "WebStorm", winget_id: "JetBrains.WebStorm", category: "Development", desc: "WebStorm is a powerful IDE for modern JavaScript development, built by JetBrains. It provides tools and features for efficient web development, including code completion, debugging, and version control integration." },

    // Document
    AppItem { name: "Adobe Acrobat Reader", winget_id: "Adobe.Acrobat.Reader.64-bit", category: "Document", desc: "Adobe Acrobat Reader is a free software application developed by Adobe Inc. that allows users to view, print, and annotate PDF (Portable Document Format) files. It is widely used for reading and interacting with PDF documents." },
    AppItem { name: "AFFiNE", winget_id: "ToEverything.AFFiNE", category: "Document", desc: "AFFiNE is a privacy-focused, local-first, open-source, and ready-to-use alternative for Notion & Miro. One hyper-fused platform for wildly creative minds." },
    AppItem { name: "Calibre", winget_id: "calibre.calibre", category: "Document", desc: "Calibre is a free and open-source e-book management software that allows users to organize, convert, and read e-books. It supports a wide range of e-book formats and provides tools for managing e-book libraries." },
    AppItem { name: "Foxit PDF Editor", winget_id: "Foxit.PhantomPDF", category: "Document", desc: "Foxit PDF Editor is a powerful PDF editing software that allows users to create, edit, and manage PDF documents. It provides features like text editing, annotation, and form filling." },
    AppItem { name: "Foxit PDF Reader", winget_id: "Foxit.FoxitReader", category: "Document", desc: "Foxit PDF Reader is a lightweight and fast PDF viewer that allows users to view, annotate, and sign PDF documents. It provides features like tabbed viewing, text highlighting, and form filling." },
    AppItem { name: "Grammarly for Windows", winget_id: "Grammarly.Grammarly", category: "Document", desc: "Grammarly for Windows is a writing assistant that helps users improve their writing by providing grammar, spelling, and style suggestions. It integrates with various applications and platforms to enhance writing quality." },
    AppItem { name: "LibreOffice", winget_id: "TheDocumentFoundation.LibreOffice", category: "Document", desc: "LibreOffice is a free and open-source office suite that includes applications for word processing, spreadsheets, presentations, and more. It is compatible with various file formats, including Microsoft Office formats." },
    AppItem { name: "NAPS2 (Document Scanner)", winget_id: "XPFPGHZZ8M7MMH", category: "Document", desc: "Scan documents to PDF and other file types, as simply as possible. Scan multiple pages, from both flatbed and automatic feeder, then export as a single PDF file. Support scanners from all manufacturers, including Canon, HP, Brother, Xerox, Epson, Fujitsu..." },
    AppItem { name: "Notepad++", winget_id: "Notepad++.Notepad++", category: "Document", desc: "Notepad++ is a free source code editor and Notepad replacement that supports several programming languages and is highly customizable." },
    AppItem { name: "Notion", winget_id: "Notion.Notion", category: "Document", desc: "Notion agents keep work moving 24/7. They capture knowledge, answer questions, and push projects forward—all while you sleep." },
    AppItem { name: "Obsidian", winget_id: "Obsidian.Obsidian", category: "Document", desc: "Obsidian stores notes privately on your device, so you can access them quickly, even offline. No one else can read them, not even us." },
    AppItem { name: "Okular", winget_id: "KDE.Okular", category: "Document", desc: "Okular is a free and open-source document viewer developed by the KDE community. It supports various document formats, including PDF, EPUB, and images, and provides features like annotations and bookmarks." },
    AppItem { name: "ONLYOffice Desktop", winget_id: "ONLYOFFICE.DesktopEditors", category: "Document", desc: "OnlyOffice is a free and open-source office suite that includes applications for word processing, spreadsheets, and presentations. It is compatible with various file formats, including Microsoft Office formats." },
    AppItem { name: "PDF24 creator", winget_id: "geeksoftwareGmbH.PDF24Creator", category: "Document", desc: "PDF24 Creator is a free PDF creation and editing software that allows users to create, edit, and convert PDF documents. It provides features like merging, splitting, and compressing PDF files." },
    AppItem { name: "PDFgear", winget_id: "PDFgear.PDFgear", category: "Document", desc: "PDFgear is a free and easy-to-use PDF editor that allows users to view, edit, and annotate PDF documents. It provides features like text editing, image manipulation, and form filling." },
    AppItem { name: "Scribus", winget_id: "Scribus.Scribus", category: "Document", desc: "Scribus is a free and open-source desktop publishing software that allows users to create professional-quality documents, including brochures, newsletters, and magazines. It provides tools for layout design, typography, and color management." },
    AppItem { name: "Sumatra PDF", winget_id: "SumatraPDF.SumatraPDF", category: "Document", desc: "Sumatra PDF is a lightweight and fast PDF viewer that allows users to view PDF documents quickly and efficiently. It supports various document formats, including PDF, EPUB, MOBI, and images." },
    AppItem { name: "WinMerge", winget_id: "WinMerge.WinMerge", category: "Document", desc: "WinMerge is an open source differencing and merging tool for Windows. WinMerge can compare files and folders, presenting differences in a visual format that is easy to understand and manipulate." },
    AppItem { name: "WPS Office", winget_id: "Kingsoft.WPSOffice", category: "Document", desc: "WPS Office is a free office suite that includes applications for word processing, spreadsheets, and presentations. It is compatible with various file formats, including Microsoft Office formats." },
    AppItem { name: "Xournal++", winget_id: "Xournal++.Xournal++", category: "Document", desc: "Xournal++ is an open-source and cross-platform note-taking software that is fast, flexible, and functional. A modern rewrite and a more feature-rich version of the wonderful Xournal program." },

    // Multimedia
    AppItem { name: "AIMP (Music Player)", winget_id: "AIMP.AIMP", category: "Multimedia Tools", desc: "AIMP is a free and feature-rich music player that supports various audio formats and provides a customizable interface. It offers features like playlist management, audio effects, and internet radio support." },
    AppItem { name: "Audacity", winget_id: "Audacity.Audacity", category: "Multimedia Tools", desc: "Audacity is a free and open-source digital audio editor and recording application. It provides tools for recording, editing, and mixing audio tracks, making it suitable for podcasting, music production, and sound design." },
    AppItem { name: "Blender (3D Graphics)", winget_id: "BlenderFoundation.Blender", category: "Multimedia Tools", desc: "Blender is a free and open-source 3D creation suite that supports the entire 3D pipeline, including modeling, rigging, animation, simulation, rendering, compositing, and motion tracking. It is widely used for creating 3D graphics, animations, and visual effects." },
    AppItem { name: "Clementine", winget_id: "Clementine.Clementine", category: "Multimedia Tools", desc: "Clementine is a free and open-source music player and library organizer. It provides a user-friendly interface for managing and playing music collections, as well as features like internet radio support and playlist creation." },
    AppItem { name: "EarTrumpet (Audio)", winget_id: "File-New-Project.EarTrumpet", category: "Multimedia Tools", desc: "EarTrumpet is a free and open-source volume control app for Windows that provides advanced audio management features. It allows users to control the volume of individual applications and provides a more intuitive interface for managing audio devices." },
    AppItem { name: "FFmpeg (full)", winget_id: "Gyan.FFmpeg", category: "Multimedia Tools", desc: "FFmpeg is a free and open-source multimedia framework that allows users to record, convert, and stream audio and video files. It supports a wide range of formats and provides powerful command-line tools for multimedia processing." },
    AppItem { name: "FreeCAD", winget_id: "FreeCAD.FreeCAD", category: "Multimedia Tools", desc: "FreeCAD is a free and open-source parametric 3D CAD modeler that allows users to create and modify 3D models. It is suitable for a wide range of applications, including product design, mechanical engineering, and architecture." },
    AppItem { name: "FxSound", winget_id: "FxSound.FxSound", category: "Multimedia Tools", desc: "FxSound is a premium audio enhancement software that improves the sound quality of music and other audio content. It provides features like 3D surround sound, bass boost, and equalizer presets." },
    AppItem { name: "GIMP (Image Editor)", winget_id: "GIMP.GIMP.3", category: "Multimedia Tools", desc: "GIMP (GNU Image Manipulation Program) is a free and open-source raster graphics editor used for image retouching, editing, and composition. It provides a wide range of tools and features for photo manipulation and graphic design." },
    AppItem { name: "ImgBurn", winget_id: "LIGHTNINGUK.ImgBurn", category: "Multimedia Tools", desc: "ImgBurn is a free and lightweight disc burning software that allows users to create, burn, and verify CD, DVD, and Blu-ray discs. It supports various image file formats and provides advanced features for disc burning." },
    AppItem { name: "Inkscape (Vector Graphics)", winget_id: "Inkscape.Inkscape", category: "Multimedia Tools", desc: "Inkscape is a free and open-source vector graphics editor used for creating and editing scalable vector graphics (SVG) files. It provides a wide range of tools and features for graphic design, illustration, and web graphics." },
    AppItem { name: "iTunes", winget_id: "Apple.iTunes", category: "Multimedia Tools", desc: "iTunes is a media player, media library, and mobile device management application developed by Apple Inc. It allows users to organize and play music and videos, as well as manage their iOS devices." },
    AppItem { name: "Kdenlive (Video Editor)", winget_id: "KDE.Kdenlive", category: "Multimedia Tools", desc: "Kdenlive is a free and open-source video editing software that provides a user-friendly interface for creating and editing video projects. It offers features like multi-track editing, effects, and transitions." },
    AppItem { name: "K-Lite Codec Standard", winget_id: "CodecGuide.K-LiteCodecPack.Standard", category: "Multimedia Tools", desc: "K-Lite Codec Pack is a collection of audio and video codecs that allows users to play a wide range of multimedia formats on their Windows system. The Standard version includes essential codecs for common formats." },
    AppItem { name: "Kodi Media Center", winget_id: "XBMCFoundation.Kodi", category: "Multimedia Tools", desc: "Kodi is a free and open-source media center software that allows users to play and organize digital media files, including videos, music, and photos. It supports various plugins and add-ons for extended functionality." },
    AppItem { name: "Krita (Image Editor)", winget_id: "KDE.Krita", category: "Multimedia Tools", desc: "Krita is a free and open-source digital painting and illustration software. It provides a wide range of tools and features for artists, including brushes, layers, and color management." },
    AppItem { name: "OBS Studio", winget_id: "OBSProject.OBSStudio", category: "Multimedia Tools", desc: "OBS Studio (Open Broadcaster Software) is a free and open-source software for video recording and live streaming. It provides a user-friendly interface for capturing and mixing video and audio sources, making it popular among content creators and streamers." },
    AppItem { name: "Paint.NET", winget_id: "dotPDN.PaintDotNet", category: "Multimedia Tools", desc: "Paint.NET is a free and user-friendly image editing software that provides a range of tools and features for photo manipulation and graphic design. It offers a simple interface and supports layers, effects, and plugins." },
    AppItem { name: "Plex Desktop", winget_id: "Plex.Plex", category: "Multimedia Tools", desc: "Plex is a media server software that allows users to organize, stream, and share their digital media files, including movies, TV shows, music, and photos. It provides a user-friendly interface and supports various devices for media playback." },
    AppItem { name: "Plex Media Server", winget_id: "Plex.PlexMediaServer", category: "Multimedia Tools", desc: "Plex is a media server software that allows users to organize, stream, and share their digital media files, including movies, TV shows, music, and photos. It provides a user-friendly interface and supports various devices for media playback." },
    AppItem { name: "Shotcut (Video Editor)", winget_id: "Meltytech.Shotcut", category: "Multimedia Tools", desc: "Shotcut is a free and open-source video editing software that provides a user-friendly interface for creating and editing video projects. It offers features like multi-track editing, effects, and transitions." },
    AppItem { name: "SMPlayer", winget_id: "SMPlayer.SMPlayer", category: "Multimedia Tools", desc: "SMPlayer is a free and open-source multimedia player that supports a wide range of audio and video formats. It provides features like subtitle support, playback speed control, and customizable interface." },
    AppItem { name: "Spotify", winget_id: "Spotify.Spotify", category: "Multimedia Tools", desc: "Spotify is a popular music streaming service that provides access to a vast library of songs, albums, and playlists. It offers personalized recommendations, offline listening, and social sharing features." },
    AppItem { name: "Stremio", winget_id: "Stremio.Stremio", category: "Multimedia Tools", desc: "Stremio is a free and open-source media center application that allows users to organize, stream, and watch video content from various sources. It provides a user-friendly interface and supports various add-ons for extended functionality." },
    AppItem { name: "VLC", winget_id: "VideoLAN.VLC", category: "Multimedia Tools", desc: "VLC (VideoLAN Client) is a free and open-source multimedia player that supports a wide range of audio and video formats. It provides features like media playback, streaming, and media conversion." },
    AppItem { name: "Yt-dlp (YouTube Downloader)", winget_id: "yt-dlp.yt-dlp", category: "Multimedia Tools", desc: "yt-dlp is a command-line program that allows users to download videos from YouTube and other video-sharing platforms. It supports various video formats and provides options for video quality and metadata." },
    AppItem { name: "Winamp", winget_id: "Winamp.Winamp", category: "Multimedia Tools", desc: "Winamp responds to the needs of listeners by offering them features designed to create a personal experience." },   

    // Games/platforms
    AppItem { name: "EA App", winget_id: "ElectronicArts.EADesktop", category: "Games", desc: "The EA App is a digital distribution platform developed by Electronic Arts (EA) for purchasing and playing video games. It serves as a hub for EA's game library, providing access to popular titles such as The Sims, FIFA, Battlefield, and more. The app offers features like game downloads, updates, social integration, and exclusive content for EA games." },
    AppItem { name: "Battle.net", winget_id: "Blizzard.BattleNet", category: "Games", desc: "Battle.net is an online gaming platform developed by Blizzard Entertainment. It serves as a hub for Blizzard's game library, including popular titles like World of Warcraft, Overwatch, Diablo, and StarCraft. The platform offers features such as game downloads, updates, social integration, and access to multiplayer gaming." },
    AppItem { name: "Epic Games Launcher", winget_id: "EpicGames.EpicGamesLauncher", category: "Games", desc: "The Epic Games Launcher is a digital distribution platform developed by Epic Games for purchasing and playing video games. It serves as a hub for Epic Games' game library, including popular titles like Fortnite, Unreal Tournament, and more. The launcher offers features such as game downloads, updates, social integration, and access to exclusive content and free games." },
    AppItem { name: "GOG Galaxy", winget_id: "GOG.Galaxy", category: "Games", desc: "GOG Galaxy is a digital distribution platform developed by GOG.com for purchasing and playing video games. It serves as a hub for GOG's game library, which includes DRM-free titles from various developers and publishers. The platform offers features such as game downloads, updates, cloud saves, achievements, and social integration." },
    AppItem { name: "PS Remote Play", winget_id: "PlayStation.PSRemotePlay", category: "Games", desc: "PS Remote Play is an application developed by Sony that allows users to stream and play PlayStation games on their PC or mobile devices. It enables players to connect to their PlayStation console remotely and access their game library, providing a seamless gaming experience outside of the console." },
    AppItem { name: "Steam", winget_id: "Valve.Steam", category: "Games", desc: "Steam is a digital distribution platform developed by Valve Corporation for purchasing and playing video games. It serves as a hub for a vast library of games from various developers and publishers, offering features such as game downloads, updates, cloud saves, achievements, and social integration." },
    AppItem { name: "Ubisoft Connect", winget_id: "Ubisoft.Connect", category: "Games", desc: "Ubisoft Connect is a digital distribution platform developed by Ubisoft for purchasing and playing video games. It serves as a hub for Ubisoft's game library, including popular titles like Assassin's Creed, Far Cry, and Rainbow Six. The platform offers features such as game downloads, updates, cloud saves, achievements, and social integration." },    

    // Microsoft tools
    AppItem { name: "Autoruns", winget_id: "Microsoft.Sysinternals.Autoruns", category: "Microsoft Tools", desc: "Autoruns is a free utility from Microsoft Sysinternals that provides detailed information about the programs and services that are configured to run automatically when a Windows system starts. It allows users to manage and disable startup items, helping to improve system performance and troubleshoot issues." },
    AppItem { name: ".NET Desktop Runtime 3.1", winget_id: "Microsoft.DotNet.DesktopRuntime.3_1", category: "Microsoft Tools", desc: ".NET Desktop Runtime 3.1 is a software framework developed by Microsoft that allows users to run desktop applications built using the .NET framework. It provides the necessary runtime components and libraries for executing .NET applications on Windows systems." },
    AppItem { name: ".NET Desktop Runtime 5", winget_id: "Microsoft.DotNet.DesktopRuntime.5", category: "Microsoft Tools", desc: ".NET Desktop Runtime 5 is a software framework developed by Microsoft that allows users to run desktop applications built using the .NET framework. It provides the necessary runtime components and libraries for executing .NET applications on Windows systems." },
    AppItem { name: ".NET Desktop Runtime 6", winget_id: "Microsoft.DotNet.DesktopRuntime.6", category: "Microsoft Tools", desc: ".NET Desktop Runtime 6 is a software framework developed by Microsoft that allows users to run desktop applications built using the .NET framework. It provides the necessary runtime components and libraries for executing .NET applications on Windows systems." },
    AppItem { name: ".NET Desktop Runtime 7", winget_id: "Microsoft.DotNet.DesktopRuntime.7", category: "Microsoft Tools", desc: ".NET Desktop Runtime 7 is a software framework developed by Microsoft that allows users to run desktop applications built using the .NET framework. It provides the necessary runtime components and libraries for executing .NET applications on Windows systems." },
    AppItem { name: ".NET Desktop Runtime 8", winget_id: "Microsoft.DotNet.DesktopRuntime.8", category: "Microsoft Tools", desc: ".NET Desktop Runtime 8 is a software framework developed by Microsoft that allows users to run desktop applications built using the .NET framework. It provides the necessary runtime components and libraries for executing .NET applications on Windows systems." },
    AppItem { name: ".NET Desktop Runtime 9", winget_id: "Microsoft.DotNet.DesktopRuntime.9", category: "Microsoft Tools", desc: ".NET Desktop Runtime 9 is a software framework developed by Microsoft that allows users to run desktop applications built using the .NET framework. It provides the necessary runtime components and libraries for executing .NET applications on Windows systems." },
    AppItem { name: "NuGet Package Manager", winget_id: "Microsoft.NuGet", category: "Microsoft Tools", desc: "NuGet Package Manager is a free and open-source package manager for the Microsoft development platform, including .NET. It allows developers to easily manage and install third-party libraries and tools in their projects, simplifying the process of adding functionality and dependencies." },
    AppItem { name: "Microsoft OneDrive", winget_id: "Microsoft.OneDrive", category: "Microsoft Tools", desc: "Microsoft OneDrive is a cloud storage service that allows users to store, sync, and share files and folders online. It provides seamless integration with Windows and Microsoft Office applications, enabling users to access their files from any device with an internet connection." },
    AppItem { name: "Microsoft Visual C++ Redistributable 2015-2022", winget_id: "Microsoft.VCRedist.2015+.x64", category: "Microsoft Tools", desc: "The Microsoft Visual C++ Redistributable is a package of runtime components required to run applications developed with Visual C++. It includes libraries such as the C Runtime (CRT), Standard C++, MFC, and others. Installing the redistributable ensures that applications built with Visual C++ can run properly on a system without needing to install the full development environment." },
    AppItem { name: "Microsoft Visual C++ Redistributable 2015-2022 (x86)", winget_id: "Microsoft.VCRedist.2015+.x86", category: "Microsoft Tools", desc: "The Microsoft Visual C++ Redistributable is a package of runtime components required to run applications developed with Visual C++. It includes libraries such as the C Runtime (CRT), Standard C++, MFC, and others. Installing the redistributable ensures that applications built with Visual C++ can run properly on a system without needing to install the full development environment." },
    AppItem { name: "PowerShell", winget_id: "Microsoft.PowerShell", category: "Microsoft Tools", desc: "PowerShell is a task automation and configuration management framework developed by Microsoft. It consists of a command-line shell and a scripting language that allows users to automate administrative tasks and manage system configurations." },
    AppItem { name: "PowerToys", winget_id: "Microsoft.PowerToys", category: "Microsoft Tools", desc: "PowerToys is a set of free and open-source utilities developed by Microsoft that enhance the Windows operating system. It provides additional features and tools for power users, such as window management, keyboard shortcuts, and file renaming." },
    AppItem { name: "Windows Terminal", winget_id: "Microsoft.WindowsTerminal", category: "Microsoft Tools", desc: "Windows Terminal is a modern, feature-rich terminal application developed by Microsoft. It provides a multi-tabbed interface for accessing various command-line tools and shells, including PowerShell, Command Prompt, and WSL (Windows Subsystem for Linux). Windows Terminal offers customization options, such as themes, fonts, and keyboard shortcuts, making it a powerful tool for developers and system administrators." },

    // Pro Tools
    AppItem { name: "Advanced IP Scanner", winget_id: "Famatech.AdvancedIPScanner", category: "Pro Tools", desc: "Advanced IP Scanner is a free and easy-to-use network scanning tool that allows users to quickly scan and analyze their local network. It provides information about connected devices, including IP addresses, MAC addresses, and device names, making it useful for network management and troubleshooting." },
    AppItem { name: "Angry IP Scanner", winget_id: "angryziber.AngryIPScanner", category: "Pro Tools", desc: "Angry IP Scanner is a free and open-source network scanning tool that allows users to quickly scan and analyze their local network. It provides information about connected devices, including IP addresses, MAC addresses, and device names, making it useful for network management and troubleshooting." },
    AppItem { name: "EFI Boot Editor", winget_id: "EFIBootEditor.EFIBootEditor", category: "Pro Tools", desc: "Boot Editor is a tool for managining EFI/UEFI boot entries on your system. It allows you to customize the boot configuration of your computer start mode." },
    AppItem { name: "HeidiSQL", winget_id: "HeidiSQL.HeidiSQL", category: "Pro Tools", desc: "HeidiSQL is free software for people who work with databases, and aims to be intuitive to use. Heidi lets you connect to a variety of databases, like MariaDB, MySQL, Microsoft SQL, PostgreSQL, SQLite, Interbase and Firebird." },
    AppItem { name: "Mullvad VPN", winget_id: "MullvadVPN.MullvadVPN", category: "Pro Tools", desc: "Mullvad VPN is a privacy-focused virtual private network (VPN) service that provides secure and anonymous internet browsing. It offers features like strong encryption, no-logs policy, and support for various platforms, making it a popular choice for users seeking online privacy and security." },
    AppItem { name: "Nmap", winget_id: "Insecure.Nmap", category: "Pro Tools", desc: "Nmap (Network Mapper) is a free and open-source network scanning tool that allows users to discover and analyze devices on a network. It provides features like host discovery, port scanning, and service detection, making it useful for network management, security auditing, and penetration testing." },
    AppItem { name: "OpenVPN Connect", winget_id: "OpenVPNTechnologies.OpenVPNConnect", category: "Pro Tools", desc: "OpenVPN Connect is the official VPN client for the OpenVPN protocol. It allows users to securely connect to VPN servers and access private networks over the internet. The client provides features like strong encryption, authentication, and support for various platforms, making it a popular choice for users seeking online privacy and security." },
    AppItem { name: "PuTTY", winget_id: "PuTTY.PuTTY", category: "Pro Tools", desc: "PuTTY is a free and open-source terminal emulator, serial console, and network file transfer application. It supports various network protocols, including SSH, Telnet, and SCP, making it a popular choice for remote access and management of servers and network devices." },
    AppItem { name: "RustDesk", winget_id: "RustDesk.RustDesk", category: "Pro Tools", desc: "RustDesk is a free and open-source remote desktop software that allows users to securely access and control remote computers. It provides features like file transfer, chat, and multi-platform support, making it a popular choice for remote work and technical support." },
    AppItem { name: "Ventoy", winget_id: "Ventoy.Ventoy", category: "Pro Tools", desc: "Ventoy is a free and open-source tool that allows users to create bootable USB drives for installing operating systems. It supports multiple ISO files on a single USB drive, making it easy to create and manage bootable media for various operating systems." },
    AppItem { name: "Wireshark", winget_id: "WiresharkFoundation.Wireshark", category: "Pro Tools", desc: "Wireshark is a free and open-source network protocol analyzer that allows users to capture and analyze network traffic in real-time. It provides detailed information about network protocols, packets, and communication patterns, making it a powerful tool for network troubleshooting, security analysis, and protocol development." },
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

/// Stocăm query-ul de search global pentru tab
static SEARCH_QUERY: OnceLock<Mutex<String>> = OnceLock::new();
fn search_query() -> &'static Mutex<String> {
    SEARCH_QUERY.get_or_init(|| Mutex::new(String::new()))
}

pub fn show_install(ui: &mut egui::Ui, log: &Arc<Mutex<String>>) {
    // Snapshot info
    let (sel_count, running, p_snapshot) = {
        let st = state().lock().unwrap();
        let p = st.progress.lock().unwrap().clone();
        (st.selected.len(), p.running, p)
    };
    ui.vertical_centered(|ui| {
        let title = egui::RichText::new(
            "Adjust the window size to full screen or scroll down for an optimal experience!",
        )
        .size(20.0) // ajustează mărimea după nevoie (ex: 18.0, 22.0)
        .color(egui::Color32::from_rgb(0, 180, 0)) // verde
        .strong();
        ui.label(title);
    });

    ui.separator();
    ui.add_space(6.0);

    ui.group(|ui| {
        if running {
            ui.label(format!("Working on: {}", p_snapshot.current_name));
            let frac = if p_snapshot.total > 0 {
                p_snapshot.current as f32 / p_snapshot.total as f32
            } else { 0.0 };
            ui.add(egui::ProgressBar::new(frac).text(format!("{} / {}", p_snapshot.current, p_snapshot.total)));
        } else {
            ui.label(if sel_count > 0 {
                format!("Selected: {} apps", sel_count)
            } else {
                "Select apps then choose an action.".to_owned()
            });
        }
    });

    ui.add_space(6.0);

    // Action buttons (styled)
    {
        let draw_action_btn = |ui: &mut egui::Ui, label: &str, enabled: bool| -> egui::Response {
            let min_size = egui::Vec2::new(150.0, 30.0);
            let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
            let (rect, resp) = ui.allocate_at_least(min_size, sense);

            let visuals = ui.style().visuals.clone();
            let normal_bg = visuals.widgets.inactive.bg_fill;
            let hover_bg = egui::Color32::from_rgb(0, 120, 215);

            let bg_fill = if resp.hovered() && enabled { hover_bg } else { normal_bg };
            let stroke_col = if resp.hovered() { egui::Color32::from_rgb(0,255,140) } else { egui::Color32::from_gray(160) };

            ui.painter().rect(rect, 6.0, bg_fill, egui::Stroke::new(1.5, stroke_col), egui::StrokeKind::Middle);
            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let text_col = if resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_rgb(0,255,140) };
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_id, text_col);

            if resp.hovered() && enabled {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            resp
        };

        let mut r_install: Option<egui::Response> = None;
        let mut r_uninstall: Option<egui::Response> = None;
        let mut r_update: Option<egui::Response> = None;
        let mut r_clear: Option<egui::Response> = None;
        let mut r_upgrade: Option<egui::Response> = None;

        ui.horizontal(|ui| {
            let disabled = sel_count == 0 || running;
            r_install  = Some(draw_action_btn(ui, "Install selections", !disabled));
            ui.add_space(6.0);
            r_uninstall = Some(draw_action_btn(ui, "Uninstall selections", !disabled));
            ui.add_space(6.0);
            r_update = Some(draw_action_btn(ui, "Update selections", true));
            ui.add_space(6.0);
            r_clear = Some(draw_action_btn(ui, "Clear selection", !disabled));
            ui.add_space(6.0);
            r_upgrade = Some(draw_action_btn(ui, "Upgrade all Applications", true));
        });

        if let Some(r) = r_install { if r.clicked() {
            spawn_task(DoWhat::Install, log.clone());
        }}
        if let Some(r) = r_uninstall { if r.clicked() {
            spawn_task(DoWhat::Uninstall, log.clone());
        }}
        if let Some(r) = r_update { if r.clicked() {
            spawn_task(DoWhat::Update, log.clone());
        }}
        if let Some(r) = r_clear { if r.clicked() {
            state().lock().unwrap().selected.clear();
        }}
        if let Some(r) = r_upgrade { if r.clicked() {
            let log = log.clone();
            std::thread::spawn(move || {
                let _ = commands::upgrade_all_apps_with_log(log);
            });
        }}
    }

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    // Search row
    {
        use egui::{Frame, Stroke, Color32, Margin, Vec2, TextStyle, Align2, StrokeKind, CursorIcon};

        let mut query = search_query().lock().unwrap();

        let draw_small_action_btn = |ui: &mut egui::Ui, label: &str| -> egui::Response {
            let min_size = Vec2::new(110.0, 28.0);
            let (rect, resp) = ui.allocate_at_least(min_size, egui::Sense::click());
            let visuals = ui.style().visuals.clone();
            let normal_bg = visuals.widgets.inactive.bg_fill;
            let hover_bg = Color32::WHITE;
            let bg_fill = if resp.hovered() { hover_bg } else { normal_bg };
            let stroke_col = if resp.hovered() { Color32::from_rgb(0,255,140) } else { Color32::from_gray(160) };

            ui.painter().rect(rect, 6.0, bg_fill, Stroke::new(1.5, stroke_col), StrokeKind::Middle);
            let font_id = TextStyle::Button.resolve(ui.style());
            let text_col = if resp.hovered() { Color32::BLACK } else { Color32::from_rgb(0,255,140) };
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_id, text_col);

            if resp.hovered() { ui.ctx().set_cursor_icon(CursorIcon::PointingHand); }
            resp
        };

        let mut resp_filter: Option<egui::Response> = None;
        let mut resp_reset: Option<egui::Response> = None;

        ui.horizontal(|ui| {
            ui.label("Search:");
            let frame = Frame::new()
                .stroke(Stroke::new(1.5, Color32::GREEN))
                .corner_radius(4.0)
                .inner_margin(Margin::symmetric(4, 2));
            frame.show(ui, |ui| {
                ui.set_min_width(300.0);
                ui.text_edit_singleline(&mut *query);
            });
            ui.add_space(6.0);
            resp_filter = Some(draw_small_action_btn(ui, "Filtrare"));
            ui.add_space(6.0);
            resp_reset = Some(draw_small_action_btn(ui, "Reset"));
        });

        if let Some(r) = resp_filter { if r.clicked() { /* apply filter if needed */ } }
        if let Some(r) = resp_reset { if r.clicked() { query.clear(); } }
    }

    ui.add_space(6.0);

    // Lista aplicatiilor filtrata
    let query = search_query().lock().unwrap().to_lowercase();
    let mut by_cat: BTreeMap<&'static str, Vec<&'static AppItem>> = BTreeMap::new();
    for item in APP_CATALOG.iter() {
        if query.is_empty() || item.name.to_lowercase().contains(&query) || item.desc.to_lowercase().contains(&query) {
            by_cat.entry(item.category).or_default().push(item);
        }
    }

    // === două coloane independente cu scroll separat ===
    let avail_h = ui.available_height().max(200.0); // asigură o înălțime rezonabilă
    let total_w = ui.available_width();
    let gap = 12.0;
    let left_w = (total_w * 0.50).max(320.0);
    let right_w = (total_w - left_w - gap).max(280.0);

    ui.horizontal(|ui_h| {
        // stânga: listă de aplicații (scroll independent)
        ui_h.allocate_ui_with_layout(egui::vec2(left_w, avail_h), egui::Layout::top_down(egui::Align::LEFT), |ui_left| {
            egui::ScrollArea::vertical()
                .id_salt("install_left_scroll")
                .auto_shrink([false, false])
                .max_height(avail_h)
                .show(ui_left, |ui_left| {
                    ui_left.set_min_width(left_w);

                    // conținutul listei (folosește by_cat calculat anterior)
                    for (category, items) in by_cat.clone().into_iter() {
                        ui_left.collapsing(category, |ui_cat| {
                            for app in items {
                                // folosește variabilă locală pentru checkbox; actualizează state la schimbare
                                let mut checked = state().lock().unwrap().selected.contains(app.winget_id);
                                let resp = ui_cat.checkbox(&mut checked, app.name).on_hover_text(app.desc);
                                if resp.changed() {
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

                    ui_left.add_space(6.0);
                });
        });

        ui_h.add_space(6.0);
        ui_h.separator();
        ui_h.add_space(6.0);

        // dreapta: Chrome panel (scroll independent)
        ui_h.allocate_ui_with_layout(egui::vec2(right_w, avail_h), egui::Layout::top_down(egui::Align::LEFT), |ui_right| {
            egui::ScrollArea::vertical()
                .id_salt("install_right_scroll")
                .auto_shrink([false, false])
                .max_height(avail_h)
                .show(ui_right, |ui_right| {
                    ui_right.set_min_width(right_w);

                    // afișează panelul Chrome în această coloană
                    show_chrome_panel(ui_right, log);

                    ui_right.add_space(6.0);
                });
        });
    });


}

fn spawn_task(what: DoWhat, log: Arc<Mutex<String>>) {
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

    if !commands::ensure_winget_ready(log.clone()) {
        log_line(&log, "ERROR: winget not ready; aborting task.");
        return;
    }

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
                p.current = idx + 1; // make progress human-friendly (1..total)
                p.current_name = name.clone();
            }

            match what {
                DoWhat::Install => {
                    if commands::winget_is_installed(id) {
                        log_line(&log, format!("ℹ {} already installed. Skipping.", name));
                    } else {
                        log_line(&log, format!("⬇ Installing {}...", name));
                        let _ = commands::winget_install(id, log.clone());
                    }
                }
                DoWhat::Uninstall => {
                    if commands::winget_is_installed(id) {
                        log_line(&log, format!("🗑 Uninstalling {}...", name));
                        let _ = commands::winget_uninstall(id, log.clone());
                    } else {
                        log_line(&log, format!("ℹ {} not found. Skipping.", name));
                    }
                }
                DoWhat::Update => {
                    if commands::winget_is_installed(id) {
                        log_line(&log, format!("⤴ Updating {}...", name));
                        let _ = commands::winget_upgrade(id, log.clone());
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
        state().lock().unwrap().selected.clear();
    });
}

//
// --- Google Chrome panel (dynamic catalog) ---
//

#[derive(Clone)]
struct ChromeExt {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    default_enabled: bool,
}

/// Lista inițială de extensii (deduplicată)
static INITIAL_CHROME_EXTS: &[ChromeExt] = &[
    ChromeExt { id: "bgnkhhnnamicmpeenaelnjfhikgbkllg", name: "Adguard Adblocker", desc: "AdGuard ad blocker effectively blocks ads on many websites.", default_enabled: true },
    ChromeExt { id: "cfnpidifppmenkapgihekkeednfoenal", name: "TrafficLight", desc: "Bitdefender TrafficLight adds a security layer while browsing.", default_enabled: true },
    ChromeExt { id: "khndhdhbebhaddchcgnalcjlaekbbeof", name: "Bitdefender Anti-tracker", desc: "Blocks trackers and allows managing tracking data per site.", default_enabled: true },
    ChromeExt { id: "fihnjjcciajhdojfnbdddfaoknhalnja", name: "I don't care about cookies", desc: "Tries to remove cookie consent popups on many sites.", default_enabled: false },
    ChromeExt { id: "oeopbcgkkoapgobdbedcemjljbihmemj", name: "Checker Plus for Gmail™", desc: "Get notifications, read or delete emails without opening Gmail.", default_enabled: false },
    ChromeExt { id: "lpcaedmchfhocbbapmcbpinfpgnhiddi", name: "Google Keep", desc: "Save to Google Keep with one click.", default_enabled: false },
    ChromeExt { id: "jldhpllghnbhlbpcmnajkpdmadaolakh", name: "Todoist for Chrome", desc: "Organize work and life with Todoist for Chrome.", default_enabled: false },
    ChromeExt { id: "ghgabhipcejejjmhhchfonmamedcbeod", name: "Click&Clean", desc: "Deletes typed URLs, Cache, Cookies and Browsing History with one click.", default_enabled: false },
    ChromeExt { id: "nlkaejimjacpillmajjnopmpbkbnocid", name: "YouTube NonStop", desc: "Remove the 'Video paused. Continue watching?' prompt.", default_enabled: false },
    ChromeExt { id: "pachckjkecffpdphbpmfolblodfkgbhl", name: "vidIQ Vision for YouTube", desc: "Uncover the secrets to success behind your favorite YouTube videos.", default_enabled: false },
];

static CHROME_EXT_CATALOG: OnceLock<Mutex<Vec<ChromeExt>>> = OnceLock::new();
fn chrome_ext_catalog() -> &'static Mutex<Vec<ChromeExt>> {
    CHROME_EXT_CATALOG.get_or_init(|| {
        let mut v = Vec::new();
        v.extend_from_slice(INITIAL_CHROME_EXTS);
        Mutex::new(v)
    })
}

/// State map: ext_id -> enabled
static CHROME_EXT_STATE_MAP: OnceLock<Mutex<HashMap<&'static str, bool>>> = OnceLock::new();
fn chrome_ext_state_map_init() -> &'static Mutex<HashMap<&'static str, bool>> {
    CHROME_EXT_STATE_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for ext in chrome_ext_catalog().lock().unwrap().iter() {
            m.insert(ext.id, ext.default_enabled);
        }
        Mutex::new(m)
    })
}

/// Add extension at runtime (id/name/desc MUST be 'static strings)
#[allow(dead_code)]
fn add_extension_to_catalog(ext: ChromeExt) {
    {
        let mut cat = chrome_ext_catalog().lock().unwrap();
        if !cat.iter().any(|e| e.id == ext.id) {
            cat.push(ext.clone());
        }
    }
    let mut state = chrome_ext_state_map_init().lock().unwrap();
    state.entry(ext.id).or_insert(ext.default_enabled);
}

// reuse constant
const CHROME_UPDATE_URL: &str = "https://clients2.google.com/service/update2/crx";

/// Detect Chrome by searching usual install locations
fn chrome_is_installed() -> bool {
    let candidates = [
        std::env::var("PROGRAMFILES").ok(),
        std::env::var("PROGRAMFILES(X86)").ok(),
        std::env::var("LOCALAPPDATA").ok(),
    ];
    for base in candidates.iter().flatten() {
        let mut p = PathBuf::from(base);
        p.push("Google");
        p.push("Chrome");
        p.push("Application");
        p.push("chrome.exe");
        if p.exists() {
            return true;
        }
    }
    false
}

/// Install Chrome using winget; returns (ok, message)
fn install_chrome_with_winget() -> (bool, String) {
    match Command::new("winget").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let res = Command::new("winget")
                .args(&[
                    "install",
                    "--id",
                    "Google.Chrome",
                    "-e",
                    "--accept-source-agreements",
                    "--accept-package-agreements",
                ])
                .status();

            match res {
                Ok(s) if s.success() => (true, "winget: install command returned success".to_string()),
                Ok(s) => (false, format!("winget returned non-zero status: {}", s)),
                Err(e) => (false, format!("failed to run winget install: {}", e)),
            }
        }
        _ => (false, "winget not found on PATH".to_string()),
    }
}

/// Add registry key HKCU\Software\Google\Chrome\Extensions\{extid}\update_url -> clients2...
fn add_chrome_external_extension_registry(ext_id: &str) -> (bool, String) {
    let key = format!(r#"HKCU\Software\Google\Chrome\Extensions\{}"#, ext_id);
    let args = ["add", &key, "/v", "update_url", "/t", "REG_SZ", "/d", CHROME_UPDATE_URL, "/f"];

    match Command::new("reg").args(&args).output() {
        Ok(out) => {
            if out.status.success() {
                (true, format!("Registry key added for extension {}", ext_id))
            } else {
                (false, format!("reg failed: {}", String::from_utf8_lossy(&out.stderr)))
            }
        }
        Err(e) => (false, format!("failed to run reg: {}", e)),
    }
}

/// --- Stylized button drawer (reusable) ------------------------------------
fn draw_action_btn(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    let min_size = egui::Vec2::new(220.0, 28.0);
    let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, resp) = ui.allocate_at_least(min_size, sense);

    let visuals = ui.style().visuals.clone();
    let normal_bg = visuals.widgets.inactive.bg_fill;
    let hover_bg = egui::Color32::from_rgb(0, 120, 215);
    let bg_fill = if resp.hovered() && enabled { hover_bg } else { normal_bg };

    ui.painter().rect_filled(rect, 6.0, bg_fill);
    ui.painter().rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(
            1.0,
            if resp.hovered() { egui::Color32::from_rgb(0,255,140) } else { egui::Color32::from_gray(160) }
        ),
        egui::StrokeKind::Middle
    );

    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let text_col = if resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_rgb(0, 255, 140) };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_id, text_col);

    if resp.hovered() && enabled {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}


pub fn show_chrome_panel(ui: &mut egui::Ui, log: &Arc<Mutex<String>>) {
    ui.separator();
    ui.vertical_centered(|ui| {
        let title = egui::RichText::new(
            "Google Chrome install & add most \n important extensions from the Chrome Web Store",
        )
        .size(20.0) // ajustează mărimea după nevoie (ex: 18.0, 22.0)
        .color(egui::Color32::from_rgb(0, 180, 0)) // verde
        .strong();
        ui.label(title);
    });


    egui::CollapsingHeader::new("Google Chrome (Install & Extensions)")
        .default_open(true)
        .show(ui, |ui| {
            // scurt text informativ
            ui.label("Verify Chrome installation + install extensions from the Chrome Web Store");

            // Afisare status + buton pe aceeasi linie
            let installed = chrome_is_installed();

            // Folosim un singur rând orizontal: label + spacer + buton
            ui.horizontal(|ui| {
                ui.label(format!("Chrome installation status: {}", if installed { "Yes" } else { "No" }));
                ui.add_space(8.0); // mică separare între text și buton

                // Forțăm min-height 0 pentru a nu introduce padding vertical implicit
                ui.set_min_height(0.0);

                let btn_label = if !installed { "Install Chrome (winget)" } else { "Reinstall/Repair Chrome (winget)" };
                let resp = draw_action_btn(ui, btn_label, true);
                if resp.clicked() {
                    let (ok, msg) = install_chrome_with_winget();
                    if let Ok(mut lg) = log.lock() {
                        if !installed {
                            lg.push_str(&format!("Installing Chrome: ok={} msg={}\n", ok, msg));
                        } else {
                            lg.push_str(&format!("Reinstall attempted: ok={} msg={}\n", ok, msg));
                        }
                    }
                }
            });

            // mic separator sub rând
            let (rect, _resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
            let y = rect.center().y;
            ui.painter().line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, egui::Color32::from_gray(140)),
            );

            // Catalog extensii: checkbox-uri
            let state_map_mutex = chrome_ext_state_map_init();
            let catalog = chrome_ext_catalog().lock().unwrap().clone();
            {
                let mut state_map = state_map_mutex.lock().unwrap();
                for ext in catalog.iter() {
                    let cur = *state_map.get(ext.id).unwrap_or(&ext.default_enabled);
                    let mut local = cur;
                    let resp = ui.checkbox(&mut local, ext.name).on_hover_text(ext.desc);
                    if resp.changed() {
                        state_map.insert(ext.id, local);
                    }
                }
            }

            ui.add_space(6.0);

            // Butonul final "Install selected extensions"
            ui.horizontal_centered(|ui| {
                let resp_install = draw_action_btn(ui, "Install selected extensions", true);
                if resp_install.clicked() {
                    let state_map = state_map_mutex.lock().unwrap();
                    let selected_ids: Vec<&str> = state_map.iter().filter_map(|(id, enabled)| if *enabled { Some(*id) } else { None }).collect();

                    if selected_ids.is_empty() {
                        if let Ok(mut lg) = log.lock() { lg.push_str("No extensions selected.\n"); }
                    } else if !chrome_is_installed() {
                        if let Ok(mut lg) = log.lock() { lg.push_str("Chrome is not installed. Install Chrome before adding extensions.\n"); }
                    } else {
                        for id in selected_ids {
                            let (ok, msg) = add_chrome_external_extension_registry(id);
                            if let Ok(mut lg) = log.lock() {
                                let name = chrome_ext_catalog().lock().unwrap().iter().find(|e| e.id == id).map(|e| e.name).unwrap_or(id);
                                lg.push_str(&format!("{} ({}) -> ok={} msg={}\n", name, id, ok, msg));
                            }
                        }
                        if let Ok(mut lg) = log.lock() {
                            lg.push_str("Extension installation: complete (Chrome will load the extensions the next time it starts).\n");
                        }
                    }
                }
            });
        });
}
