use crate::app::GifWindow;
use slint::ComponentHandle;

use i_slint_backend_winit::{WinitWindowAccessor, winit::{platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle}};

pub fn create() -> Result<GifWindow, Box<dyn std::error::Error>> {
    let gif_ui: GifWindow = GifWindow::new()?;

    let gif_ui_clone = gif_ui.as_weak();
    gif_ui.on_MoveWindow(move |x, y| {
        let pin_win_clone = gif_ui_clone.unwrap();
        let logical_pos = pin_win_clone.window().position().to_logical(pin_win_clone.window().scale_factor());
        pin_win_clone.window().set_position(slint::LogicalPosition::new(logical_pos.x + x, logical_pos.y + y));
    });
    let gif_ui_clone = gif_ui.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(gif_ui) = gif_ui_clone.upgrade() {
            gif_ui.window().with_winit_window(move |winit| {
                winit.set_skip_taskbar(true);
                winit.set_transparent(true);
                
            });
        }
    }).unwrap();

    

    Ok(gif_ui)
}