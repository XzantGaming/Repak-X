// Installation Dialog UI

use iced::{Element, Length, Alignment, Color, Border, Theme};
use iced::widget::{button, column, container, row, text, checkbox, Space, scrollable};
use crate::messages::{Message, InstallOption};
use crate::app_state::AppState;

pub struct InstallDialog;

impl InstallDialog {
    pub fn view(state: &AppState) -> Element<'static, Message> {
        if !state.show_install_dialog {
            return Space::new(Length::Shrink, Length::Shrink).into();
        }
        
        let mod_count = state.pending_install_mods.len();
        
        // Build mod list
        let mut mod_list = column![].spacing(5);
        for mod_info in &state.pending_install_mods {
            mod_list = mod_list.push(
                text(format!("• {} ({} files) - {}", 
                    mod_info.mod_name, 
                    mod_info.file_count,
                    mod_info.mod_type
                )).size(14)
            );
        }
        
        let content = container(
            column![
                // Title
                text("📦 Install Mods").size(24),
                
                Space::with_height(Length::Fixed(20.0)),
                
                // Mod info
                text(format!("{} mod(s) to install:", mod_count)).size(16),
                
                Space::with_height(Length::Fixed(10.0)),
                
                // Mod list
                scrollable(mod_list).height(Length::Fixed(150.0)),
                
                Space::with_height(Length::Fixed(20.0)),
                
                // Options
                text("Options:").size(18),
                
                Space::with_height(Length::Fixed(10.0)),
                
                checkbox("Fix Mesh Issues", state.install_fix_mesh)
                    .on_toggle(|v| Message::InstallOptionChanged(InstallOption::FixMesh(v))),
                
                checkbox("Fix Texture Issues", state.install_fix_texture)
                    .on_toggle(|v| Message::InstallOptionChanged(InstallOption::FixTexture(v))),
                
                checkbox("Convert to IoStore", state.install_to_iostore)
                    .on_toggle(|v| Message::InstallOptionChanged(InstallOption::ToIoStore(v))),
                
                Space::with_height(Length::Fixed(30.0)),
                
                // Buttons
                row![
                    button("Cancel")
                        .on_press(Message::InstallCancelled)
                        .padding(10),
                    Space::with_width(Length::Fixed(10.0)),
                    button("Install")
                        .on_press(Message::InstallConfirmed)
                        .padding(10),
                ]
                .spacing(10),
            ]
            .spacing(10)
            .padding(30)
            .align_items(Alignment::Start)
        )
        .width(Length::Fixed(500.0))
        .style(|_theme: &Theme| {
            container::Appearance {
                background: Some(Color::from_rgb(0.15, 0.15, 0.18).into()),
                border: Border {
                    width: 2.0,
                    color: Color::from_rgb(0.4, 0.7, 1.0),
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        });
        
        // Modal overlay
        container(
            container(content)
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| {
            container::Appearance {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
                ..Default::default()
            }
        })
        .into()
    }
}
