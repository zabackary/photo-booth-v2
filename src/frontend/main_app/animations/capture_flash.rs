use std::time::{Duration, Instant};

use iced::{
    Animation, Color, Length,
    widget::{Container, Space, container},
};

use super::LENGTH_DIVISOR;

pub const ANIMATION_LENGTH: Duration = Duration::from_millis(400 / LENGTH_DIVISOR);

#[derive(Debug, Clone)]
pub struct CaptureFlashAnimation {
    opacity: Animation<f32>,
}

impl CaptureFlashAnimation {
    /// Create a new capture flash animation and start it immediately
    pub fn new() -> Self {
        let opacity = Animation::new(1.0)
            .duration(ANIMATION_LENGTH)
            .easing(iced::animation::Easing::EaseOut)
            .go(0.0, Instant::now());
        Self { opacity }
    }

    /// Whether the animation has completed
    pub fn finished(&self) -> bool {
        !self.opacity.is_animating(Instant::now())
    }

    pub fn view<'a, Message: 'a>(&'a self) -> Container<'a, Message> {
        container(Space::new())
            .style(move |_| container::Style {
                background: Some(Color::WHITE.scale_alpha(self.opacity.value()).into()),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
    }
}
