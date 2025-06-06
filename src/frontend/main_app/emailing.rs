use std::time::Duration;

use iced::{
    widget::{column, container, progress_bar, vertical_space},
    Element, Length,
};

use super::loading_spinners;
use crate::frontend::title_overlay::{supporting_text, title_overlay, title_text};

#[derive(Debug)]
pub struct Emailing {
    pub progress_timeline: anim::Timeline<f32>,
}

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
        Self {
            progress_timeline: anim::Options::new(0.0, 0.8)
                .duration(Duration::from_millis(15000))
                .easing(anim::easing::cubic_ease().mode(anim::easing::EasingMode::InOut))
                .begin_animation(),
        }
    }

    pub fn update(&mut self, message: EmailingMessage) -> Option<EmailingEffect> {
        match message {
            EmailingMessage::Complete => Some(EmailingEffect::Complete),
        }
    }

    pub fn view(&self) -> Element<EmailingMessage> {
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
                progress_bar(0.0..=1.0, self.progress_timeline.value())
                    .height(8.0)
                    .into(),
            ]),
            false,
        )
    }
}
