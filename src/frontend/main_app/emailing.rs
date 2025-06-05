use iced::{
    widget::{column, container, progress_bar, vertical_space},
    Element, Length,
};

use super::loading_spinners;
use crate::frontend::title_overlay::{supporting_text, title_overlay, title_text};

#[derive(Debug, Clone)]
pub struct Emailing;

#[derive(Debug, Clone)]
pub enum EmailingMessage {
    Complete,
}

#[derive(Debug, Clone)]
pub enum EmailingEffect {
    Complete,
}

impl Emailing {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, message: EmailingMessage) -> (Self, Option<EmailingEffect>) {
        match message {
            EmailingMessage::Complete => (Self, Some(EmailingEffect::Complete)),
        }
    }

    pub fn view(&self, progress: f32) -> Element<EmailingMessage> {
        title_overlay(
            column([
                container(
                    loading_spinners::Circular::new()
                        .size(90.0)
                        .bar_height(9.0)
                        .easing(&loading_spinners::easing::STANDARD_DECELERATE),
                )
                .center(Length::Fill)
                .into(),
                title_text("We're processing your photos now.").into(),
                supporting_text(
                    "If you entered your email, check your inbox to download your pictures.",
                )
                .into(),
                vertical_space().height(12.0).into(),
                progress_bar(0.0..=1.0, progress).height(8.0).into(),
            ]),
            false,
        )
    }
}
