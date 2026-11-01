use std::sync::{ Arc, RwLock };

use slint::{ ComponentHandle, PhysicalPosition };

use crate::{
	app::{ GifWindow, OverlayWindow, Overlays },
	config::{ Config, GifData, OverlayData, Size },
	image_utils,
	overlay::{ AnimatedOverlay, Overlay, OverlayKind },
	windows::gif_window,
	windows_utils,
};

pub fn create_overlay_window() -> Result<GifWindow, Box<dyn std::error::Error>> {
	let gif_window = gif_window::create()?;

	gif_window.show()?;

	Ok(gif_window)
}
pub fn create_overlay(
	gif_data: &GifData,
	overlays: &Overlays,
	index: Option<usize>
) -> Result<(), Box<dyn std::error::Error>> {
	let gif_window = create_overlay_window()?;

	if let Some(index) = index {
		let data = gif_data.overlays.get(index).ok_or(format!("Cannot find OverlayData by {}", index))?;

		gif_window.window().set_position(PhysicalPosition::new(data.overlay_x, data.overlay_y));
	}

	{
		let mut overlay_write = overlays.write().unwrap();

		let overlay_vec = overlay_write.entry(gif_data.name.to_string()).or_insert_with(|| {
			let gif_window = Arc::new(RwLock::new(Vec::new()));

			let mut loaded_overlay = OverlayKind::Animated(AnimatedOverlay::new(gif_window.clone()));

			loaded_overlay.load_file(&gif_data.selected_file_path, false).unwrap();
			OverlayWindow {
				gif_windows: gif_window,
				overlay: loaded_overlay,
			}
		});

		overlay_vec.overlay.add_gif_window(&gif_window);

		windows_utils::toggle_clickability(&gif_window, !gif_data.locked);

		let mut write = overlay_vec.gif_windows.write().unwrap();
		write.push(gif_window);
	}

	change_overlay_scale(
		&overlays,
		gif_data.name.to_string(),
		gif_data.window_scale,
		&gif_data.image_size
	)?;

	Ok(())
}
pub fn create_all_overlays(
	gif_data: &GifData,
	overlays: &Overlays
) -> Result<(), Box<dyn std::error::Error>> {
	let size = gif_data.overlays.len();

	for i in 0..size {
		create_overlay(&gif_data, &overlays, Some(i))?;
	}

	Ok(())
}
pub fn create_overlay_with_saving(
	config: &mut Config,
	name: String,
	overlays: &Overlays
) -> Result<(), Box<dyn std::error::Error>> {
	let gif_data = config.gifs.get(&name).ok_or(format!("Couldnt get config for {}", name))?;

	create_overlay(gif_data, overlays, None)?;

	let config_gif_data = config.gifs.get_mut(&name).ok_or(format!("No config found for {}", name))?;
	config_gif_data.overlays.push(OverlayData {
		overlay_x: 0,
		overlay_y: 0,
	});

	config.save();

	Ok(())
}
pub fn remove_overlay(
	config: &mut Config,
	name: String,
	overlays: &Overlays
) -> Result<(), Box<dyn std::error::Error>> {
	let gif_data = config.gifs.get_mut(&name).ok_or(format!("Couldnt get config for {}", name))?;

	let mut write = overlays.write().unwrap();

	let overlay = write.get_mut(&name).ok_or("Not found")?;
	if let Some(removed_window) = overlay.gif_windows.write().unwrap().pop() {
		gif_data.overlays.pop();

		overlay.overlay.remove_gif_window(&removed_window);

		removed_window.set_closing_allowed(true);

		removed_window.invoke_Close();
	}

	config.save();

	Ok(())
}
pub fn remove_all_overlays(config: &mut Config, name: String, overlays: &Overlays) -> Result<(), Box<dyn std::error::Error>>  {
	let gif_data = config.gifs.get_mut(&name).ok_or(format!("Couldnt get config for {}", name))?;

	let mut write = overlays.write().unwrap();

	let Some(overlay) = write.get_mut(&name) else {return Ok(())};

	let mut gif_windows = overlay.gif_windows.write().unwrap();
	
	for i in gif_windows.iter() {
		overlay.overlay.remove_gif_window(&i);

		i.set_closing_allowed(true);

		i.invoke_Close();
	}

	gif_data.overlays.clear();

	gif_windows.clear();

	config.save();

	Ok(())
}

pub fn save_all_overlays_position(config: &mut Config, overlays: &Overlays) {
	let overlays_read = overlays.read().unwrap();
	for (name, vec) in overlays_read.iter() {
		if let Some(gif_data) = config.gifs.get_mut(name) {
			for (i, window) in vec.gif_windows.read().unwrap().iter().enumerate() {
				let window = window.window();

				if let Some(gif_data_vec) = gif_data.overlays.get_mut(i) {
					gif_data_vec.overlay_x = window.position().x;
					gif_data_vec.overlay_y = window.position().y;
				}
			}
		}
	}
}

pub fn change_overlay_scale(
	overlays: &Overlays,
	name: String,
	scale: f32,
	original_size: &Size
) -> Result<(), Box<dyn std::error::Error>> {
	let overlay_write = overlays.write().unwrap();

	if let Some(vec) = overlay_write.get(&name) {
		for i in vec.gif_windows.read().unwrap().iter() {
			image_utils::change_gif_scale(scale, &original_size, &i);
		}
	}

	Ok(())
}

pub fn change_overlay_scale_with_saving(
	config: &mut Config,
	overlays: &Overlays,
	name: String,
	scale: f32,
	original_size: &Size
) -> Result<(), Box<dyn std::error::Error>> {
	config.gifs.get_mut(&name).ok_or("No config found")?.window_scale = scale;

	change_overlay_scale(&overlays, name.to_string(), scale, &original_size)?;

	Ok(())
}
