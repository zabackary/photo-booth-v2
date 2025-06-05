// EmailEntryScreen.rs
// Encapsulated screen for EmailEntry state
use super::{full_title_overlay, supporting_text, title_text};
use iced::{
    widget::{
        button, column, container, horizontal_space, image, row, text, text_input, vertical_space,
    },
    Border, Color, Element, Length, Padding,
};

pub struct EmailEntryScreen {
    pub emails: Vec<String>,
    pub email_validation_triggered: bool,
    pub upload_handle_exists: bool,
    pub qr_code_data: Option<iced::widget::qr_code::Data>,
    pub strip_handle: Option<iced::widget::image::Handle>,
}

#[derive(Debug, Clone)]
pub enum EmailEntryMessage {
    EmailInput(String),
    EmailSubmit,
}

impl EmailEntryScreen {
    pub fn update(&mut self, _message: EmailEntryMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, EmailEntryMessage> {
        full_title_overlay(
            row([
                column([
                    title_text("Enter your email addresses").width(Length::Shrink).into(),
                    supporting_text("Start typing to add an email.").width(Length::Shrink).into(),
                    vertical_space().height(12.0).into(),
                    container(
                        column([
                            row([
                                text_input(
                                    "Enter an email",
                                    self.emails.get(0).map(|s| s.as_str()).unwrap_or("")
                                )
                                .on_input(EmailEntryMessage::EmailInput)
                                .on_submit(EmailEntryMessage::EmailSubmit)
                                .size(24)
                                .id("email_input")
                                .style(|theme: &iced::Theme, status| {
                                    let mut normal = text_input::default(theme, status);
                                    normal.border.radius = 6.0.into();
                                    normal
                                })
                                .padding(Padding { bottom: 10.0, left: 16.0, right: 16.0, top: 10.0 })
                                .into(),
                                horizontal_space().width(6.0).into(),
                                button(text(if self.emails.get(0).map_or(false, |e| !e.is_empty()) {
                                    "Press [Enter] to add"
                                } else {
                                    "Press [Enter] to finish"
                                })
                                .size(24))
                                .style(|theme: &iced::Theme, status| {
                                    let mut normal = button::primary(theme, status);
                                    normal.border.radius = 999.0.into();
                                    normal
                                })
                                .padding(Padding { bottom: 10.0, left: 24.0, right: 24.0, top: 10.0 })
                                .on_press(EmailEntryMessage::EmailSubmit)
                                .padding(10)
                                .into(),
                            ])
                            .into(),
                            vertical_space().height(6.0).into(),
                            if self.email_validation_triggered {
                                container(
                                    column([
                                        text("Please check your email address for typos.")
                                            .size(16)
                                            .into(),
                                    ])
                                    .align_x(iced::Alignment::Center)
                                )
                                .style(|theme: &iced::Theme| container::Style {
                                    border: Border::default().rounded(999.0).color(
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
                                .center_x(Length::Fill)
                                .padding(8)
                                .into()
                            } else {
                                iced::widget::Space::new(0, 0).into()
                            },
                            vertical_space().height(6.0).into(),
                            container(
                                if self.emails.len() <= 1 {
                                    column([
                                        text("You can also scan the QR code to download your photos!").into(),
                                        text("If you don't want an email, press [Enter] without entering anything.").into(),
                                        if let Some(ref qr_code_data) = self.qr_code_data {
                                            container(
                                                iced::widget::qr_code(qr_code_data).cell_size(8).style(|_|iced::widget::qr_code::Style {
                                                    background: Color::WHITE,
                                                    cell: Color::BLACK
                                                })
                                            ).width(128).height(128).padding(8).into()
                                        } else {
                                            container(
                                                column([
                                                    iced::widget::text("Uploading and generating code...").into()
                                                ])
                                                .align_x(iced::Alignment::Center)
                                                .spacing(8)
                                            ).style(|_| container::background(Color::WHITE)).padding(8).center(128)
                                        }
                                    ]).spacing(16).padding(4).align_x(iced::Alignment::Center).into()
                                } else {
                                    column(
                                        self.emails.iter().skip(1).map(|email| {
                                            container(
                                                text(email.as_str()).size(24)
                                            ).width(Length::Fill)
                                                .padding(Padding { bottom: 10.0, left: 16.0, right: 16.0, top: 10.0 })
                                                .style(|theme: &iced::Theme| container::Style {
                                                    background: Some(
                                                        theme.extended_palette().background.strong.color.into(),
                                                    ),
                                                    text_color: Some(
                                                        theme.extended_palette().background.strong.text,
                                                    ),
                                                    border: Border::default().rounded(999.0),
                                                    ..Default::default()
                                                }).into()
                                        }).collect()
                                    ).push(vertical_space()).spacing(8).into()
                                },
                            )
                            .padding(12)
                            .style(|theme: &iced::Theme| container::Style {
                                background: Some(
                                    theme.extended_palette().background.base.color.scale_alpha(0.3).into(),
                                ),
                                border: Border::default().rounded(36.0),
                                ..Default::default()
                            })
                            .width(Length::Fill)
                            .center(Length::Fill)
                            .into(),
                            vertical_space().height(12.0).into(),
                            container(
                                column([
                                    text("Make sure your email provider accepts emails from photobooth@caj.ac.jp.")
                                        .size(18)
                                        .into(),
                                ]).align_x(iced::Alignment::Center)
                            ).into()
                        ])
                        .align_x(iced::Alignment::Center),
                    )
                    .center(Length::Fill)
                    .max_width(700.0)
                    .into(),
                ])
                .padding(100)
                .align_x(iced::Alignment::Center)
                .width(Length::Fill)
                .into(),
                horizontal_space().width(12.0).into(),
                if let Some(strip_handle) = &self.strip_handle {
                    container(
                        column([
                            supporting_text("Your photos").width(Length::Shrink).into(),
                            vertical_space().height(12.0).into(),
                            image(strip_handle.clone())
                                .height(Length::Fill)
                                .content_fit(iced::ContentFit::Contain)
                                .into(),
                        ])
                        .align_x(iced::Alignment::Center)
                        .padding(30)
                    ).style(|theme: &iced::Theme| container::Style {
                        background: Some(
                            theme.extended_palette().background.base.color.scale_alpha(0.8).into(),
                        ),
                        border: Border::default().rounded(iced::border::Radius {
                            bottom_left: 24.0,
                            bottom_right: 0.0,
                            top_left: 24.0,
                            top_right: 0.0,
                        }),
                        ..Default::default()
                    }).into()
                } else {
                    iced::widget::Space::new(0, 0).into()
                }
            ])
            .align_y(iced::Alignment::Center),
        )
    }
}
