use std::time::{Duration, Instant};

use anim::{Animation, easing};
use iced::{
    Element,
    widget::{ProgressBar, column, stack, text},
};
use image::RgbaImage;

use super::{animations, status_overlay};

const PROGRESS_BAR_ANIMATION_LENGTH: std::time::Duration = std::time::Duration::from_millis(800);

#[derive(Debug)]
pub struct CapturePhotos {
    state: CapturePhotosState,
    captured_photos: Vec<RgbaImage>,
    capture_num: usize,

    progress_timeline_girth: anim::Timeline<f32>,
    progress_timeline: anim::Timeline<f32>,

    manager: crate::backend::manager::BackendManager,
    config: &'static crate::config::Config,
}

#[derive(Debug)]
enum CapturePhotosState {
    Countdown {
        count: usize,
        animation: animations::countdown_circle::CountdownCircleAnimation,
    },
    Capture {
        animation: animations::capture_flash::CaptureFlashAnimation,
        capture_complete: bool,
    },
    Preview {
        animation: animations::capture_preview::CapturePreviewAnimation,
        captured_handle: iced::widget::image::Handle,
    },
}

#[derive(Debug, Clone)]
pub enum CapturePhotosMessage {
    Animate,
    CaptureComplete(RgbaImage),
}

#[derive(Debug)]
pub enum CapturePhotosAction {
    PhotosComplete { photos: Vec<RgbaImage> },
    Task(iced::Task<CapturePhotosMessage>),
    None,
}

impl CapturePhotos {
    pub fn new(
        manager: crate::backend::manager::BackendManager,
        config: &'static crate::config::Config,
    ) -> Self {
        Self {
            captured_photos: Vec::new(),
            capture_num: 0,
            manager,
            config,
            progress_timeline_girth: anim::Options::new(0.0, 16.0)
                .duration(PROGRESS_BAR_ANIMATION_LENGTH)
                .easing(easing::cubic_ease().mode(easing::EasingMode::Out))
                .build()
                .chain(anim::builder::constant(16.0, Duration::from_hours(1)))
                .begin_animation(),
            progress_timeline: anim::Options::new(0.0, 0.0)
                .easing(easing::cubic_ease().mode(easing::EasingMode::InOut))
                .begin_animation(),
            state: CapturePhotosState::Countdown {
                count: 3,
                animation: animations::countdown_circle::CountdownCircleAnimation::new(),
            },
        }
    }

    pub fn update(&mut self, message: CapturePhotosMessage) -> CapturePhotosAction {
        self.progress_timeline.update_with_time(Instant::now());
        self.progress_timeline_girth
            .update_with_time(Instant::now());

        match message {
            CapturePhotosMessage::CaptureComplete(photo) => {
                log::trace!("Photo capture complete");
                self.captured_photos.push(photo);
                if let CapturePhotosState::Capture {
                    capture_complete,
                    animation,
                } = &mut self.state
                {
                    if animation.finished() {
                        // animation done already, move on
                        let last_photo = self.captured_photos.last().unwrap().clone();
                        self.state = CapturePhotosState::Preview {
                            animation: animations::capture_preview::CapturePreviewAnimation::new(
                                last_photo.width() as f32 / last_photo.height() as f32,
                            ),
                            captured_handle: iced::widget::image::Handle::from_rgba(
                                last_photo.width(),
                                last_photo.height(),
                                last_photo.into_raw(),
                            ),
                        };
                    } else {
                        // signal to animation that capture is ready
                        *capture_complete = true;
                    }
                }
                CapturePhotosAction::None
            }
            CapturePhotosMessage::Animate => {
                match &mut self.state {
                    CapturePhotosState::Countdown {
                        count: current,
                        animation,
                    } => {
                        if animation.finished() {
                            *current -= 1;
                            if *current == 0 {
                                self.state = CapturePhotosState::Capture {
                                    animation:
                                        animations::capture_flash::CaptureFlashAnimation::new(),
                                    capture_complete: false,
                                };
                                log::trace!("Start animation and photo capture");
                                let camera_manager = self.manager.camera_manager.clone();
                                CapturePhotosAction::Task(iced::Task::perform(
                                    async move {
                                        camera_manager
                                            .frame_still()
                                            .await
                                            .expect("failed to capture photo!")
                                    },
                                    CapturePhotosMessage::CaptureComplete,
                                ))
                            } else {
                                // Restart animation for next countdown number
                                *animation =
                                    animations::countdown_circle::CountdownCircleAnimation::new();
                                CapturePhotosAction::None
                            }
                        } else {
                            CapturePhotosAction::None
                        }
                    }
                    CapturePhotosState::Capture {
                        animation,
                        capture_complete,
                    } => {
                        if animation.finished() {
                            if *capture_complete {
                                let last_photo = self
                                    .captured_photos
                                    .last()
                                    .expect("capture didn't complete")
                                    .clone();
                                self.state = CapturePhotosState::Preview {
                                    animation:
                                        animations::capture_preview::CapturePreviewAnimation::new(
                                            last_photo.width() as f32 / last_photo.height() as f32,
                                        ),
                                    captured_handle: iced::widget::image::Handle::from_rgba(
                                        last_photo.width(),
                                        last_photo.height(),
                                        last_photo.into_raw(),
                                    ),
                                };
                            } else {
                                log::warn!(
                                    "Capture animation finished but capture not complete, waiting..."
                                );
                            }
                            CapturePhotosAction::None
                        } else {
                            CapturePhotosAction::None
                        }
                    }
                    CapturePhotosState::Preview {
                        animation,
                        captured_handle: _,
                    } => {
                        if animation.finished() {
                            if self.captured_photos.len() < self.config.photos_per_strip {
                                self.state = CapturePhotosState::Countdown {
                                    // reset countdown for next photo
                                    count: 3,
                                    animation:
                                        animations::countdown_circle::CountdownCircleAnimation::new(
                                        ),
                                };
                                self.capture_num += 1;
                                let to = (self.capture_num as f32)
                                    / (self.config.photos_per_strip as f32 - 1.0);
                                self.progress_timeline =
                                    anim::Options::new(self.progress_timeline.value(), to)
                                        .easing(
                                            easing::cubic_ease().mode(easing::EasingMode::InOut),
                                        )
                                        .duration(PROGRESS_BAR_ANIMATION_LENGTH)
                                        .build()
                                        .chain(anim::builder::constant(to, Duration::from_hours(1)))
                                        .begin_animation();
                                CapturePhotosAction::None
                            } else {
                                // All photos captured, return effect
                                let photos = self.captured_photos.drain(..).collect();
                                CapturePhotosAction::PhotosComplete { photos }
                            }
                        } else {
                            CapturePhotosAction::None
                        }
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, CapturePhotosMessage> {
        stack([
            column([
                status_overlay::status_overlay(
                    text(format!(
                        "photo {} of {}",
                        self.capture_num + 1,
                        self.config.photos_per_strip
                    ))
                    .size(24),
                )
                .into(),
                ProgressBar::new(0.0..=1.0, self.progress_timeline.value())
                    .girth(self.progress_timeline_girth.value())
                    .into(),
            ])
            .into(),
            match &self.state {
                CapturePhotosState::Countdown { animation, count } => animation.view(*count).into(),
                CapturePhotosState::Capture { animation, .. } => animation.view().into(),
                CapturePhotosState::Preview {
                    animation,
                    captured_handle,
                } => animation.view(captured_handle).into(),
            },
        ])
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<CapturePhotosMessage> {
        iced::window::frames().map(|_| CapturePhotosMessage::Animate)
    }
}
