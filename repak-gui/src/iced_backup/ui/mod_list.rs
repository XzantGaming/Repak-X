// Mod list UI component
// Displays the list of mods with search and filtering

use iced::{Element};
use iced::widget::{column, text};
use crate::messages::Message;

pub struct ModList;

impl ModList {
    pub fn view() -> Element<'static, Message> {
        column![
            text("Mod List Component"),
            text("Will display mods here"),
        ]
        .spacing(10)
        .into()
    }
}
