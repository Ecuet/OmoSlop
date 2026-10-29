use std::{rc::Rc, sync::{Arc, RwLock, Weak}};


use slint::{Image, ModelRc, PhysicalPosition, ToSharedString, VecModel};

use crate::{config::{self, Config}, image_utils, overlay::{AnimatedOverlay, Overlay, OverlayKind}, windows::{gif_window, main_window, settings_window, tray}};

slint::include_modules!();

pub struct OverlayWindow {
    gif_window: GifWindow,
    overlay: OverlayKind
}

pub struct App {
    main_window: MainWindow,
    settings_window: SettingsWindow,

    tray: MainWindowTray,

    images_ui_data: Rc<slint::VecModel<ImageData>>,

    overlays: Arc<RwLock<Vec<OverlayWindow>>>,

    config: Arc<RwLock<Config>>
}

impl App {
    pub fn new(config: Arc<RwLock<Config>>) -> Result<Self, Box<dyn std::error::Error>> {
        let main_window = main_window::create()?;
        let settings_window = settings_window::create()?;

        let tray = tray::create(&main_window, config.clone())?;

       // let overlay = Arc::new(RwLock::new(OverlayKind::Animated(AnimatedOverlay::new(gif_window.as_weak()))));

        Ok(Self {
            main_window: main_window,
            settings_window: settings_window,

            tray: tray,
            
            images_ui_data: Rc::new(VecModel::<ImageData>::default()),

            overlays: Arc::new(RwLock::new(Vec::new())),

            config: config
        })
    }

    fn load_settings(&self) {
        // let config_read = self.config.read().unwrap();

        // let scale = config_read.window_scale;
        // image_utils::change_gif_scale(scale, &self.gif_window);

        // self.gif_window.window().set_position(PhysicalPosition::new(config_read.overlay_x, config_read.overlay_y));

        self.main_window.set_ImagesData(ModelRc::from(self.images_ui_data.clone()));
    }


    fn setup_settings(&self) {
        //let weak_scale_ui = self.gif_window.as_weak();
        //let file_config = self.config.clone();
        // self.settings_window.on_ScaleChanged(move |scale: f32| {
        //     if let Some(gif_ui) = weak_scale_ui.upgrade() {
        //         image_utils::change_gif_scale(scale, &gif_ui);

        //         file_config.write().unwrap().window_scale = scale;
        //     }
        // });

        // let weak_scale_ui = self.gif_window.as_weak();
        // let file_config = self.config.clone();
        // self.settings_window.on_LockChanged(move |state: bool| {
        //     if let Some(gif_ui) = weak_scale_ui.upgrade() {
        //         gif_ui.set_Locked(state);

        //         file_config.write().unwrap().locked = state;
        //     }
        // });


    }

    fn setup_handlers(&self) {
        let weak_main = self.main_window.as_weak();
        let file_config = self.config.clone();
        let images_data = self.images_ui_data.clone();

        self.main_window.on_FileChooser(move || {
            let Some(path) = image_utils::open_file_chooser() else { return };
            let Some(string_path) = path.to_str() else { return };
            let Some(main_window) = weak_main.upgrade() else { return };
            
            let mut config = file_config.write().unwrap();

            let Ok(name) = image_utils::get_file_name(&path) else { return };

            let new_gif = config::GifData {
                name: name.to_string(),

                selected_file_path: string_path.to_string(),
                window_scale: 1.0,

                overlay_x: 0,
                overlay_y: 0,

                locked: false
            };

            config.gifs.insert(name.to_string(), new_gif);

            images_data.push(ImageData {
                 name: name.to_shared_string() 
            });

            main_window.set_ImagesData(ModelRc::from(images_data.clone()));
        });

        let config = self.config.clone();
        let overlays = self.overlays.clone();
        self.main_window.on_AddOverlay(move |name| {
            let _ = Self::create_overlay(&config.write().unwrap(), name.to_string(), &overlays);
        });

        let weak_settings = self.settings_window.as_weak();

        self.main_window.on_OpenSettings(move || {
            if let Some(settings_window) = weak_settings.upgrade() {
                settings_window.show().unwrap();
            }
        });
    }
    fn create_overlay(config: &Config, name: String, overlays: &Arc<RwLock<Vec<OverlayWindow>>>) -> Result<(), Box<dyn std::error::Error>> {
        let overlay_data = config.gifs.get(&name).ok_or("no")?;

        let gif_window = gif_window::create()?;
        let weak = gif_window.as_weak();

        let mut loaded_overlay = OverlayKind::Animated(AnimatedOverlay::new(weak));

        loaded_overlay.load_file(&overlay_data.selected_file_path, false)?;
        gif_window.show()?;

        let overlay_window = OverlayWindow {
            gif_window: gif_window,
            overlay: loaded_overlay
        };
        

        overlays.write().unwrap().push(overlay_window);

        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.setup_handlers();
        self.load_settings();
        self.setup_settings();

        self.tray.show()?;
        self.main_window.show()?;


        slint::run_event_loop()?;

        Ok(())
    }
}