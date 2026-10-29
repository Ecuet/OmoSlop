use core::time;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use slint::{Image, ModelRc, VecModel, Weak};

use crate::app::GifWindow;
use crate::{ffmpeg_utils, image_utils};
use crate::image_utils::get_directory_length;

pub enum OverlayKind {
    Animated(AnimatedOverlay)
}

impl Overlay for OverlayKind {
    fn load_file(&mut self,path: &str, force:bool) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            OverlayKind::Animated(o) => o.load_file(path, force),
        
        }
    }
  
}

pub trait Overlay {
    fn load_file(&mut self,path: &str, force:bool) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct AnimatedOverlay {
    frames: Vec<Image>,

    max_frames: usize,
    delays: Vec<u32>,

    is_destroyed: Arc<AtomicBool>,

    ui_handle: Weak<GifWindow>
}

impl Overlay for AnimatedOverlay {
    fn load_file(&mut self, path: &str, force:bool) -> Result<(), Box<dyn std::error::Error>> {
        ffmpeg_utils::convert_gif(path, force)?;

        self.load_all_images(path)?;
        self.start_loop();

        Ok(())
    }
}

impl Drop for AnimatedOverlay {
    fn drop(&mut self) {
        self.is_destroyed.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl AnimatedOverlay {    
    pub fn new(gif_window: Weak<GifWindow>) -> Self {
        Self {
            frames: Vec::new(),

            max_frames: 1,
            delays: Vec::new(),
            is_destroyed: Arc::new(AtomicBool::new(false)),

            ui_handle: gif_window
        }
    }


    pub fn load_all_images(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = std::path::Path::new(file_path);
        let gif_name = file_path.file_stem().unwrap().to_string_lossy().to_string(); 
        let path = ffmpeg_utils::get_output_path(&gif_name);

        let folder_length = get_directory_length(&path)?;
        for i in 1..=folder_length {
            let path = format!("{path}/frame_{i:04}.png");
            if let Ok(img) = slint::Image::load_from_path(std::path::Path::new(&path)) {
                self.frames.push(img);
            }
            else {
                return Err(Box::from("failed to load images"))
            }
        }

        self.delays = image_utils::get_gif_delays(&file_path).unwrap_or(vec![]); 
        self.max_frames = folder_length;

        Ok(())
    }

    pub fn start_loop(&self) {
        let frames = self.frames.clone();
        let ui_handle = self.ui_handle.clone();
        let mut current_frame: usize = 0;

        let len = frames.len();
        let delays= self.delays.clone();

        if let Some(ui) = self.ui_handle.upgrade() { 
            ui.set_frames(ModelRc::new(VecModel::from(frames)));
        }

        let destroyed_clone = self.is_destroyed.clone();

        thread::spawn(move || {
            while !destroyed_clone.load(std::sync::atomic::Ordering::Relaxed) {
                let handle = ui_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = handle.upgrade() {
                        ui.set_currentImageIndex(current_frame as i32); 
                    }
                });
                

                current_frame = (current_frame + 1) % len;

                let delay = delays.get(current_frame).copied().unwrap_or(1000/10);
    
                thread::sleep(time::Duration::from_millis(delay as u64));
            }
        });

    
    }

}
