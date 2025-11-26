// PAK Viewer - Shows contents and details of selected PAK file

use iced::{Element, Length, Alignment};
use iced::widget::{button, column, container, row, scrollable, text, Space, Column};
use crate::messages::Message;
use crate::app_state::AppState;

pub struct PakViewer;

impl PakViewer {
    /// Main view showing both file list and details
    pub fn view(state: &AppState) -> Element<Message> {
        if state.current_pak_file_idx.is_none() {
            return Self::empty_view();
        }
        
        column![
            // File list (top 60%)
            container(Self::file_list_view(state))
                .height(Length::FillPortion(6))
                .width(Length::Fill)
                .padding(10),
            
            // Details panel (bottom 40%)
            container(Self::details_view(state))
                .height(Length::FillPortion(4))
                .width(Length::Fill)
                .padding(10),
        ]
        .spacing(0)
        .into()
    }
    
    fn empty_view() -> Element<'static, Message> {
        container(
            column![
                text("Select a mod to view its contents")
                    .size(18),
                text("Click on a mod in the left panel")
                    .size(14),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
    
    fn file_list_view(state: &AppState) -> Element<'static, Message> {
        let files = state.get_pak_files_list();
        
        if files.is_empty() {
            return text("No files found in this PAK")
                .size(14)
                .into();
        }
        
        // Check if using IoStore format
        let using_utoc = if let Some(pak_entry) = state.get_selected_pak() {
            let mut utoc_path = pak_entry.path.clone();
            utoc_path.set_extension("utoc");
            utoc_path.exists()
        } else {
            false
        };
        
        let mut file_column = Column::new()
            .spacing(2)
            .padding(5);
        
        let header_text = if using_utoc {
            format!("Files in PAK (IoStore) - {} files", files.len())
        } else {
            format!("Files in PAK - {} files", files.len())
        };
        
        file_column = file_column.push(
            row![
                text(header_text).size(16),
            ]
            .spacing(10)
            .padding(5)
        );
        
        for (idx, file_path) in files.iter().enumerate() {
            let _is_selected = state.selected_file_in_table == Some(idx);
            
            let file_row = button(
                text(file_path)
                    .size(12)
            )
            .width(Length::Fill)
            .on_press(Message::FileInTableSelected(idx));
            
            file_column = file_column.push(file_row);
        }
        
        scrollable(file_column)
            .height(Length::Fill)
            .into()
    }
    
    fn details_view(state: &AppState) -> Element<'static, Message> {
        let Some(pak_entry) = state.get_selected_pak() else {
            return text("No details available").into();
        };
        
        let pak = &pak_entry.reader;
        
        // Pre-format all strings to avoid lifetime issues
        let encrypted_index = format!("{}", pak.encrypted_index());
        let encryption_guid = format!("{:?}", pak.encryption_guid());
        let mount_point = pak.mount_point().to_string();
        let version = format!("{:?}", pak.version());
        let path_hash_seed = format!("{:?}", pak.path_hash_seed());
        let mod_type = Self::detect_mod_type(state);
        
        let mut details = Column::new()
            .spacing(10)
            .padding(10);
        
        // Title
        details = details.push(
            text("PAK Details").size(18)
        );
        
        // Encryption details
        details = details.push(
            column![
                text("Encryption").size(14),
                row![text("Encrypted Index:"), Space::with_width(Length::Fixed(10.0)), text(encrypted_index)].spacing(5),
                row![text("Encryption GUID:"), Space::with_width(Length::Fixed(10.0)), text(encryption_guid)].spacing(5),
            ]
            .spacing(5)
            .padding(5)
        );
        
        // PAK details
        details = details.push(
            column![
                text("PAK Info").size(14),
                row![text("Mount Point:"), Space::with_width(Length::Fixed(10.0)), text(mount_point)].spacing(5),
                row![text("Version:"), Space::with_width(Length::Fixed(10.0)), text(version)].spacing(5),
                row![text("Path Hash Seed:"), Space::with_width(Length::Fixed(10.0)), text(path_hash_seed)].spacing(5),
            ]
            .spacing(5)
            .padding(5)
        );
        
        // Mod type
        details = details.push(
            row![text("Mod Type:"), Space::with_width(Length::Fixed(10.0)), text(mod_type)].spacing(5)
        );
        
        scrollable(details)
            .height(Length::Fill)
            .into()
    }
    
    fn detect_mod_type(state: &AppState) -> String {
        let files = state.get_pak_files_list();
        // Use the sophisticated detection from utils that includes character names
        crate::utils::get_current_pak_characteristics(files)
    }
}
