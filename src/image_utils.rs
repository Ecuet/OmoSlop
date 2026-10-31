use std::{ collections::HashMap, fs::{ self, File }, path::{ Path, PathBuf }, sync::{OnceLock, RwLock} };

use image::{GenericImageView, Rgba};
use rfd::FileHandle;
use slint::{Image, SharedPixelBuffer};

use crate::{ app::GifWindow, config::Size, ffmpeg_utils };

pub fn get_directory_length(directory: &str) -> Result<usize, std::io::Error> {
	let path = fs::read_dir(directory)?;
	let length = path
		.filter_map(|entry| entry.ok())
		.filter(|entry| entry.file_name().to_string_lossy().starts_with("frame_"))
		.count();

	Ok(length)
}

pub fn get_gif_delays(path: &Path) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
	let file = File::open(path)?;

	let options = gif::DecodeOptions::default();

	let mut decoder = options.read_info(file)?;
	let mut delays: Vec<u32> = Vec::new();

	while let Some(frame) = decoder.read_next_frame()? {
		let delay: u32 = u32::from(frame.delay) * 10;

		let final_delay = if delay == 0 { 1 / 10 } else { delay };

		delays.push(final_delay);
	}
	Ok(delays)
}

pub async fn open_file_chooser() -> Option<PathBuf> {
	let image = rfd::AsyncFileDialog
		::new()
		.add_filter("Image", &["png", "gif", "jpeg", "jpg"])
		.pick_file()
		.await;

	if let Some(image) = image {
		return Some(image.path().to_path_buf())
	}
	else {
		return None
	}
}

pub fn get_image_size(path: &Path) -> Result<Size, Box<dyn std::error::Error>> {
	let dims = image::image_dimensions(path)?;

	Ok(Size {
		width: dims.0,
		height: dims.1
	})
}

pub fn change_gif_scale(scale: f32, original_size: &Size, gif_ui: &GifWindow) {
	let gif_x: f32 = original_size.width as f32;
	let gif_y: f32 = original_size.height as f32;

    let new_x = (gif_x * scale).into();
    let new_y = (gif_y * scale).into();

	gif_ui.set_ImageXWidth(new_x);
	gif_ui.set_ImageXHeight(new_y);
}

pub fn get_file_name(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
	Ok(
		path
			.file_stem()
			.ok_or(format!("No file path for {}", path.to_string_lossy().to_string()))?
			.to_string_lossy()
			.to_string()
	)
}

pub fn get_rendered_frames(name: String) -> Result<Vec<Image>, Box<dyn std::error::Error>> {
	let path = ffmpeg_utils::get_output_path(&name);

	let mut vec = Vec::new();

	let folder_length = get_directory_length(&path)?;
	for i in 1..=folder_length {
		let path = format!("{path}/frame_{i:04}.png");
		if let Ok(img) = slint::Image::load_from_path(std::path::Path::new(&path)) {
			vec.push(img);
		}
		else {
			return Err(Box::from("failed to load images"))
		}
	}

	Ok(vec)
}