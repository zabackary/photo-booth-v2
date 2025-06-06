use std::time::Duration;

use anim::Animation;
use iced::{
    widget::{column, progress_bar, row, text, vertical_space},
    Alignment, Element,
};

use crate::frontend::{
    loading_spinners,
    title_overlay::{supporting_text, title_overlay, title_text},
};

use super::{animations, status_overlay};

#[derive(Debug)]
pub struct RenderedPreview {
    progress_timeline: anim::Timeline<f32>,
    template_preview_timeline: anim::Timeline<animations::upsell_templates::AnimationState>,
    is_completed: bool,
    pub strip_handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
pub enum RenderedPreviewMessage {
    Tick,
    Skip,
}

#[derive(Debug, Clone)]
pub enum RenderedPreviewEffect {
    Complete,
}

impl RenderedPreview {
    pub fn new(strip_handle: iced::widget::image::Handle) -> Self {
        Self {
            progress_timeline: anim::Options::new(0.0, 1.0)
                .duration(Duration::from_millis(
                    animations::upsell_templates::ANIMATION_LENGTH,
                ))
                .easing(anim::easing::linear())
                .begin_animation(),
            template_preview_timeline: animations::upsell_templates::animation().begin_animation(),
            is_completed: false,
            strip_handle,
        }
    }

    pub fn update(&mut self, message: RenderedPreviewMessage) -> Option<RenderedPreviewEffect> {
        match message {
            RenderedPreviewMessage::Tick => {
                self.template_preview_timeline.update();
                let progress_completed = self.progress_timeline.update().is_completed();
                let template_completed = self.template_preview_timeline.update().is_completed();

                if progress_completed && template_completed && !self.is_completed {
                    self.is_completed = true;
                    Some(RenderedPreviewEffect::Complete)
                } else {
                    None
                }
            }
            RenderedPreviewMessage::Skip => {
                self.progress_timeline = anim::Options::new(self.progress_timeline.value(), 1.0)
                    .duration(Duration::from_millis(1000))
                    .easing(anim::easing::cubic_ease().mode(anim::easing::EasingMode::InOut))
                    .begin_animation();
                None
            }
        }
    }

    pub fn view(&self) -> Element<RenderedPreviewMessage> {
        iced::widget::stack([
            title_overlay(
                column([
                    animations::upsell_templates::view(
                        &self.strip_handle,
                        self.template_preview_timeline.value(),
                    )
                    .into(),
                    title_text("Your photos are ready!").into(),
                    supporting_text("On the next screen, enter your emails.").into(),
                    vertical_space().height(12.0).into(),
                    progress_bar(0.0..=1.0, self.progress_timeline.value())
                        .height(4.0)
                        .into(),
                ]),
                false,
            )
            .into(),
            status_overlay::status_overlay(
                row([
                    loading_spinners::Circular::new()
                        .size(30.0)
                        .bar_height(3.0)
                        .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                        .into(),
                    text("Uploading photos in the background...").into(),
                ])
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .into(),
        ])
        .into()
    }
}
