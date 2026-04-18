use std::time::{Duration, Instant};

use anim::Animation as _;
use iced::{
    Border, Color, Element, Length, Padding, Vector,
    widget::{button, column, container, float, image, row, space, text},
};

use crate::frontend::title_overlay::{full_title_overlay, supporting_text, title_text};

fn arrow_slide_animation() -> impl anim::Animation<Item = f32> {
    anim::builder::key_frames([
        anim::KeyFrame::new(0.0).by_percent(0.0),
        anim::KeyFrame::new(1.0)
            .easing(anim::easing::cubic_ease().mode(anim::easing::EasingMode::Out))
            .by_percent(0.4),
        anim::KeyFrame::new(0.0)
            .easing(anim::easing::cubic_ease().mode(anim::easing::EasingMode::Out))
            .by_duration(Duration::from_millis(500)),
    ])
}

#[derive(Debug)]
pub struct CopiesPrompt {
    strip_handle: iced::widget::image::Handle,
    copies: u32,
    min_copies: Option<u32>,
    max_copies: Option<u32>,

    up_arrow_animation: anim::Timeline<f32>,
    down_arrow_animation: anim::Timeline<f32>,
}

#[derive(Debug, Clone)]
pub enum CopiesPromptMessage {
    ChangeCopies(i32),
    ContinuePress,
    Animate,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CopiesPromptAction {
    Complete { copies: u32 },
    Task(iced::Task<CopiesPromptMessage>),
    None,
}

impl CopiesPrompt {
    pub fn new(
        strip_handle: iced::widget::image::Handle,
        default_copies: u32,
        min_copies: Option<u32>,
        max_copies: Option<u32>,
    ) -> Self {
        Self {
            copies: default_copies
                .max(min_copies.unwrap_or(1))
                .min(max_copies.unwrap_or(u32::MAX)),
            strip_handle,
            min_copies,
            max_copies,
            up_arrow_animation: anim::builder::constant(0.0, Duration::ZERO).to_timeline(),
            down_arrow_animation: anim::builder::constant(0.0, Duration::ZERO).to_timeline(),
        }
    }

    pub fn update(&mut self, message: CopiesPromptMessage) -> CopiesPromptAction {
        match message {
            CopiesPromptMessage::ChangeCopies(delta) => {
                let new_copies = (self.copies as i32 + delta)
                    .max(self.min_copies.unwrap_or(1) as i32)
                    .min(self.max_copies.unwrap_or(u32::MAX) as i32) as u32;
                if new_copies > self.copies {
                    self.up_arrow_animation = arrow_slide_animation().begin_animation();
                } else if new_copies < self.copies {
                    self.down_arrow_animation = arrow_slide_animation().begin_animation();
                }
                self.copies = new_copies;
                CopiesPromptAction::None
            }
            CopiesPromptMessage::ContinuePress => CopiesPromptAction::Complete { copies: self.copies },
            CopiesPromptMessage::Animate => {
                self.up_arrow_animation.update_with_time(Instant::now());
                self.down_arrow_animation.update_with_time(Instant::now());
                CopiesPromptAction::None
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<CopiesPromptMessage> {
        iced::Subscription::batch(
            [
                iced::keyboard::listen().filter_map(|event| match event {
                    iced::keyboard::Event::KeyReleased {
                        key:
                            iced::keyboard::Key::Named(
                                iced::keyboard::key::Named::Enter | iced::keyboard::key::Named::Space,
                            ),
                        ..
                    } => Some(CopiesPromptMessage::ContinuePress),
                    iced::keyboard::Event::KeyReleased {
                        key:
                            iced::keyboard::Key::Named(
                                iced::keyboard::key::Named::ArrowUp | iced::keyboard::key::Named::ArrowRight,
                            ),
                        ..
                    } => Some(CopiesPromptMessage::ChangeCopies(1)),
                    iced::keyboard::Event::KeyReleased {
                        key:
                            iced::keyboard::Key::Named(
                                iced::keyboard::key::Named::ArrowDown | iced::keyboard::key::Named::ArrowLeft,
                            ),
                        ..
                    } => Some(CopiesPromptMessage::ChangeCopies(-1)),
                    _ => None,
                }),
                if self.up_arrow_animation.status().is_animating() || self.down_arrow_animation.status().is_animating()
                {
                    iced::window::frames().map(|_| CopiesPromptMessage::Animate)
                } else {
                    iced::Subscription::none()
                },
            ],
        )
    }

    pub fn view<'a>(&'a self) -> Element<'a, CopiesPromptMessage> {
        full_title_overlay(
            row([
                column([
                    title_text("How many copies?").width(Length::Shrink).into(),
                    supporting_text(
                        "Use arrow keys to adjust the number of copies. Additional copies may incur extra costs.",
                    )
                    .width(Length::Shrink)
                    .into(),
                    space().height(12.0).into(),
                    // Up arrow icon
                    float(
                        text("▲")
                            .color(if self.copies >= self.max_copies.unwrap_or(u32::MAX) {
                                Color::BLACK.scale_alpha(0.3)
                            } else {
                                Color::BLACK
                            })
                            .size(42),
                    )
                    .translate(|_, _| Vector::new(0.0, self.up_arrow_animation.value() * -20.0))
                    .scale(self.up_arrow_animation.value() * 0.2 + 1.0)
                    .into(),
                    container(text(self.copies.to_string()).size(64))
                        .center_x(100.0)
                        .center_y(140.0)
                        .style(|theme: &iced::Theme| container::Style {
                            background: Some(theme.extended_palette().primary.base.color.into()),
                            text_color: Some(theme.extended_palette().primary.base.text),
                            border: Border {
                                radius: 12.0.into(),
                                ..Default::default()
                            },
                            shadow: Default::default(),
                            ..Default::default()
                        })
                        .into(),
                    // Down arrow icon
                    float(
                        text("▼")
                            .color(if self.copies <= self.min_copies.unwrap_or(1) {
                                Color::BLACK.scale_alpha(0.3)
                            } else {
                                Color::BLACK
                            })
                            .size(42),
                    )
                    .translate(|_, _| Vector::new(0.0, self.down_arrow_animation.value() * 20.0))
                    .scale(self.down_arrow_animation.value() * 0.2 + 1.0)
                    .into(),
                    space().height(12.0).into(),
                    button(text("Press [Enter] to confirm").size(24))
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
                        .on_press(CopiesPromptMessage::ContinuePress)
                        .padding(10)
                        .into(),
                ])
                .padding(100)
                .align_x(iced::Alignment::Center)
                .width(Length::Fill)
                .into(),
                space().width(12.0).into(),
                container(
                    column([
                        supporting_text("Your photos").width(Length::Shrink).into(),
                        space().height(12.0).into(),
                        image(self.strip_handle.clone())
                            .height(Length::Fill)
                            .content_fit(iced::ContentFit::Contain)
                            .into(),
                    ])
                    .align_x(iced::Alignment::Center)
                    .padding(30),
                )
                .style(|theme: &iced::Theme| container::Style {
                    background: Some(theme.extended_palette().background.base.color.scale_alpha(0.8).into()),
                    border: Border::default().rounded(iced::border::Radius {
                        bottom_left: 24.0,
                        bottom_right: 0.0,
                        top_left: 24.0,
                        top_right: 0.0,
                    }),
                    ..Default::default()
                })
                .into(),
            ])
            .align_y(iced::Alignment::Center),
        )
    }
}
