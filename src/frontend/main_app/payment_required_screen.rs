// PaymentRequiredScreen.rs
// Encapsulated screen for PaymentRequired state
use iced::{Element, widget::{container, column, vertical_space, text, image}, Length, Color, Border, Padding};
use super::{title_overlay, title_text, supporting_text};

pub struct PaymentRequiredScreen {
    pub error: Option<String>,
    pub logo_handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
pub enum PaymentRequiredMessage {
    KeyReleased(crate::KeyMessage),
}

impl PaymentRequiredScreen {
    pub fn update(&mut self, message: PaymentRequiredMessage) {
        // Handle key events if needed
    }

    pub fn view<'a>(&'a self) -> Element<'a, PaymentRequiredMessage> {
        title_overlay(
            container(
                container(
                    column([
                        vertical_space().height(6).into(),
                        image(self.logo_handle.clone())
                            .width(800)
                            .height(300)
                            .content_fit(iced::ContentFit::Contain)
                            .into(),
                        vertical_space().height(6).into(),
                        text("Press [SPACE] to get started.")
                            .size(36)
                            .into(),
                        vertical_space().height(12).into(),
                        text(env!("PRIVACY_NOTE"))
                            .size(18)
                            .into(),
                        vertical_space().height(12).into(),
                        if let Some(error_message) = &self.error {
                            column([
                                vertical_space().height(12).into(),
                                container(column([
                                    text(error_message)
                                        .shaping(iced::widget::text::Shaping::Advanced)
                                        .size(16)
                                        .into()
                                ]))
                                .style(|theme: &iced::Theme| container::Style {
                                    border: iced::Border::default().rounded(4.0).color(
                                        theme.extended_palette().danger.strong.color,
                                    ).width(1.0),
                                    background: Some(
                                        theme.extended_palette().danger.weak.color.into(),
                                    ),
                                    text_color: Some(
                                        theme.extended_palette().danger.weak.text,
                                    ),
                                    ..Default::default()
                                })
                                .padding(8)
                                .into(),
                            ]).into()
                        } else {
                            iced::widget::Space::new(0, 0).into()
                        },
                    ])
                    .align_x(iced::Alignment::Center),
                )
                .max_width(780)
                .padding(18)
                .style(|theme: &iced::Theme| container::Style {
                    border: iced::Border::default().rounded(28),
                    background: Some(theme.extended_palette().primary.base.color.into()),
                    text_color: Some(Color::from_rgb8(0xff, 0xff, 0xff)),
                    ..Default::default()
                }),
            )
            .center(Length::Fill),
            false,
        )
        .into()
    }
}
