use std::time::{Duration, Instant};

use iced::{
    Animation, Border, Length,
    widget::{Container, container, text},
};

use super::LENGTH_DIVISOR;

pub const ANIMATION_LENGTH: Duration = Duration::from_millis(1000 / LENGTH_DIVISOR);

const MIN_TEXT_SIZE: f32 = f32::MIN_POSITIVE;
const TEXT_SIZE: f32 = 60.0;

#[derive(Debug, Clone)]
pub struct CountdownCircleAnimation {
    progress: Animation<bool>,
}

impl CountdownCircleAnimation {
    pub fn new() -> Self {
        let progress = Animation::new(false)
            .duration(ANIMATION_LENGTH)
            .easing(iced::animation::Easing::EaseOutCubic)
            .go(true, Instant::now());

        Self { progress }
    }

    pub fn finished(&self) -> bool {
        !self.progress.is_animating(Instant::now())
    }

    pub fn view<Message: 'static>(&self, value: usize) -> Container<'static, Message> {
        let t = self.progress.interpolate(0.0, 1.0, Instant::now());

        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

        let (opacity, text_size) = if t < 0.4 {
            let tt = t / 0.4;
            (lerp(0.0, 1.0, tt), lerp(MIN_TEXT_SIZE, TEXT_SIZE, tt))
        } else if t < 0.8 {
            (1.0, TEXT_SIZE)
        } else {
            let tt = (t - 0.8) / 0.2;
            (lerp(1.0, 0.0, tt), lerp(TEXT_SIZE, MIN_TEXT_SIZE, tt))
        };

        container(
            container(text(format!("{value}")).size(text_size))
                .padding(24)
                .style(move |theme: &iced::Theme| container::Style {
                    text_color: Some(
                        theme
                            .extended_palette()
                            .primary
                            .strong
                            .text
                            .scale_alpha(opacity),
                    ),
                    background: Some(
                        theme
                            .extended_palette()
                            .primary
                            .strong
                            .color
                            .scale_alpha(opacity)
                            .into(),
                    ),
                    border: Border {
                        radius: 9999.0.into(),
                        ..Default::default()
                    },
                    shadow: Default::default(),
                    snap: true,
                }),
        )
        .center(Length::Fill)
    }
}
