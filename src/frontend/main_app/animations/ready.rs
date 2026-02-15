use std::time::{Duration, Instant};

use iced::{
    Animation, Border, Length,
    widget::{Container, column, container, space, text},
};

use super::LENGTH_DIVISOR;

pub const ANIMATION_LENGTH: Duration = Duration::from_millis(3000 / LENGTH_DIVISOR);

const TEXT_SIZE: f32 = 60.0;

#[derive(Debug, Clone)]
pub struct ReadyAnimation {
    progress: Animation<f32>,
}

impl ReadyAnimation {
    pub fn new() -> Self {
        let progress = Animation::new(0.0)
            .duration(ANIMATION_LENGTH)
            .easing(iced::animation::Easing::EaseOut)
            .go(1.0, Instant::now());

        Self { progress }
    }

    pub fn finished(&self) -> bool {
        !self.progress.is_animating(Instant::now())
    }

    pub fn view<Message: 'static>(&self) -> Container<'static, Message> {
        let t = self.progress.value().clamp(0.0, 1.0);

        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

        let (opacity, text_size, offset) = if t < 0.4 {
            let tt = t / 0.4;
            (
                lerp(0.0, 1.0, tt),
                lerp(TEXT_SIZE * 0.8, TEXT_SIZE, tt),
                lerp(200.0, 0.0, tt),
            )
        } else if t < 0.8 {
            (1.0, TEXT_SIZE, 0.0)
        } else {
            let tt = (t - 0.8) / 0.2;
            (
                lerp(1.0, 0.0, tt),
                lerp(TEXT_SIZE, TEXT_SIZE * 0.8, tt),
                lerp(0.0, 200.0, tt),
            )
        };

        container(column([
            space().height(offset).into(),
            container(text("Ready?").size(text_size))
                .style(move |theme: &iced::Theme| container::Style {
                    text_color: Some(
                        theme
                            .extended_palette()
                            .primary
                            .weak
                            .text
                            .scale_alpha(opacity),
                    ),
                    background: Some(
                        theme
                            .extended_palette()
                            .primary
                            .weak
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
                })
                .padding(24)
                .into(),
        ]))
        .center(Length::Fill)
    }
}
