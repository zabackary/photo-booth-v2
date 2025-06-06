use anim::Animation;
use iced::{
    widget::{stack, text},
    Element,
};
use image::RgbaImage;

use super::{animations, status_overlay, PHOTO_COUNT};

#[derive(Debug)]
pub struct CapturePhotos {
    current: usize,
    state: CapturePhotosState,
    countdown_timeline: Option<anim::Timeline<animations::countdown_circle::AnimationState>>,
    capture_timeline: Option<anim::Timeline<animations::capture_flash::AnimationState>>,
    preview_timeline: Option<anim::Timeline<animations::capture_preview::AnimationState>>,
    captured_handle: Option<iced::widget::image::Handle>,
}

#[derive(Debug, Clone)]
enum CapturePhotosState {
    Countdown { current: usize },
    Capture,
    Preview,
}

#[derive(Debug, Clone)]
pub enum CapturePhotosMessage {
    Tick,
}

#[derive(Debug, Clone)]
pub enum CapturePhotosEffect {
    CaptureStill,
    PhotosComplete { photos: Vec<RgbaImage> },
}

impl CapturePhotos {
    pub fn new() -> Self {
        Self {
            current: 0,
            state: CapturePhotosState::Countdown { current: 3 },
            countdown_timeline: Some(animations::countdown_circle::animation().begin_animation()),
            capture_timeline: None,
            preview_timeline: None,
            captured_handle: None,
        }
    }

    pub fn update(
        &mut self,
        message: CapturePhotosMessage,
        captured_photos: &mut Vec<RgbaImage>,
    ) -> Option<CapturePhotosEffect> {
        match message {
            CapturePhotosMessage::Tick => {
                match &mut self.state {
                    CapturePhotosState::Countdown { current } => {
                        if let Some(timeline) = &mut self.countdown_timeline {
                            if timeline.update().is_completed() {
                                *current -= 1;
                                if *current == 0 {
                                    self.state = CapturePhotosState::Capture;
                                    self.countdown_timeline = None;
                                    log::trace!("Start animation");
                                    self.capture_timeline = Some(
                                        animations::capture_flash::animation().begin_animation(),
                                    );
                                    return Some(CapturePhotosEffect::CaptureStill);
                                } else {
                                    self.countdown_timeline = Some(
                                        animations::countdown_circle::animation().begin_animation(),
                                    );
                                }
                            }
                        }
                    }
                    CapturePhotosState::Capture => {
                        if let Some(timeline) = &mut self.capture_timeline {
                            if timeline.update().is_completed() {
                                let last_photo = captured_photos
                                    .last()
                                    .expect("capture didn't complete")
                                    .clone();
                                self.state = CapturePhotosState::Preview;
                                self.capture_timeline = None;
                                self.preview_timeline = Some(
                                    animations::capture_preview::animation().begin_animation(),
                                );
                                self.captured_handle =
                                    Some(iced::widget::image::Handle::from_rgba(
                                        last_photo.width(),
                                        last_photo.height(),
                                        last_photo.into_raw(),
                                    ));
                            }
                        }
                    }
                    CapturePhotosState::Preview => {
                        if let Some(timeline) = &mut self.preview_timeline {
                            if timeline.update().is_completed() {
                                self.current += 1;
                                if self.current < PHOTO_COUNT {
                                    self.state = CapturePhotosState::Countdown { current: 3 };
                                    self.preview_timeline = None;
                                    self.captured_handle = None;
                                    self.countdown_timeline = Some(
                                        animations::countdown_circle::animation().begin_animation(),
                                    );
                                } else {
                                    // All photos captured, return effect
                                    let photos = captured_photos.drain(..).collect();
                                    return Some(CapturePhotosEffect::PhotosComplete { photos });
                                }
                            }
                        }
                    }
                }
                None
            }
        }
    }

    pub fn view(&self) -> Element<CapturePhotosMessage> {
        stack([
            status_overlay::status_overlay(
                text(format!("photo {} of {PHOTO_COUNT}", self.current + 1)).size(24),
            )
            .into(),
            match &self.state {
                CapturePhotosState::Countdown { current } => {
                    if let Some(timeline) = &self.countdown_timeline {
                        animations::countdown_circle::view(*current, timeline.value()).into()
                    } else {
                        text("Starting...").into()
                    }
                }
                CapturePhotosState::Capture => {
                    if let Some(timeline) = &self.capture_timeline {
                        animations::capture_flash::view(timeline.value()).into()
                    } else {
                        text("Capturing...").into()
                    }
                }
                CapturePhotosState::Preview => {
                    if let (Some(timeline), Some(handle)) =
                        (&self.preview_timeline, &self.captured_handle)
                    {
                        animations::capture_preview::view(handle, timeline.value()).into()
                    } else {
                        text("Processing...").into()
                    }
                }
            },
        ])
        .into()
    }
}
