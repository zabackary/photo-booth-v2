use iced::{
    Element,
    widget::{column, space},
};

use crate::frontend::components::{
    dialog_overlay, styled_button, supporting_text, supporting_text_small,
};

#[derive(Debug, Clone)]
pub struct Error {
    message_friendly: String,
    message_internal: String,
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
    pub fn new(friendly: String, internal: String) -> Self {
        Self {
            message_friendly: friendly,
            message_internal: format!("error code: {}", internal),
        }
    }

    pub fn update(&mut self, message: ErrorMessage) -> ErrorAction {
        match message {
            ErrorMessage::Close => ErrorAction::Complete,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ErrorMessage> {
        iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyReleased {
                key:
                    iced::keyboard::Key::Named(
                        iced::keyboard::key::Named::Space | iced::keyboard::key::Named::Enter,
                    ),
                ..
            } => Some(ErrorMessage::Close),
            _ => None,
        })
    }

    pub fn view(&self) -> Element<'_, ErrorMessage> {
        dialog_overlay(
            "Uh oh. Something went wrong.",
            column([
                supporting_text(&self.message_friendly).into(),
                space().height(12.0).into(),
                supporting_text_small(&self.message_internal).into(),
                space().height(12.0).into(),
                styled_button("SPACE", "to close message")
                    .on_press(ErrorMessage::Close)
                    .into(),
            ])
            .align_x(iced::Alignment::Center),
        )
    }
}
