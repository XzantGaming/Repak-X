// Settings window UI
// In Iced 0.12, this is rendered as a modal overlay
// TODO: Upgrade to Iced 0.13+ for true multi-window support

use iced::{Element, Length, Alignment, Color, Background, Border};
use iced::widget::{button, column, container, row, text, checkbox};
use crate::messages::Message;
use crate::app_state::AppState;

pub struct SettingsWindow;

impl SettingsWindow {
    pub fn view(state: &AppState) -> Element<Message> {
        if !state.show_settings {
            return column![].into();
        }
        
        let content = column![
            // Header
            row![
                text("Settings").size(24),
                button(text("✕")).on_press(Message::SettingsClosed),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
            
            // Settings sections
            Self::appearance_section(state),
            Self::general_section(state),
            Self::mods_view_section(state),
            Self::usmap_section(state),
            Self::updates_section(state),
            
            // Close button
            row![
                button(text("Close")).on_press(Message::SettingsClosed)
                    .width(Length::Fixed(100.0)),
            ]
            .spacing(10),
        ]
        .spacing(20)
        .padding(20);
        
        // Simple centered container
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
    
    fn appearance_section(state: &AppState) -> Element<'static, Message> {
        column![
            text("Appearance").size(18),
            row![
                text("UI Scale:"),
                text(format!("{:.1}x", state.ui_scale)),
            ]
            .spacing(10),
            checkbox("Compact Mode", state.compact_mode)
                .on_toggle(|_| Message::CompactModeToggled),
            checkbox("Use Custom Font", state.use_custom_font)
                .on_toggle(|_| Message::CustomFontToggled),
            checkbox("Use Custom Palette", state.use_custom_palette)
                .on_toggle(|_| Message::CustomPaletteToggled),
            checkbox("Apply Palette in Light Mode", state.apply_palette_in_light_mode)
                .on_toggle(|_| Message::ApplyPaletteInLightModeToggled),
        ]
        .spacing(10)
        .into()
    }
    
    fn general_section(state: &AppState) -> Element<'static, Message> {
        column![
            text("General").size(18),
            checkbox("Hide Internal Suffix (_P)", state.hide_internal_suffix)
                .on_toggle(|_| Message::HideInternalSuffixToggled),
            checkbox("Confirm Before Deleting", state.confirm_on_delete)
                .on_toggle(|_| Message::ConfirmOnDeleteToggled),
        ]
        .spacing(10)
        .into()
    }
    
    fn mods_view_section(state: &AppState) -> Element<'static, Message> {
        column![
            text("Mods View").size(18),
            checkbox("Show Tag Chips", state.show_tag_chips)
                .on_toggle(|_| Message::ShowTagChipsToggled),
        ]
        .spacing(10)
        .into()
    }
    
    fn usmap_section(state: &AppState) -> Element<Message> {
        column![
            text("USmap Configuration").size(18),
            row![
                text("USmap File:"),
                text(if state.usmap_path.is_empty() { 
                    "Not set" 
                } else { 
                    &state.usmap_path 
                }),
            ]
            .spacing(10),
            row![
                button(text("Browse")).on_press(Message::UsmapFileSelected),
                button(text("Clear")).on_press(Message::UsmapFileCleared),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .into()
    }
    
    fn updates_section(state: &AppState) -> Element<'static, Message> {
        column![
            text("Updates").size(18),
            checkbox("Auto-check for updates", state.auto_check_updates)
                .on_toggle(|_| Message::AutoCheckUpdatesToggled),
            button(text("Check Now")).on_press(Message::UpdateCheckStarted),
        ]
        .spacing(10)
        .into()
    }
}
