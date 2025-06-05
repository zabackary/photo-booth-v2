// CapturePhotosPrepareScreen.rs
// Encapsulated screen for CapturePhotosPrepare state
use iced::{Element};
use super::animations;

pub struct CapturePhotosPrepareScreen {
    pub ready_timeline: anim::Timeline<animations::ready::AnimationState>,
}

#[derive(Debug, Clone)]
pub enum CapturePhotosPrepareMessage {
    Tick,
}

impl CapturePhotosPrepareScreen {
    pub fn update(&mut self, _message: CapturePhotosPrepareMessage) {}

    pub fn view<'a>(&'a self) -> Element<'a, CapturePhotosPrepareMessage> {
        animations::ready::view(self.ready_timeline.value()).into()
    }
}
