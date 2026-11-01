
use std::fs;
use std::process::Command;

use crate::image_utils;


pub const OUTPUT_PATH: &str = "gif/output";

fn prepare_output_folder(path: &str) -> Result<(), std::io::Error> {
    let _ = fs::remove_dir_all(&path);

    fs::create_dir_all(&path)?;

    Ok(())
}

fn create_default_folders() -> Result<(), std::io::Error> {
    fs::create_dir_all("gif")?;
    fs::create_dir_all("gif/input")?;
    fs::create_dir_all("gif/output")?;

    Ok(())
}

pub fn get_output_path(gif_name: &str) -> String {
    format!("{OUTPUT_PATH}/{gif_name}")
}


pub fn convert_gif(file_path: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(file_path);
    let gif_name = image_utils::get_file_name(path)?;

    let formatted_output_path = format!("gif/output/{gif_name}");

    if !force & fs::exists(&formatted_output_path).unwrap_or(false) {
        if image_utils::get_directory_length(&formatted_output_path).unwrap_or(0) != 0 {
            return Ok(())
        }
        
    }

    create_default_folders()?;
    prepare_output_folder(&formatted_output_path)?;

    let output = format!("{}/frame_%04d.png", get_output_path(&gif_name));

    // let _status = Command::new("./ffmpeg.exe").args([
    //     "-i",
    //     file_path,
    //     &output
    // ]).status()?;

    let context = ez_ffmpeg::FfmpegContext::builder()
    .input(file_path)
    .output(&*output)
    .build()?;

    let result = ez_ffmpeg::FfmpegScheduler::new(context)
    .start()?
    .wait()?;

    Ok(())
}