use std::time::{Duration, Instant};

use iced::{
    Animation, Color, Length, Rotation,
    widget::{Container, column, container, image, image::Handle, responsive, space},
};

use super::LENGTH_DIVISOR;

pub const ANIMATION_LENGTH: Duration = Duration::from_millis(3000 / LENGTH_DIVISOR);

#[derive(Debug, Clone)]
pub struct CapturePreviewAnimation {
    progress: Animation<bool>,
    photo_aspect_ratio: f32,
}

impl CapturePreviewAnimation {
    pub fn new(photo_aspect_ratio: f32) -> Self {
        let progress = Animation::new(false)
            .duration(ANIMATION_LENGTH)
            .easing(iced::animation::Easing::EaseInOutCubic)
            .go(true, Instant::now());

        Self {
            progress,
            photo_aspect_ratio,
        }
    }

    pub fn finished(&self) -> bool {
        !self.progress.is_animating(Instant::now())
    }

    pub fn view<'a, Message: 'a>(&'a self, handle: &'a Handle) -> Container<'a, Message> {
        container(responsive(move |size| {
            let t = self.progress.interpolate(0.0, 1.0, Instant::now());

            // keyframe interpolation helper
            let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

            // original keyframes at 0.0, 0.2, 0.8, 1.0
            let (opacity, offset_scale, width_scale, rotation_radians, background_opacity) =
                if t < 0.2 {
                    let tt = t / 0.2;
                    (
                        lerp(0.0, 1.0, tt),
                        lerp(1.0, 0.0, tt),
                        lerp(0.4, 1.0, tt),
                        lerp(0.0, 0.0, tt),
                        lerp(0.0, 0.9, tt),
                    )
                } else if t < 0.8 {
                    (1.0, 0.0, 1.0, 0.0, 0.9)
                } else {
                    let tt = (t - 0.8) / 0.2;
                    (
                        lerp(1.0, 0.8, tt),
                        lerp(0.0, 0.0, tt),
                        lerp(1.0, 0.0, tt),
                        lerp(0.0, 1.0, tt),
                        lerp(0.9, 0.0, tt),
                    )
                };

            let image_width = width_scale * size.width * 0.8;
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
                    .rotation(Rotation::Floating(rotation_radians.into()))
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
