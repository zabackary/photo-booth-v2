// StudentIDEntryScreen.rs
// Encapsulated screen for StudentIDEntry state
use super::{supporting_text, title_overlay, title_text};
use iced::{
    widget::{
        button, column, container, horizontal_space, image, row, text, text_input, vertical_space,
    },
    Element, Length, Padding,
};

pub struct StudentIDEntryScreen {
    pub student_id: String,
    pub strip_handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
pub enum StudentIDEntryMessage {
    StudentIDInput(String),
    StudentIDSubmit,
}

impl StudentIDEntryScreen {
    pub fn update(&mut self, _message: StudentIDEntryMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, StudentIDEntryMessage> {
        title_overlay(
            row([
                column([
                    image(self.strip_handle.clone())
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Contain)
                        .into(),
                    vertical_space().height(12.0).into(),
                    title_text("Would you like it printed?").width(Length::Shrink).into(),
                    supporting_text("We'll deliver two copies of your photo to you next week for only 300 yen, billed to your student account. If you would prefer not to purchase one, press [Enter] without entering anything.").width(Length::Shrink).into(),
                    vertical_space().height(12.0).into(),
                    container(
                        row([
                            text_input(
                                "Enter your student ID",
                                &self.student_id,
                            )
                            .on_input(StudentIDEntryMessage::StudentIDInput)
                            .on_submit(StudentIDEntryMessage::StudentIDSubmit)
                            .style(|theme: &iced::Theme, status| {
                                let mut normal = text_input::default(theme, status);
                                normal.border.radius = 6.0.into();
                                normal
                            })
                            .padding(Padding { bottom: 10.0, left: 16.0, right: 16.0, top: 10.0 })
                            .size(24)
                            .id("student_id_input")
                            .into(),
                            horizontal_space().width(6.0).into(),
                            button(text(if !self.student_id.is_empty() {
                                "[Enter] to confirm"
                            } else {
                                "[Enter] to cancel"
                            })
                            .size(24))
                            .on_press(StudentIDEntryMessage::StudentIDSubmit)
                            .style(|theme: &iced::Theme, status| {
                                let mut normal = button::primary(theme, status);
                                normal.border.radius = 999.0.into();
                                normal
                            })
                            .padding(Padding { bottom: 10.0, left: 24.0, right: 24.0, top: 10.0 })
                            .into(),
                        ])
                    )
                    .max_width(700.0)
                    .into(),
                ])
                .padding(100)
                .align_x(iced::Alignment::Center)
                .width(Length::Fill)
                .into(),
            ])
            .align_y(iced::Alignment::Center),
            false,
        )
    }
}
