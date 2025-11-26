// Iced Migration - Work in Progress
// This file will replace main.rs once complete
// Original egui version backed up in main_egui_backup.rs

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Row},
    window, Application, Command, Element, Length, Settings, Theme,
};

fn main() -> iced::Result {
    RepakApp::run(Settings {
        window: window::Settings {
            size: iced::Size::new(1366.0, 768.0),
            min_size: Some(iced::Size::new(1100.0, 650.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

// Message types for the application
#[derive(Debug, Clone)]
enum Message {
    // Placeholder messages - will expand as we port features
    Loaded,
    SearchChanged(String),
    ModSelected(usize),
    SettingsPressed,
    SettingsClosed,
}

// Main application state
struct RepakApp {
    // UI state
    search_query: String,
    selected_mod: Option<usize>,
    show_settings: bool,
    
    // Business logic will be added here
    // manager: RepakModManager,
}

impl Application for RepakApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                search_query: String::new(),
                selected_mod: None,
                show_settings: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Repak GUI - Iced Migration WIP")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded => Command::none(),
            Message::SearchChanged(query) => {
                self.search_query = query;
                Command::none()
            }
            Message::ModSelected(index) => {
                self.selected_mod = Some(index);
                Command::none()
            }
            Message::SettingsPressed => {
                self.show_settings = true;
                Command::none()
            }
            Message::SettingsClosed => {
                self.show_settings = false;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("Repak GUI - Iced Migration").size(24),
            text("Work in Progress - Basic skeleton").size(14),
            row![
                text("Search:"),
                text_input("Search mods...", &self.search_query)
                    .on_input(Message::SearchChanged)
                    .padding(10)
                    .width(Length::Fixed(300.0)),
            ]
            .spacing(10),
            button("Open Settings").on_press(Message::SettingsPressed),
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}
