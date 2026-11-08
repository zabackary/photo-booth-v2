use iced::{
    Alignment, Background, Color, Element, Length, Padding, Radians,
    gradient::Linear,
    widget::{Container, Text, button, container, row, space, text},
};

use crate::{FONT_BOLD, FONT_ITALIC};

pub fn title_overlay<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    minimize_overlay: bool,
) -> Element<'a, Message> {
    container(content)
        .style(move |theme: &iced::Theme| {
            container::background(Background::Gradient(if minimize_overlay {
                iced::Gradient::Linear(
                    Linear::new(Radians::PI)
                        .add_stop(0.0, Color::TRANSPARENT)
                        .add_stop(0.4, Color::TRANSPARENT)
                        .add_stop(1.0, theme.extended_palette().background.base.color),
                )
            } else {
                iced::Gradient::Linear(
                    Linear::new(Radians::PI)
                        .add_stop(
                            0.0,
                            theme
                                .extended_palette()
                                .background
                                .base
                                .color
                                .scale_alpha(0.7),
                        )
                        .add_stop(
                            0.4,
                            theme
                                .extended_palette()
                                .background
                                .base
                                .color
                                .scale_alpha(0.7),
                        )
                        .add_stop(1.0, theme.extended_palette().background.base.color),
                )
            }))
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::End)
        .align_x(Alignment::Center)
        .into()
}

pub fn full_title_overlay<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content)
        .style(move |theme: &iced::Theme| {
            container::background(
                theme
                    .extended_palette()
                    .background
                    .base
                    .color
                    .scale_alpha(0.7),
            )
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::End)
        .align_x(Alignment::Center)
        .into()
}

pub fn dialog_overlay<'a, Message: 'a>(
    title: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        container(iced::widget::column([
            text(title)
                .font(FONT_BOLD)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                })
                .size(36)
                .wrapping(text::Wrapping::Word)
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .into(),
            space().height(12.0).into(),
            content.into(),
        ]))
        .padding(24)
        .max_width(800.0)
        .style(move |theme: &iced::Theme| {
            container::background(Background::Color(
                theme
                    .extended_palette()
                    .background
                    .strong
                    .color
                    .scale_alpha(0.9),
            ))
            .border(iced::border::rounded(24.0))
        }),
    )
    .padding(24)
    .style(move |theme: &iced::Theme| {
        container::background(Background::Color(
            theme
                .extended_palette()
                .background
                .base
                .color
                .scale_alpha(0.7),
        ))
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Center)
    .align_x(Alignment::Center)
    .into()
}

pub fn title_text(content: &str) -> Text<'_> {
    text(content)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        })
        .font(FONT_BOLD)
        .size(42)
        .wrapping(text::Wrapping::Word)
        .align_x(Alignment::Center)
        .width(Length::Fill)
}

pub fn supporting_text(content: &str) -> Text<'_> {
    text(content)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .base
                    .text
                    .scale_alpha(0.6),
            ),
        })
        .size(32)
        .wrapping(text::Wrapping::Word)
        .align_x(Alignment::Center)
        .width(Length::Fill)
}

pub fn supporting_text_small(content: &str) -> Text<'_> {
    text(content)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .base
                    .text
                    .scale_alpha(0.4),
            ),
        })
        .font(FONT_ITALIC)
        .size(24)
        .wrapping(text::Wrapping::Word)
        .align_x(Alignment::Center)
        .width(Length::Fill)
}

/// A label indicating that a key can be pressed to trigger an action. Styled
/// similar to <key> in HTML.
pub fn keyboard_label<Message>(content: &str) -> Container<'_, Message> {
    container(
        text(content)
            .font(FONT_BOLD)
            .size(18)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
    )
    .style(|theme: &iced::Theme| {
        container::background(Background::Color(
            theme
                .extended_palette()
                .background
                .base
                .color
                .scale_alpha(0.8),
        ))
        .border(iced::border::rounded(4.0))
    })
    .padding(4)
}

/// A button.
///
/// * `key` is the keybind, which should be in all caps
/// * `action` is the action, which might be something like "to save"
pub fn styled_button<'a, Message: 'a>(
    key: &'static str,
    action: &'static str,
) -> iced::widget::Button<'a, Message> {
    button(
        row([
            keyboard_label(key).into(),
            space().width(8.0).into(),
            text(action).size(24).into(),
        ])
        .align_y(iced::Alignment::Center),
    )
    .style(|theme: &iced::Theme, status| {
        let mut normal = button::primary(theme, status);
        normal.border.radius = 999.0.into();
        normal
    })
    .padding(Padding {
        // all around 10?
        bottom: 10.0,
        left: 24.0,
        right: 24.0,
        top: 10.0,
    })
}
