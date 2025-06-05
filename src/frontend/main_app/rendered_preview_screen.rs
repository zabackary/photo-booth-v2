// RenderedPreviewScreen.rs
// Encapsulated screen for RenderedPreview state
use iced::{Element, widget::{column, vertical_space, progress_bar}, Length};
use super::{title_overlay, title_text, supporting_text};
use super::animations;

pub struct RenderedPreviewScreen {
    pub progress_timeline: anim::Timeline<f32>,
    pub template_preview_timeline: anim::Timeline<animations::upsell_templates::AnimationState>,
    pub strip_handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
pub enum RenderedPreviewMessage {
    Tick,
}

impl RenderedPreviewScreen {
    pub fn update(&mut self, _message: RenderedPreviewMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, RenderedPreviewMessage> {
        use super::loading_spinners;
        use iced::{widget::{row}, Color, Border};
        iced::widget::stack([
            title_overlay(
                column([
                    animations::upsell_templates::view(&self.strip_handle, self.template_preview_timeline.value()).into(),
                    title_text("Your photos are ready!").into(),
                    supporting_text("On the next screen, enter your emails.").into(),
                    vertical_space().height(12.0).into(),
                    progress_bar(0.0..=1.0, self.progress_timeline.value())
                        .height(4.0)
                        .into(),
                ]),
                false,
            ).into(),
            super::status_overlay::status_overlay(row([
                loading_spinners::Circular::new()
                    .size(30.0)
                    .bar_height(3.0)
                    .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                    .into(),
                iced::widget::text("Uploading photos in the background...").into()
            ]).spacing(8).align_y(iced::Alignment::Center)).into()
        ])
    }
}
