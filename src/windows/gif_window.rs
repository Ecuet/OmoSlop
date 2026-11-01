use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::GifWindow;
use i_slint_backend_winit::{WinitWindowAccessor, winit::platform::windows::WindowExtWindows};
use raw_window_handle::RawWindowHandle;
use slint::ComponentHandle;



pub fn create() -> Result<GifWindow, Box<dyn std::error::Error>> {
    let gif_ui: GifWindow = GifWindow::new()?;

    let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    gif_ui.set_id(id as i32);

    gif_ui.show()?;

    let gif_ui_clone = gif_ui.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(gif_ui) = gif_ui_clone.upgrade() {
            gif_ui.window().with_winit_window(move |winit| {
                winit.set_skip_taskbar(true);
                winit.set_transparent(true);
            });
        }
    }).unwrap();

    let gif_ui_weak = gif_ui.as_weak();
    gif_ui.window().on_close_requested(move || {
        if let Some(gif_ui) = gif_ui_weak.upgrade() {
            if gif_ui.get_closing_allowed() {
                return slint::CloseRequestResponse::HideWindow
            }
            return slint::CloseRequestResponse::KeepWindowShown
        }
        return slint::CloseRequestResponse::HideWindow
    });

    let gif_ui_clone = gif_ui.as_weak();
    gif_ui.on_MoveWindow(move |x, y| {
        let pin_win_clone = gif_ui_clone.unwrap();
        let logical_pos = pin_win_clone.window().position().to_logical(pin_win_clone.window().scale_factor());
        pin_win_clone.window().set_position(slint::LogicalPosition::new(logical_pos.x + x, logical_pos.y + y));
    });
    

    Ok(gif_ui)
}