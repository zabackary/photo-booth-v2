use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use anim::{Animatable, Animation, Timeline, easing};
use iced::{
    Color, Length, Rotation,
    widget::{Container, column, container, image, image::Handle, responsive, space},
};

use super::LENGTH_DIVISOR;

const IMAGE_RELATIVE_SIZE: f32 = 0.8;
pub const ANIMATION_LENGTH: Duration = Duration::from_millis(3000 / LENGTH_DIVISOR);

#[derive(Debug, Clone, Copy, Animatable)]
struct AnimationState {
    opacity: f32,
    offset_scale: f32,
    width_scale: f32,
    rotation_radians: f32,
    background_opacity: f32,
}

fn animation() -> impl anim::Animation<Item = AnimationState> {
    anim::builder::key_frames([
        anim::KeyFrame::new(AnimationState {
            opacity: 0.0,
            offset_scale: 1.0,
            width_scale: 0.95,
            rotation_radians: 0.0,
            background_opacity: 0.0,
        })
        .by_percent(0.0),
        anim::KeyFrame::new(AnimationState {
            opacity: 1.0,
            offset_scale: 0.0,
            width_scale: 1.0,
            rotation_radians: 0.0,
            background_opacity: 0.6,
        })
        .easing(easing::cubic_ease().mode(easing::EasingMode::Out))
        .by_percent(0.2),
        anim::KeyFrame::new(AnimationState {
            opacity: 1.0,
            offset_scale: 0.0,
            width_scale: 1.0,
            rotation_radians: 0.0,
            background_opacity: 0.6,
        })
        .by_percent(0.8),
        anim::KeyFrame::new(AnimationState {
            opacity: 0.8,
            offset_scale: 0.0,
            width_scale: 0.0,
            rotation_radians: 0.7,
            background_opacity: 0.0,
        })
        .easing(easing::cubic_ease().mode(easing::EasingMode::In))
        .by_duration(ANIMATION_LENGTH),
    ])
}

#[derive(Debug)]
pub struct CapturePreviewAnimation {
    timeline: RefCell<Timeline<AnimationState>>,
    photo_aspect_ratio: f32,
}

impl CapturePreviewAnimation {
    pub fn new(photo_aspect_ratio: f32) -> Self {
        Self {
            timeline: RefCell::new(animation().begin_animation()),
            photo_aspect_ratio,
        }
    }

    pub fn finished(&self) -> bool {
        self.timeline.borrow().status().is_completed()
    }

    pub fn view<'a, Message: 'a>(&'a self, handle: &'a Handle) -> Container<'a, Message> {
        container(responsive(move |size| {
            self.timeline.borrow_mut().update_with_time(Instant::now());
            let AnimationState {
                opacity,
                offset_scale,
                width_scale,
                rotation_radians,
                background_opacity,
            } = self.timeline.borrow().value();

            let image_width = width_scale * size.width * IMAGE_RELATIVE_SIZE;
            let image_height = image_width / self.photo_aspect_ratio;

            let remaining_vertical_space = size.height - image_height;

            container(column([
                space()
                    .height(remaining_vertical_space * offset_scale)
                    .into(),
                image(handle)
                    .opacity(opacity)
                    .width(image_width)
                    .height(image_height)
                    .rotation(Rotation::Solid(rotation_radians.into()))
                    .into(),
            ]))
            .style(move |_| container::Style {
                background: Some(Color::BLACK.scale_alpha(background_opacity).into()),
                ..Default::default()
            })
            .center(Length::Fill)
            .into()
        }))
    }
}
