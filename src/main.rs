#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, RwLock};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(RwLock::new(Config::new()));

    app::App::new(config)?.run()?;

    Ok(())
}

