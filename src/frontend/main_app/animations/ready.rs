use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use anim::{Animatable, Animation, Timeline, easing};
use iced::{
    Border, Length, Vector,
    widget::{Container, container, float, text},
};

use super::LENGTH_DIVISOR;

pub const ANIMATION_LENGTH: Duration = Duration::from_millis(3000 / LENGTH_DIVISOR);

#[derive(Debug, Clone, Copy, Animatable)]
pub struct AnimationState {
    opacity: f32,
    scale: f32,
    offset: f32,
}

const TEXT_SIZE: f32 = 24.0;

pub fn animation() -> impl anim::Animation<Item = AnimationState> {
    anim::builder::key_frames([
        anim::KeyFrame::new(AnimationState {
            opacity: 0.0,
            scale: 1.0, // unfortunately iced doesn't support scale <= 1
            offset: 200.0,
        })
        .by_percent(0.0),
        anim::KeyFrame::new(AnimationState {
            opacity: 1.0,
            scale: 2.0,
            offset: 0.0,
        })
        .easing(easing::cubic_ease().mode(easing::EasingMode::Out))
        .by_percent(0.4),
        anim::KeyFrame::new(AnimationState {
            opacity: 1.0,
            scale: 2.0,
            offset: 0.0,
        })
        .by_percent(0.8),
        anim::KeyFrame::new(AnimationState {
            opacity: 0.0,
            scale: 1.0,
            offset: 200.0,
        })
        .easing(easing::cubic_ease().mode(easing::EasingMode::In))
        .by_duration(ANIMATION_LENGTH),
    ])
}

#[derive(Debug)]
pub struct ReadyAnimation {
    timeline: RefCell<Timeline<AnimationState>>,
}

impl ReadyAnimation {
    pub fn new() -> Self {
        Self {
            timeline: RefCell::new(animation().begin_animation()),
        }
    }

    pub fn finished(&self) -> bool {
        self.timeline.borrow().status().is_completed()
    }

    pub fn view<Message: 'static>(&self) -> Container<'static, Message> {
        self.timeline.borrow_mut().update_with_time(Instant::now());
        let AnimationState {
            opacity,
            scale,
            offset,
        } = self.timeline.borrow().value();

        container(
            float(
                container(text("Get ready!").size(TEXT_SIZE))
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
                    .padding(12),
            )
            .scale(scale)
            .translate(move |_, _| Vector::new(0.0, offset)),
        )
        .center(Length::Fill)
    }
}
