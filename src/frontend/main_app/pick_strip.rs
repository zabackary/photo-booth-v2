use std::time::{Duration, Instant};

use iced::{
    Animation, Element, Length, Vector,
    widget::{column, container, float, image as image_widget, image::Handle, row, space},
};

use crate::frontend::{
    main_app::animations::LENGTH_DIVISOR,
    title_overlay::{supporting_text, title_overlay, title_text},
};

#[derive(Debug)]
pub struct PickStrip {
    selection: usize,
    items: Vec<PickStripItem>,
}

#[derive(Debug, Clone)]
pub enum PickStripMessage {
    Animate,
    Previous,
    Next,
    Finish,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PickStripAction {
    Complete { selection: usize },
    Task(iced::Task<PickStripMessage>),
    None,
}

impl PickStrip {
    pub fn new(strips: Vec<image::RgbaImage>) -> Self {
        Self {
            selection: 0,
            items: strips
                .into_iter()
                .enumerate()
                .map(|(i, strip)| {
                    PickStripItem::new(
                        Handle::from_rgba(strip.width(), strip.height(), strip.into_raw()),
                        i == 0,
                    )
                })
                .collect(),
        }
    }

    pub fn update(&mut self, message: PickStripMessage) -> PickStripAction {
        match message {
            PickStripMessage::Animate => PickStripAction::None,
            PickStripMessage::Previous => {
                self.items[self.selection].toggle_selected(false);
                if self.selection > 0 {
                    self.selection -= 1;
                } else {
                    self.selection = self.items.len() - 1;
                }
                self.items[self.selection].toggle_selected(true);
                PickStripAction::None
            }
            PickStripMessage::Next => {
                self.items[self.selection].toggle_selected(false);
                if self.selection < self.items.len() - 1 {
                    self.selection += 1;
                } else {
                    self.selection = 0;
                }
                self.items[self.selection].toggle_selected(true);
                PickStripAction::None
            }
            PickStripMessage::Finish => PickStripAction::Complete {
                selection: self.selection,
            },
        }
    }

    pub fn subscription(&self) -> iced::Subscription<PickStripMessage> {
        iced::Subscription::batch([
            iced::keyboard::listen().filter_map(|event| {
                if let iced::keyboard::Event::KeyReleased { key, .. } = event {
                    match key {
                        iced::keyboard::Key::Named(
                            iced::keyboard::key::Named::ArrowLeft
                            | iced::keyboard::key::Named::ArrowUp,
                        ) => Some(PickStripMessage::Previous),
                        iced::keyboard::Key::Named(
                            iced::keyboard::key::Named::ArrowRight
                            | iced::keyboard::key::Named::ArrowDown,
                        ) => Some(PickStripMessage::Next),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                        | iced::keyboard::Key::Named(iced::keyboard::key::Named::Space) => {
                            Some(PickStripMessage::Finish)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }),
            if self.items.iter().any(|item| item.is_animating()) {
                iced::window::frames().map(|_| PickStripMessage::Animate)
            } else {
                iced::Subscription::none()
            },
        ])
    }

    pub fn view(&self) -> Element<'_, PickStripMessage> {
        title_overlay(
            column([
                container(
                    row(self.items.iter().map(|item| item.view()))
                        .spacing(42.0)
                        .padding(56.0)
                        .height(Length::Fill),
                )
                .center_x(Length::Fill)
                .into(),
                title_text("Pick your style!").into(),
                supporting_text(
                    "Use the arrow keys to choose your style and press Enter to select it.",
                )
                .into(),
                space().height(12.0).into(),
            ]),
            false,
        )
    }
}

/// A single photo strip in the screen
#[derive(Debug)]
struct PickStripItem {
    image: Handle,
    selected_animation: Animation<bool>,
}

impl PickStripItem {
    fn new(image: Handle, selected: bool) -> Self {
        Self {
            image,
            selected_animation: Animation::new(selected)
                .duration(Duration::from_millis(500 / LENGTH_DIVISOR))
                .easing(iced::animation::Easing::EaseInOut),
        }
    }

    fn is_animating(&self) -> bool {
        self.selected_animation.is_animating(Instant::now())
    }

    fn toggle_selected(&mut self, selected: bool) {
        self.selected_animation.go_mut(selected, Instant::now());
    }

    fn view(&self) -> Element<'_, PickStripMessage> {
        let now = Instant::now();
        float(
            image_widget(self.image.clone())
                .opacity(self.selected_animation.interpolate(0.6, 1.0, now)),
        )
        .scale(self.selected_animation.interpolate(1.0, 1.1, now))
        .style(move |_theme| float::Style {
            shadow: iced::Shadow {
                color: iced::Color::BLACK
                    .scale_alpha(self.selected_animation.interpolate(0.0, 1.0, now)),
                offset: Vector::new(0.0, self.selected_animation.interpolate(0.0, 4.0, now)),
                blur_radius: self.selected_animation.interpolate(0.0, 10.0, now),
            },
            ..float::Style::default()
        })
        .into()
    }
}
