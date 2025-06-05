use iced::{
    widget::{column, container, image, vertical_space},
    Element, Length,
};

use crate::frontend::title_overlay::{supporting_text, title_overlay, title_text};

#[derive(Debug, Clone)]
pub struct PaymentRequired;

#[derive(Debug, Clone)]
pub enum PaymentRequiredMessage {
    StartSession,
}

#[derive(Debug, Clone)]
pub enum PaymentRequiredEffect {
    StartSession,
}

impl PaymentRequired {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        _message: PaymentRequiredMessage,
    ) -> (Self, Option<PaymentRequiredEffect>) {
        (Self, Some(PaymentRequiredEffect::StartSession))
    }

    pub fn view<'a>(&self, error: Option<&'a str>) -> Element<'a, PaymentRequiredMessage> {
        title_overlay(
            column([
                container(
                    image(iced::widget::image::Handle::from_bytes(
                        include_bytes!("../../../assets/banner.png").to_vec(),
                    ))
                    .width(Length::Fill)
                    .content_fit(iced::ContentFit::Contain),
                )
                .width(Length::Fill)
                .max_width(500)
                .into(),
                title_text("Welcome to the Photo Booth!").into(),
                match error {
                    Some(error_message) => supporting_text(error_message).into(),
                    None => supporting_text("Press [SPACE] to start taking photos.").into(),
                },
                vertical_space().height(12.0).into(),
            ]),
            true,
        )
    }
}
