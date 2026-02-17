use iced::{
    Element,
    widget::{column, space},
};

use crate::frontend::title_overlay::{supporting_text, title_overlay, title_text};

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

#[derive(Debug, Clone)]
pub enum ErrorMessage {
    Close,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorAction {
    Task(iced::Task<ErrorMessage>),
    Complete,
    None,
}

impl Error {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn update(&mut self, message: ErrorMessage) -> ErrorAction {
        match message {
            ErrorMessage::Close => ErrorAction::Complete,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ErrorMessage> {
        iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyReleased {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
                ..
            } => Some(ErrorMessage::Close),
            _ => None,
        })
    }

    pub fn view(&self) -> Element<'_, ErrorMessage> {
        title_overlay(
            column([
                title_text("Uh oh. Something went wrong.").into(),
                supporting_text(&self.message).into(),
                supporting_text("Press [SPACE] to close this message.").into(),
                space().height(12.0).into(),
            ]),
            false,
        )
    }
}
