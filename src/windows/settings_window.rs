use crate::app::SettingsWindow;


pub fn create() -> Result<SettingsWindow, Box<dyn std::error::Error>> {
    let settings_window = SettingsWindow::new()?;
    

    Ok(settings_window)
}