// Dialogs UI - Tag Manager and other dialogs

use iced::{Element, Length, Alignment};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use crate::messages::Message;
use crate::app_state::AppState;
use std::collections::BTreeSet;

pub struct Dialogs;

impl Dialogs {
    pub fn tag_manager(state: &AppState) -> Element<Message> {
        if !state.show_tag_manager {
            return column![].into();
        }
        
        let all_tags = state.get_all_custom_tags();
        
        let mut tag_list = column![
            text("Existing Tags").size(16),
        ].spacing(5);
        
        for tag in &all_tags {
            let tag_row = row![
                text(tag.clone()).width(Length::Fill),
                button(text("Delete")).on_press(Message::TagDeleted(tag.clone())),
            ]
            .spacing(10)
            .align_items(Alignment::Center);
            
            tag_list = tag_list.push(tag_row);
        }
        
        let content = column![
            // Header
            row![
                text("Tag Manager").size(24),
                Space::with_width(Length::Fill),
                button(text("✕")).on_press(Message::TagManagerClosed),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
            
            // New tag input
            text("Create New Tag").size(18),
            row![
                text_input("Tag name...", &state.new_global_tag_input)
                    .on_input(Message::NewTagInputChanged)
                    .width(Length::Fixed(300.0)),
                button(text("Create")).on_press(Message::NewTagCreated),
            ]
            .spacing(10),
            
            // Tag list
            scrollable(tag_list)
                .height(Length::Fixed(300.0)),
            
            // Close button
            button(text("Close")).on_press(Message::TagManagerClosed)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fixed(500.0));
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
    
    pub fn error_dialog(message: &str) -> Element<'static, Message> {
        let content = column![
            text("Error").size(24),
            text(message),
            button(text("OK")).on_press(Message::ErrorDismissed),
        ]
        .spacing(20)
        .padding(20);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
