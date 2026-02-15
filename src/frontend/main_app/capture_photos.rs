use iced::{
    Element,
    widget::{stack, text},
};
use image::RgbaImage;

use super::{animations, status_overlay};

#[derive(Debug)]
pub struct CapturePhotos {
    state: CapturePhotosState,
    captured_photos: Vec<RgbaImage>,

    manager: crate::backend::manager::BackendManager,
    config: &'static crate::config::Config,
}

#[derive(Debug, Clone)]
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
            manager,
            config,
            state: CapturePhotosState::Countdown {
                count: 3,
                animation: animations::countdown_circle::CountdownCircleAnimation::new(),
            },
        }
    }

    pub fn update(&mut self, message: CapturePhotosMessage) -> CapturePhotosAction {
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
            status_overlay::status_overlay(
                text(format!(
                    "photo {} of {}",
                    self.captured_photos.len() + 1,
                    self.config.photos_per_strip
                ))
                .size(24),
            )
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
