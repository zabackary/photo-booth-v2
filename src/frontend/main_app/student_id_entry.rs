use iced::{
    widget::{
        button, column, container, horizontal_space, image, row, text, text_input, vertical_space,
    },
    Border, Element, Length, Padding,
};

use crate::frontend::title_overlay::{full_title_overlay, supporting_text, title_text};

#[derive(Debug, Clone)]
pub struct StudentIDEntry {
    student_id: String,
    pub strip_handle: Option<iced::widget::image::Handle>,
    pub upload_handle: Option<String>, // Generic upload handle ID
    pub emails: Vec<String>,           // Store emails from previous step
}

#[derive(Debug, Clone)]
pub enum StudentIDEntryMessage {
    StudentIDInput(String),
    StudentIDSubmit,
}

#[derive(Debug, Clone)]
pub enum StudentIDEntryEffect {
    Submit { student_id: String },
}

impl StudentIDEntry {
    pub fn new() -> Self {
        Self {
            student_id: String::new(),
            strip_handle: None,
            upload_handle: None,
            emails: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        message: StudentIDEntryMessage,
    ) -> (Self, Option<StudentIDEntryEffect>) {
        match message {
            StudentIDEntryMessage::StudentIDInput(input) => {
                self.student_id = input;
                (self.clone(), None)
            }
            StudentIDEntryMessage::StudentIDSubmit => {
                let student_id = self.student_id.trim().to_string();
                (
                    self.clone(),
                    Some(StudentIDEntryEffect::Submit { student_id }),
                )
            }
        }
    }

    pub fn view(&self) -> Element<StudentIDEntryMessage> {
        full_title_overlay(
            row([
                column([
                    title_text("Enter your student ID")
                        .width(Length::Shrink)
                        .into(),
                    supporting_text("This is optional but helps us track usage.")
                        .width(Length::Shrink)
                        .into(),
                    vertical_space().height(12.0).into(),
                    container(
                        column([
                            row([
                                text_input("Student ID (optional)", &self.student_id)
                                    .on_input(StudentIDEntryMessage::StudentIDInput)
                                    .on_submit(StudentIDEntryMessage::StudentIDSubmit)
                                    .size(24)
                                    .id("student_id_input")
                                    .style(|theme: &iced::Theme, status| {
                                        let mut normal = text_input::default(theme, status);
                                        normal.border.radius = 6.0.into();
                                        normal
                                    })
                                    .padding(Padding {
                                        bottom: 10.0,
                                        left: 16.0,
                                        right: 16.0,
                                        top: 10.0,
                                    })
                                    .into(),
                                horizontal_space().width(6.0).into(),
                                button(text("Continue").size(24))
                                    .style(|theme: &iced::Theme, status| {
                                        let mut normal = button::primary(theme, status);
                                        normal.border.radius = 999.0.into();
                                        normal
                                    })
                                    .padding(Padding {
                                        bottom: 10.0,
                                        left: 24.0,
                                        right: 24.0,
                                        top: 10.0,
                                    })
                                    .on_press(StudentIDEntryMessage::StudentIDSubmit)
                                    .padding(10)
                                    .into(),
                            ])
                            .into(),
                            vertical_space().height(12.0).into(),
                            container(
                                column([text(
                                    "Press [Enter] to continue without entering a student ID.",
                                )
                                .size(18)
                                .into()])
                                .align_x(iced::Alignment::Center),
                            )
                            .into(),
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
                        .padding(30),
                    )
                    .style(|theme: &iced::Theme| container::Style {
                        background: Some(
                            theme
                                .extended_palette()
                                .background
                                .base
                                .color
                                .scale_alpha(0.8)
                                .into(),
                        ),
                        border: Border::default().rounded(iced::border::Radius {
                            bottom_left: 24.0,
                            bottom_right: 0.0,
                            top_left: 24.0,
                            top_right: 0.0,
                        }),
                        ..Default::default()
                    })
                    .into()
                } else {
                    iced::widget::Space::new(0, 0).into()
                },
            ])
            .align_y(iced::Alignment::Center),
        )
    }
}
