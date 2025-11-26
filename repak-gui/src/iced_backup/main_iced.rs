// Repak GUI - Iced Version
// Modular architecture for better maintainability
// Original egui version backed up in: main_egui_backup.rs, main_egui_old.rs

// Hide console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Core modules
// mod file_table;  // Has egui dependencies, not needed for Iced version
// mod install_mod;  // Old egui version - kept for reference
mod install_mod_core;  // New pure Rust version without UI dependencies
// mod uasset_detection;  // TODO: Port to Iced (has egui dependencies)
// mod uasset_api_integration;  // Not needed for Iced
mod utils;
// pub mod ios_widget;
mod utoc_utils;
// mod welcome;

// New modular structure
mod app_state;
mod messages;
mod ui;

use iced::{
    window, Application, Command, Element, Settings, Theme, Size, Event,
    event,
};
use log::{info, error};
use messages::Message;
use app_state::AppState;
use ui::MainWindow;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> iced::Result {
    // Initialize logging
    use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger, LevelFilter};
    use std::fs::File;
    
    let log_file = dirs::config_dir()
        .map(|mut p| {
            p.push("repak_mod_manager");
            std::fs::create_dir_all(&p).ok();
            p.push("repak_gui.log");
            p
        })
        .and_then(|p| File::create(p).ok());

    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![];
    
    // Only use terminal logging in debug builds (when console is available)
    #[cfg(debug_assertions)]
    loggers.push(TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ));

    // Always log to file
    if let Some(file) = log_file {
        loggers.push(WriteLogger::new(LevelFilter::Info, Config::default(), file));
    }

    CombinedLogger::init(loggers).expect("Failed to initialize logger");

    // Run the Iced application
    RepakApp::run(Settings {
        window: window::Settings {
            size: Size::new(1366.0, 768.0),
            min_size: Some(Size::new(1100.0, 650.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

// Main application struct
struct RepakApp {
    state: AppState,
}

impl Application for RepakApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let mut state = AppState::load_config().unwrap_or_default();
        
        // Auto-check for updates if enabled
        if state.auto_check_updates {
            state.spawn_update_check(false);
        }
        
        (
            Self { state },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Repak GUI v{} - Iced", VERSION)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded => {
                self.tick();
                Command::none()
            }
            
            Message::SearchChanged(query) => {
                self.state.search_query = query;
                self.state.update_search_filter();
                Command::none()
            }
            
            Message::SearchCleared => {
                self.state.search_query.clear();
                self.state.update_search_filter();
                Command::none()
            }
            
            Message::SettingsOpened => {
                self.state.show_settings = true;
                Command::none()
            }
            
            Message::SettingsClosed => {
                self.state.show_settings = false;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::GamePathBrowse => {
                // Open folder picker using rfd
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.state.game_path = path.clone();
                    self.state.game_path_input = path.to_string_lossy().to_string();
                    self.state.load_pak_files();
                    let _ = self.state.save_config();
                }
                Command::none()
            }
            
            Message::GamePathChanged(path) => {
                self.state.game_path_input = path;
                Command::none()
            }
            
            Message::GamePathSet(path) => {
                self.state.game_path = path;
                self.state.game_path_input = self.state.game_path.to_string_lossy().to_string();
                self.state.load_pak_files();
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::ModToggled(index) => {
                self.state.toggle_mod(index);
                Command::none()
            }
            
            Message::TagFilterEnabled(enabled) => {
                self.state.tag_filter_enabled = enabled;
                self.state.update_search_filter();
                Command::none()
            }
            
            Message::CustomTagFilterEnabled(enabled) => {
                self.state.custom_tag_filter_enabled = enabled;
                self.state.update_search_filter();
                Command::none()
            }
            
            Message::FiltersCleared => {
                self.state.tag_filter_enabled = false;
                self.state.selected_tag_filters.clear();
                self.state.custom_tag_filter_enabled = false;
                self.state.selected_custom_tag_filters.clear();
                self.state.update_search_filter();
                Command::none()
            }
            
            Message::SelectionModeToggled => {
                self.state.selection_mode = !self.state.selection_mode;
                if !self.state.selection_mode {
                    self.state.selected_mods.clear();
                }
                Command::none()
            }
            
            Message::TagManagerOpened => {
                self.state.show_tag_manager = true;
                Command::none()
            }
            
            Message::TagManagerClosed => {
                self.state.show_tag_manager = false;
                Command::none()
            }
            
            // Settings toggles
            Message::CompactModeToggled => {
                self.state.compact_mode = !self.state.compact_mode;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::CustomFontToggled => {
                self.state.use_custom_font = !self.state.use_custom_font;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::CustomPaletteToggled => {
                self.state.use_custom_palette = !self.state.use_custom_palette;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::ApplyPaletteInLightModeToggled => {
                self.state.apply_palette_in_light_mode = !self.state.apply_palette_in_light_mode;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::HideInternalSuffixToggled => {
                self.state.hide_internal_suffix = !self.state.hide_internal_suffix;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::ConfirmOnDeleteToggled => {
                self.state.confirm_on_delete = !self.state.confirm_on_delete;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::ShowTagChipsToggled => {
                self.state.show_tag_chips = !self.state.show_tag_chips;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::AutoCheckUpdatesToggled => {
                self.state.auto_check_updates = !self.state.auto_check_updates;
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::UsmapFileSelected => {
                // TODO: Open file picker for usmap file
                Command::none()
            }
            
            Message::UsmapFileCleared => {
                self.state.usmap_path.clear();
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::UpdateCheckStarted => {
                self.state.spawn_update_check(true);
                Command::none()
            }
            
            // Tag management
            Message::NewTagInputChanged(input) => {
                self.state.new_global_tag_input = input;
                Command::none()
            }
            
            Message::NewTagCreated => {
                let tag = self.state.new_global_tag_input.trim().to_string();
                if !tag.is_empty() && !self.state.custom_tag_catalog.contains(&tag) {
                    self.state.custom_tag_catalog.push(tag);
                    self.state.custom_tag_catalog.sort();
                    self.state.new_global_tag_input.clear();
                    let _ = self.state.save_config();
                }
                Command::none()
            }
            
            Message::TagDeleted(tag) => {
                self.state.custom_tag_catalog.retain(|t| t != &tag);
                // Remove tag from all mods
                for mod_entry in &mut self.state.pak_files {
                    mod_entry.custom_tags.retain(|t| t != &tag);
                }
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::TagAdded(mod_index, tag) => {
                if let Some(mod_entry) = self.state.pak_files.get_mut(mod_index) {
                    if !mod_entry.custom_tags.contains(&tag) {
                        mod_entry.custom_tags.push(tag.clone());
                        mod_entry.custom_tags.sort();
                    }
                }
                if !self.state.custom_tag_catalog.contains(&tag) {
                    self.state.custom_tag_catalog.push(tag);
                    self.state.custom_tag_catalog.sort();
                }
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::TagRemoved(mod_index, tag) => {
                if let Some(mod_entry) = self.state.pak_files.get_mut(mod_index) {
                    mod_entry.custom_tags.retain(|t| t != &tag);
                }
                let _ = self.state.save_config();
                Command::none()
            }
            
            Message::BulkTagAdded(tag) => {
                self.state.bulk_add_tag_to_selected(&tag);
                Command::none()
            }
            
            Message::BulkTagRemoved(tag) => {
                for &index in &self.state.selected_mods {
                    if let Some(mod_entry) = self.state.pak_files.get_mut(index) {
                        mod_entry.custom_tags.retain(|t| t != &tag);
                    }
                }
                let _ = self.state.save_config();
                Command::none()
            }
            
            // Folder operations
            Message::FolderCreated(name) => {
                if !name.trim().is_empty() {
                    self.state.create_folder(name);
                }
                Command::none()
            }
            
            Message::FolderDeleted(folder_id) => {
                self.state.delete_folder(&folder_id);
                Command::none()
            }
            
            Message::FolderToggled(folder_id) => {
                self.state.toggle_folder(&folder_id);
                Command::none()
            }
            
            Message::FolderExpanded(folder_id, expanded) => {
                if let Some(folder) = self.state.folders.iter_mut().find(|f| f.id == folder_id) {
                    folder.expanded = expanded;
                    let _ = self.state.save_config();
                }
                Command::none()
            }
            
            // Mod operations
            Message::ModRenamed(index, new_name) => {
                if let Some(mod_entry) = self.state.pak_files.get_mut(index) {
                    // TODO: Actually rename the file
                    info!("Rename mod {} to {}", index, new_name);
                }
                Command::none()
            }
            
            Message::ModDeleted(index) => {
                if self.state.confirm_on_delete {
                    // TODO: Show confirmation dialog
                    info!("Delete mod {} (confirmation needed)", index);
                } else {
                    self.state.delete_mod(index);
                }
                Command::none()
            }
            
            // Error handling
            Message::ErrorOccurred(error) => {
                error!("Error: {}", error);
                // TODO: Show error dialog
                Command::none()
            }
            
            Message::ErrorDismissed => {
                // TODO: Hide error dialog
                Command::none()
            }
            
            // Mod selection
            Message::ModSelected(index) => {
                self.state.select_mod(index);
                Command::none()
            }
            
            Message::SelectionCleared => {
                self.state.clear_selection();
                Command::none()
            }
            
            Message::FileInTableSelected(index) => {
                self.state.selected_file_in_table = Some(index);
                Command::none()
            }
            
            // Mod installation
            Message::FilesDropped(paths) => {
                info!("Files dropped: {} files", paths.len());
                
                // Parse dropped files
                let game_path = self.state.game_path.clone();
                let usmap_path = self.state.usmap_path.clone();
                
                Command::perform(
                    async move {
                        install_mod_core::parse_dropped_files(
                            paths,
                            &game_path,
                            None, // TODO: Convert usmap_path from String to Option<PathBuf>
                        )
                    },
                    |result| match result {
                        Ok(mods) => {
                            info!("Parsed {} mods", mods.len());
                            Message::ModsParsed(mods)
                        }
                        Err(e) => {
                            error!("Failed to parse mods: {}", e);
                            Message::ErrorOccurred(e)
                        }
                    }
                )
            }
            
            Message::InstallModClicked => {
                // Open file picker
                Command::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Mod Files", &["pak", "zip"])
                            .set_title("Select Mod Files")
                            .pick_files()
                            .await
                    },
                    |result| {
                        if let Some(files) = result {
                            let paths: Vec<std::path::PathBuf> = files.iter()
                                .map(|f| f.path().to_path_buf())
                                .collect();
                            Message::FilesDropped(paths)
                        } else {
                            Message::InstallCancelled
                        }
                    }
                )
            }
            
            Message::ModsParsed(mods) => {
                info!("Mods parsed successfully: {}", mods.len());
                self.state.pending_install_mods = mods;
                self.state.show_install_dialog = true;
                
                // Auto-detect mesh and texture files
                // TODO: Implement auto-detection based on mod contents
                
                Command::none()
            }
            
            Message::InstallDialogClosed => {
                self.state.show_install_dialog = false;
                self.state.pending_install_mods.clear();
                Command::none()
            }
            
            Message::InstallConfirmed => {
                info!("Installing {} mods", self.state.pending_install_mods.len());
                self.state.installing = true;
                self.state.install_status = "Installing...".to_string();
                self.state.install_progress = 0.0;
                
                // Prepare installation
                let mods = self.state.pending_install_mods.clone();
                let options = install_mod_core::InstallOptions {
                    fix_mesh: self.state.install_fix_mesh,
                    fix_texture: self.state.install_fix_texture,
                    to_iostore: self.state.install_to_iostore,
                    game_path: self.state.game_path.clone(),
                    usmap_path: None, // TODO: Convert usmap_path from String to Option<PathBuf>
                };
                
                let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                
                // Start installation
                Command::perform(
                    async move {
                        install_mod_core::install_mods(
                            mods,
                            options,
                            Box::new(|progress, status| {
                                // Progress callback - we'll handle this via messages
                                info!("Progress: {:.0}% - {}", progress * 100.0, status);
                            }),
                            cancel_flag,
                        ).await
                    },
                    |result| match result {
                        Ok(()) => {
                            info!("Installation complete!");
                            Message::ModInstallCompleted(Ok(()))
                        }
                        Err(e) => {
                            error!("Installation failed: {}", e);
                            Message::ModInstallCompleted(Err(e))
                        }
                    }
                )
            }
            
            Message::InstallCancelled => {
                self.state.show_install_dialog = false;
                self.state.pending_install_mods.clear();
                Command::none()
            }
            
            Message::InstallProgress(progress, status) => {
                self.state.install_progress = progress;
                self.state.install_status = status;
                Command::none()
            }
            
            Message::InstallOptionChanged(option) => {
                use messages::InstallOption;
                match option {
                    InstallOption::FixMesh(value) => self.state.install_fix_mesh = value,
                    InstallOption::FixTexture(value) => self.state.install_fix_texture = value,
                    InstallOption::ToIoStore(value) => self.state.install_to_iostore = value,
                    InstallOption::Compression(_) => {}, // TODO: Handle compression
                }
                Command::none()
            }
            
            // Placeholder for other messages
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        MainWindow::view(&self.state)
    }
    
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            // Poll for update results every second
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| Message::Loaded),
            // Listen for window events (drag & drop)
            event::listen_with(|event, _status| {
                match event {
                    Event::Window(_id, window::Event::FileDropped(path)) => {
                        Some(Message::FilesDropped(vec![path]))
                    }
                    _ => None
                }
            })
        ])
    }
}

impl RepakApp {
    fn tick(&mut self) {
        // Check for update results
        self.state.check_update_result();
    }
}
