use iced::{
    Element,
    widget::{column, row, space},
};

use crate::frontend::title_overlay::{dialog_overlay, styled_button, supporting_text};

#[derive(Debug, Clone)]
pub struct Discard {}

#[derive(Debug, Clone)]
pub enum DiscardMessage {
    Cancel,
    Confirm,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DiscardAction {
    Task(iced::Task<DiscardMessage>),
    Cancel,
    Confirm,
    None,
}

impl Discard {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: DiscardMessage) -> DiscardAction {
        match message {
            DiscardMessage::Cancel => DiscardAction::Cancel,
            DiscardMessage::Confirm => DiscardAction::Confirm,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<DiscardMessage> {
        iced::keyboard::listen().filter_map(|event| {
            if let iced::keyboard::Event::KeyReleased { key, .. } = event {
                match key {
                    iced::keyboard::Key::Named(
                        iced::keyboard::key::Named::Space | iced::keyboard::key::Named::Enter,
                    ) => Some(DiscardMessage::Confirm),
                    iced::keyboard::Key::Character(char) if char == "y" => {
                        Some(DiscardMessage::Confirm)
                    }
                    // Escape, n, or q cancels the discard
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                        Some(DiscardMessage::Cancel)
                    }
                    iced::keyboard::Key::Character(char) if char == "n" || char == "q" => {
                        Some(DiscardMessage::Cancel)
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    pub fn view(&self) -> Element<'_, DiscardMessage> {
        dialog_overlay(
            "Are you sure you want to discard your photos?",
            column([
                supporting_text("The discard key was pressed. If you continue, your photos will be deleted and you'll start over.").into(),
                space().height(12.0).into(),
                row([
                    styled_button("n", "to cancel")
                    .on_press(DiscardMessage::Cancel)
                    .into(),
                    styled_button("y", "to confirm")
                    .on_press(DiscardMessage::Confirm)
                    .into(),
                ]).spacing(12.0).into()
            ])
            .align_x(iced::Alignment::Center),
        )
    }
}
