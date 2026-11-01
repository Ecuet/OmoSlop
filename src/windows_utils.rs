use raw_window_handle::RawWindowHandle;
use slint::{ ComponentHandle, Window };

use crate::app::GifWindow;

use i_slint_backend_winit::{
	WinitWindowAccessor,
	winit::{ platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle },
};
use winapi::um::winuser::{
	GWL_EXSTYLE,
	GetWindowLongPtrW,
	SetWindowLongPtrW,
	WS_EX_LAYERED,
	WS_EX_TRANSPARENT,
};
pub fn toggle_clickability(gif_ui: &GifWindow, state: bool) {
	let weak_gif_ui = gif_ui.as_weak();
	gif_ui.window().on_winit_window_event(move |_, _| {
		if let Some(gif_ui) = weak_gif_ui.upgrade() {
			gif_ui.window().with_winit_window(move |winit| {
				winit.set_skip_taskbar(true);
				winit.set_transparent(true);

				match winit.window_handle().unwrap().as_raw() {
					RawWindowHandle::Win32(win32) => {
						let hwnd = win32.hwnd.get() as winapi::shared::windef::HWND;

						unsafe {
							let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
							if state {
								SetWindowLongPtrW(hwnd, GWL_EXSTYLE, WS_EX_TRANSPARENT as isize);
							} else {
								SetWindowLongPtrW(
									hwnd,
									GWL_EXSTYLE,
									(WS_EX_TRANSPARENT as isize) | (WS_EX_LAYERED as isize)
								);
							}
						}
					}
					_ => {}
				}
			});
		}
        i_slint_backend_winit::EventResult::Propagate
	});
}
