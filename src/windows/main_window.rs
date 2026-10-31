use notify_rust::Notification;
use slint::ComponentHandle;

use crate::{ app::MainWindow, ignore_err };

pub fn create() -> Result<MainWindow, Box<dyn std::error::Error>> {
	let main_ui: MainWindow = MainWindow::new()?;
	//    setup_window!(main_ui);

	let main_ui_clone = main_ui.as_weak();
	main_ui.window().on_close_requested(move || {
		if let Some(main_ui) = main_ui_clone.upgrade() {
			let _ = main_ui.hide();

			if !main_ui.get_ClosedNotificationShown() {
				ignore_err!(
					Notification::new()
						.summary("OmoSlop")
						.body("You can open main window again in tray.")
						.show()
				);

                main_ui.set_ClosedNotificationShown(true);
			}
		}
		return slint::CloseRequestResponse::KeepWindowShown;
	});

	Ok(main_ui)
}
