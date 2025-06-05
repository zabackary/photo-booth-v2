// CapturePhotosScreen.rs
// Encapsulated screen for CapturePhotos state
use iced::{Element};
use super::animations;

pub struct CapturePhotosScreen {
    pub current: usize,
    pub state: super::CapturePhotosState,
}

#[derive(Debug, Clone)]
pub enum CapturePhotosMessage {
    Tick,
    CaptureStill,
}

impl CapturePhotosScreen {
    pub fn update(&mut self, _message: CapturePhotosMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, CapturePhotosMessage> {
        use super::status_overlay;
        use iced::{widget::{text, stack}, Length};
        use super::PHOTO_COUNT;
        stack([
            status_overlay::status_overlay(text(format!("photo {} of {}", self.current + 1, PHOTO_COUNT)).size(24)).into(),
            match &self.state {
                super::CapturePhotosState::Countdown { current, countdown_timeline } =>
                    animations::countdown_circle::view(*current, countdown_timeline.value()).into(),
                super::CapturePhotosState::Capture { capture_timeline } =>
                    animations::capture_flash::view(capture_timeline.value()).into(),
                super::CapturePhotosState::Preview { preview_timeline, captured_handle } =>
                    animations::capture_preview::view(captured_handle, preview_timeline.value()).into(),
            }
        ])
    }
}
