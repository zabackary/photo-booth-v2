use iced::Element;

use super::animations;

#[derive(Debug)]
pub struct CapturePhotosPrepare {
    animation: animations::ready::ReadyAnimation,
}

#[derive(Debug, Clone)]
pub enum CapturePhotosPrepareMessage {
    Animate,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CapturePhotosPrepareAction {
    Task(iced::Task<CapturePhotosPrepareMessage>),
    Complete,
    None,
}

impl CapturePhotosPrepare {
    pub fn new() -> Self {
        Self {
            animation: animations::ready::ReadyAnimation::new(),
        }
    }

    pub fn update(&mut self, message: CapturePhotosPrepareMessage) -> CapturePhotosPrepareAction {
        match message {
            CapturePhotosPrepareMessage::Animate => {
                if self.animation.finished() {
                    CapturePhotosPrepareAction::Complete
                } else {
                    CapturePhotosPrepareAction::None
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, CapturePhotosPrepareMessage> {
        self.animation.view().into()
    }

    pub fn subscription(&self) -> iced::Subscription<CapturePhotosPrepareMessage> {
        iced::window::frames().map(|_| CapturePhotosPrepareMessage::Animate)
    }
}
