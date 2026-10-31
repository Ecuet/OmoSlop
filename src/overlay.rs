use core::time;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use std::thread;

use slint::{ComponentHandle, Image, ModelRc, VecModel, Weak};

use crate::app::{GifWindow, GifWindows};
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
    fn add_gif_window(&mut self, gif_window: &GifWindow) { 
        match self {
            OverlayKind::Animated(o) => o.add_gif_window(&gif_window),
        
        }
    }
    fn remove_gif_window(&mut self, gif_window: &GifWindow) { 
        match self {
            OverlayKind::Animated(o) => o.remove_gif_window(&gif_window),
        
        }
    }
}

pub trait Overlay {
    fn load_file(&mut self,path: &str, force:bool) -> Result<(), Box<dyn std::error::Error>>;

    fn add_gif_window(&mut self, gif_window: &GifWindow);

    fn remove_gif_window(&mut self, gif_window: &GifWindow);
}

pub struct AnimatedOverlay {
    frames: Vec<Image>,

    max_frames: usize,
    delays: Vec<u32>,

    is_destroyed: Arc<AtomicBool>,

    weak_gifs: Arc<RwLock<Vec<Weak<GifWindow>>>>,

    gif_window: GifWindows
}

impl Overlay for AnimatedOverlay {
    fn load_file(&mut self, path: &str, force:bool) -> Result<(), Box<dyn std::error::Error>> {
        ffmpeg_utils::convert_gif(path, force)?;

        self.load_all_images(path)?;
        self.start_loop();

        Ok(())
    }
    fn add_gif_window(&mut self, gif_window: &GifWindow) {
        //let gif_windows = self.gif_window.read().unwrap();
        let frames = self.frames.clone();

        //let weak: Vec<Weak<GifWindow>> = gif_windows.iter().map(|i| i.as_weak()).collect();

        self.weak_gifs.write().unwrap().push(gif_window.as_weak()); // = weak; 

        gif_window.set_frames(ModelRc::new(VecModel::from(frames.clone())));
    }

    fn remove_gif_window(&mut self, gif_window: &GifWindow) {
        let mut write = self.weak_gifs.write().unwrap();
        let find = write.iter().position(|x| x.upgrade().unwrap().get_id() == gif_window.get_id());
        if let Some(id) = find {
            write.remove(id);
        }
    }
}

impl Drop for AnimatedOverlay {
    fn drop(&mut self) {
        self.is_destroyed.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
 
impl AnimatedOverlay {    
    pub fn new(gif_windows: GifWindows) -> Self {
        Self {
            frames: Vec::new(),

            max_frames: 1,
            delays: Vec::new(),
            is_destroyed: Arc::new(AtomicBool::new(false)),

            weak_gifs: Arc::new(RwLock::new(Vec::new())),

            gif_window: gif_windows
        }
    }


    pub fn load_all_images(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = std::path::Path::new(file_path);
        let gif_name = file_path.file_stem().unwrap().to_string_lossy().to_string(); 
        let path = ffmpeg_utils::get_output_path(&gif_name);

        let folder_length = get_directory_length(&path)?;

        self.frames = image_utils::get_rendered_frames(gif_name.to_string())?;

        self.delays = image_utils::get_gif_delays(&file_path).unwrap_or(vec![]); 
        self.max_frames = folder_length;

        Ok(())
    }

    pub fn start_loop(&self) {
        let frames = self.frames.clone();
        let gif_windows = self.gif_window.read().unwrap();
        let mut current_frame: usize = 0;

        let len = frames.len();
        let delays= self.delays.clone();

        for ui in gif_windows.iter() {
            ui.set_frames(ModelRc::new(VecModel::from(frames.clone())));
        }

        let destroyed_clone = self.is_destroyed.clone();
 
        let weak_gifs = self.weak_gifs.clone();

        thread::spawn(move || {
            while !destroyed_clone.load(std::sync::atomic::Ordering::Relaxed) {
                let weak_gifs = weak_gifs.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    for ui_weak in weak_gifs.read().unwrap().iter(){ 
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_currentImageIndex(current_frame as i32); 
                        }
                    }
                });

                current_frame = (current_frame + 1) % len;

                let delay = delays.get(current_frame).copied().unwrap_or(1000/10);
    
                thread::sleep(time::Duration::from_millis(delay as u64));
            }
        });

    
    }

}
