#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, os, sync::{Arc, RwLock}};

use rfd::MessageDialogResult;

use crate::config::Config;

mod ffmpeg_utils;
mod image_utils;
mod config;
mod app;
mod overlay;
mod windows;
mod windows_utils;
mod overlays_utils;

#[macro_export]
macro_rules! ignore_err {
    ($expr:expr) => {
        if let Err(e) = $expr {
            eprintln!("Error: {}", e);
        }
    };
}

fn check_for_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    let found = fs::read_dir("./")?.filter_map(|e| e.ok()).find(|e| e.file_name() == "ffmpeg.exe");

    if let None = found {
        return Err(Box::from("FFmpeg not found".to_string()))
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = check_for_ffmpeg() {
        let result = rfd::MessageDialog::new()
        .set_title("Failed to run the app")
        .set_description("FFmpeg not found in the current directory, do you wish to download it?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();

        if let MessageDialogResult::Yes = result {
            ignore_err!(webbrowser::open("https://www.ffmpeg.org/download.html"));
        }

        return Ok(())
    }

    let config = Arc::new(RwLock::new(Config::new()));

    app::App::new(config)?.run()?;

    Ok(())
}

