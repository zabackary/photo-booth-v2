use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use anim::{Animatable, Animation, Timeline, easing};
use iced::{
    Border, Length,
    widget::{Container, container, float, text},
};

use super::LENGTH_DIVISOR;

pub const ANIMATION_LENGTH: Duration = Duration::from_millis(1000 / LENGTH_DIVISOR);

#[derive(Debug, Clone, Copy, Animatable)]
pub struct AnimationState {
    opacity: f32,
    scale: f32,
}

const SIZE: f32 = 70.0; // will be multiplied by 2
const TEXT_SIZE: f32 = 30.0;

pub fn animation() -> impl anim::Animation<Item = AnimationState> {
    anim::builder::key_frames([
        anim::KeyFrame::new(AnimationState {
            opacity: 0.0,
            scale: 1.0, // unfortunately iced doesn't support scale <= 1
        })
        .by_percent(0.0),
        anim::KeyFrame::new(AnimationState {
            opacity: 1.0,
            scale: 2.0,
        })
        .easing(easing::cubic_ease().mode(easing::EasingMode::Out))
        .by_percent(0.4),
        anim::KeyFrame::new(AnimationState {
            opacity: 1.0,
            scale: 2.0,
        })
        .by_percent(0.8),
        anim::KeyFrame::new(AnimationState {
            opacity: 0.0,
            scale: 1.0,
        })
        .easing(easing::cubic_ease().mode(easing::EasingMode::In))
        .by_duration(ANIMATION_LENGTH),
    ])
}

#[derive(Debug)]
pub struct CountdownCircleAnimation {
    timeline: RefCell<Timeline<AnimationState>>,
}

impl CountdownCircleAnimation {
    pub fn new() -> Self {
        Self {
            timeline: RefCell::new(animation().begin_animation()),
        }
    }

    pub fn finished(&self) -> bool {
        self.timeline.borrow().status().is_completed()
    }

    pub fn view<Message: 'static>(&self, value: usize) -> Container<'static, Message> {
        self.timeline.borrow_mut().update_with_time(Instant::now());
        let AnimationState { opacity, scale } = self.timeline.borrow().value();

        container(
            float(
                container(text(format!("{value}")).size(TEXT_SIZE))
                    .center(SIZE)
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
            .scale(scale),
        )
        .center(Length::Fill)
    }
}
