// EmailingScreen.rs
// Encapsulated screen for Emailing state
use iced::{Element, widget::{column, container, text, progress_bar, vertical_space}, Length};
use super::{title_overlay, title_text, supporting_text};

pub struct EmailingScreen {
    pub progress_timeline: anim::Timeline<f32>,
}

#[derive(Debug, Clone)]
pub enum EmailingMessage {
    Tick,
}

impl EmailingScreen {
    pub fn update(&mut self, _message: EmailingMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, EmailingMessage> {
        use super::loading_spinners;
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
                supporting_text("If you entered your email, check your inbox to download your pictures.").into(),
                vertical_space().height(12.0).into(),
                progress_bar(0.0..=1.0, self.progress_timeline.value())
                    .height(8.0)
                    .into(),
            ]),
            false,
        )
    }
}
