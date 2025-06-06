use iced::{
    widget::{column, container, image, text, vertical_space},
    Alignment, Element, Length,
};

use crate::frontend::title_overlay::{self, supporting_text};

#[derive(Debug, Clone)]
pub struct PaymentRequired {
    logo_handle: iced::widget::image::Handle,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PaymentRequiredMessage {
    // StartSession,
}

#[derive(Debug, Clone)]
pub enum PaymentRequiredEffect {
    StartSession,
}

impl PaymentRequired {
    pub fn with_error(error: String) -> Self {
        Self {
            logo_handle: iced::widget::image::Handle::from_bytes(
                include_bytes!("../../../assets/caj-logo.png").to_vec(),
            ),
            error: Some(error),
        }
    }

    pub fn new() -> Self {
        Self {
            logo_handle: iced::widget::image::Handle::from_bytes(
                include_bytes!("../../../assets/caj-logo.png").to_vec(),
            ),
            error: None,
        }
    }

    pub fn update(&mut self, _message: PaymentRequiredMessage) -> Option<PaymentRequiredEffect> {
        Some(PaymentRequiredEffect::StartSession)
    }

    pub fn view<'a>(&'a self) -> Element<'a, PaymentRequiredMessage> {
        title_overlay::faded_overlay(
            column([
                vertical_space().height(Length::Fill).into(),
                container(image(self.logo_handle.clone()).content_fit(iced::ContentFit::Contain))
                    .center_x(Length::Fill)
                    .into(),
                text(format!("{} Photo Booth", env!("EVENT_NAME")))
                    .style(|theme: &iced::Theme| text::Style {
                        color: Some(theme.extended_palette().background.base.text),
                    })
                    .size(42)
                    .wrapping(text::Wrapping::None)
                    .align_x(Alignment::Center)
                    .width(Length::Fill)
                    .into(),
                match &self.error {
                    Some(error_message) => supporting_text(error_message).into(),
                    None => vertical_space().height(0.0).into(),
                },
                supporting_text("Press [SPACE] to start taking photos.").into(),
                vertical_space().height(12.0).into(),
            ]),
            false,
        )
    }
}
