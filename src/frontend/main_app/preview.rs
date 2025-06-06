use iced::{
    widget::{column, vertical_space},
    Element,
};

use crate::frontend::title_overlay::{supporting_text, title_text};

#[derive(Debug, Clone)]
pub struct Preview;

#[derive(Debug, Clone)]
pub enum PreviewMessage {}

#[derive(Debug, Clone)]
pub enum PreviewEffect {}

impl Preview {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: PreviewMessage) -> Option<PreviewEffect> {
        None
    }

    pub fn view(&self) -> Element<PreviewMessage> {
        iced::widget::container(
            iced::widget::container(column([
                title_text("Get ready to take your pictures")
                    .width(iced::Length::Shrink)
                    .into(),
                vertical_space().height(12.0).into(),
                supporting_text("Press [SPACE] to start when you're ready.")
                    .width(iced::Length::Shrink)
                    .into(),
            ]))
            .padding(12)
            .width(iced::Length::Shrink)
            .style(move |theme: &iced::Theme| iced::widget::container::Style {
                text_color: Some(theme.extended_palette().primary.weak.text),
                background: Some(
                    theme
                        .extended_palette()
                        .primary
                        .weak
                        .color
                        .scale_alpha(0.7)
                        .into(),
                ),
                border: iced::Border {
                    radius: 24.0.into(),
                    ..Default::default()
                },
                shadow: Default::default(),
            }),
        )
        .center(iced::Length::Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::End)
        .padding(24)
        .into()
    }
}
