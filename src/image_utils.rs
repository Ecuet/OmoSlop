use std::{fs::{self, File}, path::{Path, PathBuf}};

use slint::{Image, Timer, Weak};

use crate::{app::GifWindow, Config, config, ffmpeg_utils};

pub enum ImageType {
    Gif, 
    Static,
    None
}

pub fn get_directory_length(directory: &str) -> Result<usize, std::io::Error> {
    let path = fs::read_dir(directory)?;
    let length = path
    .filter_map(|entry| entry.ok())
    .filter(|entry| 
        entry.file_name().to_string_lossy().starts_with("frame_"))
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

        let final_delay = if delay == 0 {1/10} else {delay};

        delays.push(final_delay);
    }
    Ok(delays)
}

pub fn open_file_chooser() -> Option<PathBuf> {
    let image = rfd::FileDialog::new()
    .add_filter("Image", &["png","gif","jpeg","jpg"])
    .pick_file();

    image
}

pub fn get_image_type(selected_file: &str) -> ImageType {
    let file_type = selected_file.split(".").last().unwrap_or("");

    match file_type {
        "gif" => ImageType::Gif,
        "png" | "jpg" | "jpeg" => ImageType::Static,
        _ => ImageType::None
    }
}

pub fn change_gif_scale(scale: f32, gif_ui: &GifWindow) {
    let gif_x: f32 = 100.0;
    let gif_y: f32 = 100.0;

    gif_ui.set_ImageXWidth((gif_x * scale).into());
    gif_ui.set_ImageXHeight(gif_y);
}

pub fn get_file_name(path: &Path) -> Result<String, Box<dyn std::error::Error>>{
    Ok(path.file_stem().ok_or("none")?.to_string_lossy().to_string())
}