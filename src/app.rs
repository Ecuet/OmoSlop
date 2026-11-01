use std::{ collections::HashMap, fs, rc::Rc, sync::{ Arc, RwLock } };

use pollster::FutureExt;
use slint::{ Model, ModelRc, ToSharedString, VecModel };
use crate::{
	config::{ self, Config },
	ffmpeg_utils,
	ignore_err,
	image_utils,
	overlay::OverlayKind,
	overlays_utils,
	windows::{ main_window, settings_window, tray },
	windows_utils,
};

slint::include_modules!();

pub type GifWindows = Arc<RwLock<Vec<GifWindow>>>;
pub struct OverlayWindow {
	pub gif_windows: GifWindows,
	pub overlay: OverlayKind,
}

pub type Overlays = Arc<RwLock<HashMap<String, OverlayWindow>>>;
pub struct App {
	main_window: MainWindow,
	settings_window: SettingsWindow,

	tray: MainWindowTray,

	overlays: Overlays,

	config: Arc<RwLock<Config>>,
}

impl App {
	pub fn new(config: Arc<RwLock<Config>>) -> Result<Self, Box<dyn std::error::Error>> {
		let main_window = main_window::create()?;
		let settings_window = settings_window::create()?;

		let overlays = Arc::new(RwLock::new(HashMap::new()));

		let tray = tray::create(&main_window, config.clone(), &overlays.clone())?;

		Ok(Self {
			main_window: main_window,
			settings_window: settings_window,

			tray: tray,

			overlays: overlays,

			config: config,
		})
	}

	fn load_settings(&self) {
		let config = self.config.read().unwrap();
		let images_ui_data = Rc::new(VecModel::<ImageData>::default());

		images_ui_data.push(ImageData::default());

		for (name, gif_data) in config.gifs.iter() {
			images_ui_data.push(ImageData {
				name: name.to_shared_string(),
				frames: ModelRc::new(
					VecModel::from(image_utils::get_rendered_frames(name.to_string()).unwrap_or_default())
				),
			});

			ignore_err!(overlays_utils::create_all_overlays(&gif_data, &self.overlays));

			ignore_err!(
				overlays_utils::change_overlay_scale(
					&self.overlays,
					name.to_string(),
					gif_data.window_scale,
					&gif_data.image_size
				)
			);

			ignore_err!(ffmpeg_utils::convert_gif(&gif_data.selected_file_path, false));
		}

		for (name, over) in self.overlays.read().unwrap().iter() {
			let locked = config.gifs.get(name).unwrap().locked;
			for i in over.gif_windows.read().unwrap().iter() {
				windows_utils::toggle_clickability(i, !locked);
			}
		}

		self.main_window.set_ImagesData(ModelRc::from(images_ui_data));
	}

	fn setup_settings(&self) {}

	fn setup_handlers(&self) {
		let weak_main = self.main_window.as_weak();
		let file_config = self.config.clone();

		self.main_window.on_FileChooser(move || {
			let weak_main = weak_main.clone();
			let file_config = file_config.clone();

			std::thread::spawn(move || {
				let Some(path) = image_utils::open_file_chooser().block_on() else {
					return;
				};
				let Some(string_path) = path.to_str() else {
					return;
				};

				let mut config = file_config.write().unwrap();

				let Ok(name) = image_utils::get_file_name(&path) else {
					return;
				};

				let Ok(size) = image_utils::get_image_size(&path) else {
					return;
				};

				if let Some(_found) = config.gifs.iter().find(|&x| x.1.selected_file_path == path) {
					rfd::MessageDialog
						::new()
						.set_title("Failed")
						.set_description("This file is already loaded.")
						.set_buttons(rfd::MessageButtons::Ok)
						.show();
					return;
				}

				let new_gif = config::GifData {
					name: name.to_string(),

					selected_file_path: string_path.to_string(),
					window_scale: 1.0,

					image_size: size,

					locked: false,
					overlays: Vec::new(),
				};

				config.gifs.insert(name.to_string(), new_gif);

				//let image_data = image_data.clone();

				slint
					::invoke_from_event_loop(move || {
						let Some(main_window) = weak_main.upgrade() else {
							return;
						};

						if let Err(e) = ffmpeg_utils::convert_gif(&path.to_string_lossy().to_string(), false) {
							rfd::MessageDialog
								::new()
								.set_title("Failed")
								.set_description(format!("Failed to load the gif: {}", e))
								.set_buttons(rfd::MessageButtons::Ok)
								.show();
							eprintln!("Convert gif failed: {}", e);
							return;
						}

						let image_data = ImageData {
							name: name.to_shared_string(),
							frames: ModelRc::new(
								VecModel::from(
									image_utils::get_rendered_frames(name.to_string()).unwrap_or_default()
								)
							),
						};

						let model = main_window.get_ImagesData();
						model.as_any().downcast_ref::<VecModel<ImageData>>().unwrap().push(image_data);
					})
					.unwrap();
			});
		});

		let config = self.config.clone();
		let overlays = self.overlays.clone();
		self.main_window.on_AddOverlay(move |name| {
			ignore_err!(
				overlays_utils::create_overlay_with_saving(
					&mut *config.write().unwrap(),
					name.to_string(),
					&overlays
				)
			);
		});

		let config = self.config.clone();
		let overlays = self.overlays.clone();
		self.main_window.on_RemoveOverlay(move |name| {
			ignore_err!(
				overlays_utils::remove_overlay(&mut *config.write().unwrap(), name.to_string(), &overlays)
			);
		});

		let config = self.config.clone();
		let overlays = self.overlays.clone();
		self.main_window.on_ChangeScale(move |name, scale| {
			let mut config = config.write().unwrap();
			let Some(get) = config.gifs.get(&name.to_string()) else {return};
			let size = get.image_size.clone();

			ignore_err!(
				overlays_utils::change_overlay_scale_with_saving(
					&mut *config,
					&overlays,
					name.to_string(),
					scale,
					&size
				)
			);
		});

		let config = self.config.clone();
		let overlays = self.overlays.clone();
		self.main_window.on_LockChanged(move |name, state| {
			let mut config = config.write().unwrap();
			let Some(get_config) = config.gifs.get_mut(&name.to_string()) else {
				return;
			};
			get_config.locked = state;

			let Ok(overlay_read) = overlays.read() else {
				return;
			};
			let Some(overlays) = overlay_read.get(&name.to_string()) else {
				return;
			};

			for i in overlays.gif_windows.read().unwrap().iter() {
				windows_utils::toggle_clickability(i, !state);
			}
		});

		let config = self.config.clone();
		let overlays = self.overlays.clone();
		let main_window_weak = self.main_window.as_weak();

		self.main_window.on_DeleteFromList(move |name| {
			let Some(main_window) = main_window_weak.upgrade() else {return};
			let mut config = config.write().unwrap();

			ignore_err!(overlays_utils::remove_all_overlays(&mut *config, name.to_string(), &overlays));

			config.gifs.remove(&name.to_string());

			config.save();

			let model = main_window.get_ImagesData();

			let downgrade=  model.as_any().downcast_ref::<VecModel<ImageData>>().unwrap();
		

			for i in 0..model.row_count() {
				if model.row_data(i).unwrap().name == name {
					downgrade.remove(i);
					break;
				}
			}
			main_window.set_current_selected_image("".into());
 
			let output_path = ffmpeg_utils::get_output_path(&name.to_string());
			ignore_err!(fs::remove_dir_all(output_path));
		});

		let weak_settings = self.settings_window.as_weak();

		self.main_window.on_OpenSettings(move || {
			if let Some(settings_window) = weak_settings.upgrade() {
				settings_window.show().unwrap();
			}
		});
		let main_weak = self.main_window.as_weak();
		let config = self.config.clone();
		self.main_window.on_UpdateUIOverlaySettings(move |name| {
			let Some(main_window) = main_weak.upgrade() else {
				return;
			};
			let config = config.read().unwrap();

			let Some(overlay_data) = config.gifs.get(&name.to_string()) else {
				return;
			};

			main_window.set_ui_Lock(overlay_data.locked);
			main_window.set_ui_Scale(overlay_data.window_scale);
		});
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
