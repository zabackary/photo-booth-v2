use std::time::{Duration, Instant};

use iced::{
    Animation, Element, Length,
    widget::{column, container, progress_bar, space},
};

use crate::frontend::{
    components::{supporting_text, title_overlay, title_text},
    loading_spinners,
};

#[derive(Debug)]
pub struct Emailing {
    progress_animation_start: f32,
    progress_animation: Animation<bool>,
    finished: bool,
}

#[derive(Debug, Clone)]
pub enum EmailingMessage {
    Animate,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum EmailingAction {
    Complete,
    Task(iced::Task<EmailingMessage>),
    None,
}

impl Emailing {
    pub fn new() -> Self {
        Self {
            progress_animation_start: 0.0,
            progress_animation: Animation::new(false)
                .duration(Duration::from_millis(500))
                .easing(iced::animation::Easing::EaseInOut)
                .go(true, std::time::Instant::now()),
            finished: false,
        }
    }

    pub fn finish(&mut self) {
        self.finished = true;
        self.progress_animation_start = self.progress_animation.interpolate(
            self.progress_animation_start,
            1.0,
            std::time::Instant::now(),
        );
        self.progress_animation = Animation::new(false)
            .duration(Duration::from_millis(500))
            .easing(iced::animation::Easing::EaseInOut)
            .go(true, std::time::Instant::now());
    }

    pub fn update(&mut self, message: EmailingMessage) -> EmailingAction {
        match message {
            EmailingMessage::Animate => {
                if self.finished && !self.progress_animation.is_animating(Instant::now()) {
                    EmailingAction::Complete
                } else {
                    EmailingAction::None
                }
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<EmailingMessage> {
        iced::window::frames().map(|_| EmailingMessage::Animate)
    }

    pub fn view(&self) -> Element<'_, EmailingMessage> {
        let value = self.progress_animation.interpolate(
            self.progress_animation_start,
            if self.finished { 1.0 } else { 0.8 },
            std::time::Instant::now(),
        );
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
                title_text("Sending your photos to you...").into(),
                supporting_text("If you don't receive your email in a few minutes, check your spam folder then contact us!").into(),
                space().height(12.0).into(),
                progress_bar(0.0..=1.0, value).girth(8.0).into(),
            ]),
            false,
        )
    }
}
