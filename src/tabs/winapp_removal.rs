use crate::commands::remove_app;

pub fn show_winapp_removal(
    ui: &mut egui::Ui,
    log_output: &mut String,
    show_popup: &mut bool,
    popup_message: &mut String,
) {
    ui.heading("🗑 Windows App Removal");
    ui.label("Select preinstalled apps you want to remove (UWP).");

    ui.separator();

    // ===== ⚠ Bulk Removal =====
    ui.group(|ui| {
        ui.heading("⚠ Bulk Removal");
        if ui.button("Remove All Listed Apps").clicked() {
            *log_output = "⚠ Are you sure you want to remove ALL listed apps?".to_string();
            *show_popup = true;
            *popup_message = "⚠ Confirm bulk removal of all listed apps?".to_string();
        }
        if ui.button("Force Remove All").clicked() {
            *log_output = "⚠ Are you sure you want to FORCE remove ALL listed apps?".to_string();
            *show_popup = true;
            *popup_message = "⚠ Confirm FORCE bulk removal of all listed apps?".to_string();
        }
    });

    ui.separator();

    // ===== Example Section =====
    ui.group(|ui| {
        ui.heading("📧 Communication");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Outlook for Windows").clicked() {
                *log_output = remove_app("Microsoft.OutlookForWindows");
            }
            if ui.button("Skype").clicked() {
                *log_output = remove_app("Microsoft.SkypeApp");
            }
            if ui.button("Teams").clicked() {
                *log_output = remove_app("MSTeams");
            }
        });
    });

    ui.separator();

    // ===== 📧 Communication =====
    ui.group(|ui| {
        ui.heading("📧 Communication");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Outlook for Windows").clicked() {
                *log_output = remove_app("Microsoft.OutlookForWindows");
            }
            if ui.button("Skype").clicked() {
                *log_output = remove_app("Microsoft.SkypeApp");
            }
            if ui.button("Teams").clicked() {
                *log_output = remove_app("MSTeams");
            }
            if ui.button("GroupMe").clicked() {
                *log_output = remove_app("Microsoft.GroupMe10");
            }
            if ui.button("To-Do").clicked() {
                *log_output = remove_app("Microsoft.Todos");
            }
            if ui.button("Your Phone").clicked() {
                *log_output = remove_app("Microsoft.YourPhone");
            }
            if ui.button("CommsPhone").clicked() {
                *log_output = remove_app("Microsoft.CommsPhone");
            }
            if ui.button("Messaging").clicked() {
                *log_output = remove_app("Microsoft.Messaging");
            }
            if ui.button("Mail & Calendar").clicked() {
                *log_output = remove_app("microsoft.windowscommunicationsapps");
            }
        });
    });

    ui.separator();

    // ===== 🎥 Media & Creativity =====
    ui.group(|ui| {
        ui.heading("🎥 Media & Creativity");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Clipchamp").clicked() {
                *log_output = remove_app("Clipchamp.Clipchamp");
            }
            if ui.button("Camera").clicked() {
                *log_output = remove_app("Microsoft.WindowsCamera");
            }
            if ui.button("MS Paint").clicked() {
                *log_output = remove_app("Microsoft.MSPaint");
            }
            if ui.button("3D Builder").clicked() {
                *log_output = remove_app("Microsoft.3DBuilder");
            }
            if ui.button("3D Viewer").clicked() {
                *log_output = remove_app("Microsoft.Microsoft3DViewer");
            }
            if ui.button("Print 3D").clicked() {
                *log_output = remove_app("Microsoft.Print3D");
            }
            if ui.button("Mixed Reality Portal").clicked() {
                *log_output = remove_app("Microsoft.MixedReality.Portal");
            }
        });
    });

    ui.separator();

    // ===== 📊 Microsoft Apps =====
    ui.group(|ui| {
        ui.heading("📊 Microsoft Apps");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Office Hub").clicked() {
                *log_output = remove_app("Microsoft.MicrosoftOfficeHub");
            }
            if ui.button("OneNote").clicked() {
                *log_output = remove_app("Microsoft.Office.OneNote");
            }
            if ui.button("Sway").clicked() {
                *log_output = remove_app("Microsoft.Office.Sway");
            }
            if ui.button("Sticky Notes").clicked() {
                *log_output = remove_app("Microsoft.MicrosoftStickyNotes");
            }
            if ui.button("Family Safety").clicked() {
                *log_output = remove_app("MicrosoftCorporationII.MicrosoftFamily");
            }
        });
    });

    ui.separator();

    // ===== 🌐 Bing Apps =====
    ui.group(|ui| {
        ui.heading("🌐 Bing Apps");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Bing Weather").clicked() {
                *log_output = remove_app("Microsoft.BingWeather");
            }
            if ui.button("Bing Sports").clicked() {
                *log_output = remove_app("Microsoft.BingSports");
            }
            if ui.button("Bing Finance").clicked() {
                *log_output = remove_app("Microsoft.BingFinance");
            }
            if ui.button("Bing News").clicked() {
                *log_output = remove_app("Microsoft.BingNews");
            }
        });
    });

    ui.separator();

    // ===== 🎮 Games =====
    ui.group(|ui| {
        ui.heading("🎮 Games");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Solitaire Collection").clicked() {
                *log_output = remove_app("Microsoft.MicrosoftSolitaireCollection");
            }
            if ui.button("Minecraft for Windows").clicked() {
                *log_output = remove_app("Microsoft.MinecraftUWP");
            }
        });
    });

    ui.separator();

    // ===== ⚙ System & Misc =====
    ui.group(|ui| {
        ui.heading("⚙ System & Misc");
        ui.horizontal_wrapped(|ui| {
            if ui.button("People").clicked() {
                *log_output = remove_app("Microsoft.People");
            }
            if ui.button("Maps").clicked() {
                *log_output = remove_app("Microsoft.WindowsMaps");
            }
            if ui.button("Wallet").clicked() {
                *log_output = remove_app("Microsoft.Wallet");
            }
            if ui.button("Get Started").clicked() {
                *log_output = remove_app("Microsoft.Getstarted");
            }
            if ui.button("Feedback Hub").clicked() {
                *log_output = remove_app("Microsoft.WindowsFeedbackHub");
            }
            if ui.button("Alarms & Clock").clicked() {
                *log_output = remove_app("Microsoft.WindowsAlarms");
            }
            if ui.button("OneConnect").clicked() {
                *log_output = remove_app("Microsoft.OneConnect");
            }
            if ui.button("Windows Phone").clicked() {
                *log_output = remove_app("Microsoft.WindowsPhone");
            }
            if ui.button("Voice Recorder").clicked() {
                *log_output = remove_app("Microsoft.WindowsSoundRecorder");
            }
        });
    });
}
