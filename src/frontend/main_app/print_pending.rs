use std::time::{Duration, Instant};

use iced::{
    Animation, Element, Length,
    widget::{column, container, progress_bar, space},
};

use crate::frontend::{
    loading_spinners,
    title_overlay::{supporting_text, title_overlay, title_text},
};

#[derive(Debug)]
pub struct PrintPending {
    progress_animation_start: f32,
    progress_animation: Animation<bool>,
    finished: bool,
}

#[derive(Debug, Clone)]
pub enum PrintPendingMessage {
    Animate,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PrintPendingAction {
    Complete,
    Task(iced::Task<PrintPendingMessage>),
    None,
}

impl PrintPending {
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

    pub fn update(&mut self, message: PrintPendingMessage) -> PrintPendingAction {
        match message {
            PrintPendingMessage::Animate => {
                if self.finished && !self.progress_animation.is_animating(Instant::now()) {
                    PrintPendingAction::Complete
                } else {
                    PrintPendingAction::None
                }
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<PrintPendingMessage> {
        iced::window::frames().map(|_| PrintPendingMessage::Animate)
    }

    pub fn view(&self) -> Element<'_, PrintPendingMessage> {
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
                title_text("Waiting for a printer to be ready").into(),
                supporting_text("Hold tight...").into(),
                space().height(12.0).into(),
                progress_bar(0.0..=1.0, value).girth(8.0).into(),
            ]),
            false,
        )
    }
}
