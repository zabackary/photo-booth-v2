use iced::{
    widget::{
        button, column, container, horizontal_space, image, row, text, text_input, vertical_space,
    },
    Border, Color, Element, Length, Padding,
};
use regex::Regex;

use crate::frontend::title_overlay::{full_title_overlay, supporting_text, title_text};

const QR_CODE_QUIET_ZONE: usize = 2;
pub const QR_CODE_VERSION: iced::widget::qr_code::Version =
    iced::widget::qr_code::Version::Normal(5);
const QR_CODE_SIDE_LENGTH: usize = QR_CODE_QUIET_ZONE * 2 + (5 * 4 + 17);

const EMAIL_REGEX: &str = r"^([a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+)@([a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$";

#[derive(Debug, Clone)]
pub struct EmailEntry {
    emails: Vec<String>,
    email_validation_triggered: bool,
}

#[derive(Debug, Clone)]
pub enum EmailEntryMessage {
    EmailInput(String),
    EmailSubmit,
}

#[derive(Debug, Clone)]
pub enum EmailEntryEffect {
    Submit { emails: Vec<String> },
}

impl EmailEntry {
    pub fn new() -> Self {
        Self {
            emails: vec![String::new()],
            email_validation_triggered: false,
        }
    }

    pub fn get_emails(&self) -> Vec<String> {
        self.emails
            .iter()
            .filter(|e| !e.is_empty())
            .cloned()
            .collect()
    }

    pub fn update(&mut self, message: EmailEntryMessage) -> (Self, Option<EmailEntryEffect>) {
        match message {
            EmailEntryMessage::EmailInput(input) => {
                if let Some(first) = self.emails.get_mut(0) {
                    *first = input;
                }
                self.email_validation_triggered = false;
                (self.clone(), None)
            }
            EmailEntryMessage::EmailSubmit => {
                let empty_string = String::new();
                let current_email = self.emails.get(0).unwrap_or(&empty_string).trim();

                if current_email.is_empty() {
                    // Submit with current emails (excluding empty ones)
                    let emails = self.get_emails();
                    return (self.clone(), Some(EmailEntryEffect::Submit { emails }));
                }

                // Validate email
                let regex = Regex::new(EMAIL_REGEX).unwrap();
                if !regex.is_match(current_email) {
                    self.email_validation_triggered = true;
                    return (self.clone(), None);
                }

                // Add email to list and clear input
                self.emails.push(current_email.to_string());
                self.emails[0] = String::new();
                self.email_validation_triggered = false;

                (self.clone(), None)
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        _upload_handle: Option<&'a impl Clone>,
        qr_code_data: Option<&'a iced::widget::qr_code::Data>,
        strip_handle: Option<&iced::widget::image::Handle>,
    ) -> Element<'a, EmailEntryMessage> {
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
                                        if let Some(ref qr_code_data) = qr_code_data {
                                            container(
                                                iced::widget::qr_code(qr_code_data).cell_size(8).style(|_|iced::widget::qr_code::Style {
                                                    background: Color::WHITE,
                                                    cell: Color::BLACK
                                                })
                                            ).width((QR_CODE_SIDE_LENGTH * 8) as u16).height((QR_CODE_SIDE_LENGTH * 8) as u16).padding(8).into()
                                        } else {
                                            container(
                                                column([
                                                    iced::widget::text("Uploading and generating code...").into()
                                                ])
                                                .align_x(iced::Alignment::Center)
                                                .spacing(8)
                                            ).style(|_| container::background(Color::WHITE)).padding(8).center((QR_CODE_SIDE_LENGTH * 8) as u16).into()
                                        }
                                    ]).spacing(16).padding(4).align_x(iced::Alignment::Center)
                                } else {
                                    let email_elements: Vec<Element<EmailEntryMessage>> = self.emails.iter().skip(1).map(|email| {
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
                                    }).collect();
                                    column(email_elements).push(vertical_space()).spacing(8)
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
                        .width(Length::Fill)
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
                if let Some(strip_handle) = strip_handle {
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
