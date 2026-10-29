use std::{collections::HashMap, fs, io::{Read, Write}};

use crate::image_utils::ImageType;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GifData {
    pub name: String,

    pub selected_file_path: String,
    pub window_scale: f32,

    pub locked: bool,

    pub overlay_x: i32,
    pub overlay_y: i32
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub gifs: HashMap<String, GifData>
}

impl Config {
    pub fn new() -> Self {
        if let Ok(data) = Self::load() {
            return data;
        }

        Self { 
            gifs: HashMap::new()
        }
    }

    pub fn update_selected_file_path(&mut self, gif_name: &str, path: &str) {
        
        //self.selected_file_path = path.to_string();

        self.save();
    }

    pub fn save(&self) {
        let file = match fs::File::create("config.json") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("{}",e);
                return;
            }   
        };

        if let Err(err) = serde_json::to_writer_pretty(file, &self) {
            eprintln!("{}", err);
        }
    }

    fn load() -> Result<Config, Box<dyn std::error::Error>> {
        let file = fs::read_to_string("config.json")?;
        let config: Config = serde_json::from_str(&file)?;

        Ok(config)
    }

}