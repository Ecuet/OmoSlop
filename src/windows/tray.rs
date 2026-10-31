use std::sync::{ Arc, RwLock };

use slint::ComponentHandle;

use crate::{
	app::{ GifWindow, MainWindow, MainWindowTray, Overlays },
	config::Config,
	overlays_utils,
};

pub fn create(
	main_ui: &MainWindow,
	config: Arc<RwLock<Config>>,
	overlays: &Overlays
) -> Result<MainWindowTray, Box<dyn std::error::Error>> {
	let tray = MainWindowTray::new().unwrap();

	let main_ui_clone = main_ui.as_weak();
	tray.on_show(move || {
		if let Some(main_ui) = main_ui_clone.upgrade() {
			let _ = main_ui.show();
		}
	});
	let file_config = config.clone();
	let overlays = overlays.clone();
	tray.on_quit(move || {
		overlays_utils::save_all_overlays_position(&mut config.write().unwrap(), &overlays);
		file_config.write().unwrap().save();

		slint::quit_event_loop().unwrap();
	});

	Ok(tray)
}
