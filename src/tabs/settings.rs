use std::sync::{Arc, Mutex};
use crate::commands;

#[allow(dead_code)]
pub fn show_settings(ui: &mut egui::Ui, log_output: &Arc<Mutex<String>>) {
    ui.heading("⚙ Settings");

    ui.separator();

    ui.label("Here you can configure application settings (theme, preferences, etc).");

    // if ui.button("🔄 Reset Output").clicked() {
    //     *log_output.lock().unwrap() = String::new();
    // }

    if ui.button("Switch to Light Mode").clicked() {
        *log_output.lock().unwrap() = commands::change_theme("light");
    }
    if ui.button("Switch to Dark Mode").clicked() {
        *log_output.lock().unwrap() = commands::change_theme("dark");
    }
    if ui.button("Open Display Settings").clicked() {
        *log_output.lock().unwrap() = commands::quick_access_settings("display");
    }

    if ui.button("ℹ About").clicked() {
        let mut log = log_output.lock().unwrap();
        *log = "Eoliann Windows Tools v1.0.3\nCreated by Eoliann Dev\nQuick tools for Windows administration.\nWebsite: https://github.com/eoliann".to_string();
    }
}

// Example usage: Call this function from your main UI code
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    #[test]
    fn test_show_settings_usage() {
        let log_output = Arc::new(Mutex::new(String::new()));
        // Cannot directly instantiate egui::Ui, so just test log_output logic
        {
            *log_output.lock().unwrap() = String::from("test");
            assert_eq!(&*log_output.lock().unwrap(), "test");
        }
    }
}
