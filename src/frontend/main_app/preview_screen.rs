// PreviewScreen.rs
// Encapsulated screen for Preview state
use iced::{Element, widget::{column, vertical_space}, Length};
use super::{title_overlay, title_text, supporting_text};

pub struct PreviewScreen;

#[derive(Debug, Clone)]
pub enum PreviewMessage {
    KeyReleased(crate::KeyMessage),
}

impl PreviewScreen {
    pub fn update(&mut self, _message: PreviewMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, PreviewMessage> {
        title_overlay(
            column([
                title_text("Get ready to take your pictures").into(),
                supporting_text("Press [SPACE] to start when you're ready.").into(),
                vertical_space().height(12.0).into(),
            ]),
            true,
        )
    }
}
