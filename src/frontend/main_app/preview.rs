use iced::{
    widget::{column, vertical_space},
    Element,
};

use crate::frontend::title_overlay::{supporting_text, title_overlay, title_text};

#[derive(Debug, Clone)]
pub struct Preview;

#[derive(Debug, Clone)]
pub enum PreviewMessage {}

#[derive(Debug, Clone)]
pub enum PreviewEffect {}

impl Preview {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: PreviewMessage) -> (Self, Option<PreviewEffect>) {
        (self.clone(), None)
    }

    pub fn view(&self) -> Element<PreviewMessage> {
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
