// Main window UI - Primary application interface

use iced::{Element, Length, Alignment, Color, Border, Theme};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Space};
use crate::messages::Message;
use crate::app_state::AppState;
use super::{SettingsWindow, dialogs::Dialogs, PakViewer, InstallDialog};

pub struct MainWindow;

impl MainWindow {
    pub fn view(state: &AppState) -> Element<Message> {
        // Show dialogs/windows based on state
        if state.show_settings {
            return SettingsWindow::view(state);
        } else if state.show_tag_manager {
            return Dialogs::tag_manager(state);
        }
        
        // Main 2-panel layout
        let left_panel = column![
            // Title bar
            Self::title_bar(),
            
            // Game path selector
            Self::game_path_section(state),
            
            // Search and filters
            Self::search_section(state),
            
            // Mod list
            Self::mod_list_section(state),
            
            // Status bar
            Self::status_bar(state),
        ]
        .spacing(10)
        .padding(10)
        .width(Length::FillPortion(3));
        
        // Right panel - PAK viewer
        let right_panel = container(
            PakViewer::view(state)
        )
        .width(Length::FillPortion(7))
        .height(Length::Fill)
        .padding(10);
        
        // Combine panels
        let main_view = row![left_panel, right_panel]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);
        
        // Show install dialog if active (using container overlay instead of Stack)
        if state.show_install_dialog {
            container(
                column![
                    main_view,
                    InstallDialog::view(state)
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            main_view.into()
        }
    }
    
    fn title_bar() -> Element<'static, Message> {
        container(
            row![
                text("🎮 Repak GUI").size(28).style(Color::from_rgb(0.4, 0.7, 1.0)),
                Space::with_width(Length::Fill),
                button("⚙️ Settings")
                    .on_press(Message::SettingsOpened)
                    .padding(10),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .padding(15)
        .style(|_theme: &Theme| {
            container::Appearance {
                background: Some(Color::from_rgb(0.15, 0.15, 0.18).into()),
                border: Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .into()
    }
    
    fn game_path_section(state: &AppState) -> Element<Message> {
        container(
            column![
                text("📁 Game Directory").size(16).style(Color::from_rgb(0.7, 0.7, 0.7)),
                row![
                    text_input("Select game directory...", &state.game_path_input)
                        .on_input(Message::GamePathChanged)
                        .padding(10)
                        .width(Length::Fill),
                    button("Browse")
                        .on_press(Message::GamePathBrowse)
                        .padding(10),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            ]
            .spacing(8)
        )
        .padding(15)
        .style(|_theme: &Theme| {
            container::Appearance {
                background: Some(Color::from_rgb(0.12, 0.12, 0.15).into()),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb(0.3, 0.3, 0.35),
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .into()
    }
    
    fn search_section(state: &AppState) -> Element<Message> {
        let search_box = container(
            column![
                row![
                    text("🔍 Search").size(16).style(Color::from_rgb(0.7, 0.7, 0.7)),
                    Space::with_width(Length::Fill),
                    button("🏷️ Tags")
                        .on_press(Message::TagManagerOpened)
                        .padding(8),
                ],
                row![
                    text_input("Search mods...", &state.search_query)
                        .on_input(Message::SearchChanged)
                        .padding(10)
                        .width(Length::Fill),
                    button("✖")
                        .on_press(Message::SearchCleared)
                        .padding(10),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            ]
            .spacing(8)
        )
        .padding(15)
        .style(|_theme: &Theme| {
            container::Appearance {
                background: Some(Color::from_rgb(0.12, 0.12, 0.15).into()),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb(0.3, 0.3, 0.35),
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill);
        
        search_box.into()
    }
    
    fn mod_list_section(state: &AppState) -> Element<Message> {
        if state.pak_files.is_empty() {
            return container(
                text("No mods found. Select a game directory to load mods.")
                    .size(16)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }
        
        let mut mod_list = Column::new().spacing(5);
        
        // Group by folder
        let mut mods_by_folder: std::collections::HashMap<Option<String>, Vec<usize>> = 
            std::collections::HashMap::new();
        
        for (index, mod_entry) in state.pak_files.iter().enumerate() {
            if !state.is_mod_visible(index) {
                continue;
            }
            mods_by_folder.entry(mod_entry.folder_id.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }
        
        // Display folders first
        for folder in &state.folders {
            if let Some(mod_indices) = mods_by_folder.get(&Some(folder.id.clone())) {
                if mod_indices.is_empty() {
                    continue;
                }
                
                // Folder header
                let folder_header = row![
                    text(format!("📁 {}", folder.name)).size(16),
                    Space::with_width(Length::Fill),
                    text(format!("{} mods", mod_indices.len())).size(14),
                ]
                .spacing(10)
                .padding(5);
                
                mod_list = mod_list.push(folder_header);
                
                // Mods in folder
                if folder.expanded {
                    for &index in mod_indices {
                        if let Some(mod_entry) = state.pak_files.get(index) {
                            mod_list = mod_list.push(Self::mod_entry_row(state, index, mod_entry, true));
                        }
                    }
                }
            }
        }
        
        // Display mods without folder
        if let Some(mod_indices) = mods_by_folder.get(&None) {
            for &index in mod_indices {
                if let Some(mod_entry) = state.pak_files.get(index) {
                    mod_list = mod_list.push(Self::mod_entry_row(state, index, mod_entry, false));
                }
            }
        }
        
        scrollable(mod_list)
            .height(Length::Fill)
            .into()
    }
    
    fn mod_entry_row(state: &AppState, index: usize, mod_entry: &crate::app_state::ModEntry, indented: bool) -> Element<'static, Message> {
        let mut mod_name = mod_entry.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        // Apply hide_internal_suffix if enabled
        if state.hide_internal_suffix {
            // Remove _P suffix (e.g., "_1234_P")
            if let Some(pos) = mod_name.rfind('_') {
                if let Some(suffix) = mod_name.get(pos+1..) {
                    if suffix.chars().all(|c| c.is_numeric() || c == 'P') && suffix.ends_with('P') {
                        mod_name = mod_name[..pos].to_string();
                    }
                }
            }
        }
        
        let checkbox_icon = if mod_entry.enabled { "☑" } else { "☐" };
        let _is_selected = state.current_pak_file_idx == Some(index);
        
        // Create clickable row for selection
        let mod_button = button(
            row![
                text(checkbox_icon),
                Space::with_width(Length::Fixed(5.0)),
                text(&mod_name),
            ]
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .on_press(Message::ModSelected(index));
        
        let toggle_button = button(
            text(if mod_entry.enabled { "✓" } else { "✗" })
        )
        .on_press(Message::ModToggled(index));
        
        let mut row_content = row![
            mod_button,
            toggle_button,
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        if indented {
            row_content = row![
                Space::with_width(Length::Fixed(20.0)),
                row_content,
            ]
            .spacing(0);
        }
        
        // Add tags if enabled
        if state.show_tag_chips && !mod_entry.custom_tags.is_empty() {
            let tags_text = mod_entry.custom_tags.join(", ");
            row_content = row![
                row_content,
                text(format!("🏷 {}", tags_text)).size(12),
            ]
            .spacing(10);
        }
        
        row_content.into()
    }
    
    fn status_bar(state: &AppState) -> Element<Message> {
        let status_text = if state.pak_files.is_empty() {
            "No mods loaded".to_string()
        } else {
            let visible_count = state.pak_files.iter()
                .enumerate()
                .filter(|(i, _)| state.is_mod_visible(*i))
                .count();
            format!("{} mods ({} visible)", state.pak_files.len(), visible_count)
        };
        
        row![
            text(status_text),
            Space::with_width(Length::Fill),
            button("📦 Install Mod")
                .on_press(Message::InstallModClicked)
                .padding(10),
            text(format!("v{}", env!("CARGO_PKG_VERSION"))),
        ]
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center)
        .into()
    }
}
