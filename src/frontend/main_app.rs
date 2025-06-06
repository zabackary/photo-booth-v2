use std::time::Duration;

use anim::Animation;
use iced::{
    widget::{image::Handle, row, text},
    Alignment, ContentFit, Element, Length, Task,
};
use image::RgbaImage;

use crate::{backend::render_take::render_take, AppPage, KeyMessage, PhotoBoothMessage};

use super::{
    camera_feed::{CameraFeed, CameraFeedOptions},
    loading_spinners,
};

mod animations;
mod capture_photos;
mod email_entry;
mod emailing;
mod payment_required;
mod preview;
mod rendered_preview;
mod status_overlay;
mod student_id_entry;

use capture_photos::{CapturePhotos, CapturePhotosEffect, CapturePhotosMessage};
use email_entry::{EmailEntry, EmailEntryEffect, EmailEntryMessage};
use emailing::{Emailing, EmailingEffect, EmailingMessage};
use payment_required::{PaymentRequired, PaymentRequiredEffect, PaymentRequiredMessage};
use preview::{Preview, PreviewMessage};
use rendered_preview::{RenderedPreview, RenderedPreviewEffect, RenderedPreviewMessage};
use student_id_entry::{StudentIDEntry, StudentIDEntryEffect, StudentIDEntryMessage};

const PHOTO_ASPECT_RATIO: f32 = 3.0 / 2.0;
const PHOTO_COUNT: usize = 4;

enum MainAppState {
    PaymentRequired(PaymentRequired),
    Preview(Preview),
    CapturePhotosPrepare {
        ready_timeline: anim::Timeline<animations::ready::AnimationState>,
    },
    CapturePhotos(CapturePhotos),
    RenderedPreview(RenderedPreview),
    EmailEntry(EmailEntry),
    Emailing(Emailing),
    StudentIDEntry(StudentIDEntry),
}

#[derive(Debug, Clone)]
pub enum MainAppMessage<S: crate::backend::servers::ServerBackend + 'static> {
    Camera(super::camera_feed::CameraMessage),
    Tick,
    KeyReleased(KeyMessage),
    CaptureStill,
    Uploaded(Result<S::UploadHandle, String>),
    Emailed(Result<(), String>),
    OtherKeyPress,

    EmailEntry(EmailEntryMessage),
    StudentIDEntry(StudentIDEntryMessage),
    PaymentRequired(PaymentRequiredMessage),
    Emailing(EmailingMessage),
    CapturePhotos(CapturePhotosMessage),
    Preview(PreviewMessage),
    RenderedPreview(RenderedPreviewMessage),
}

pub struct MainApp<
    C: crate::backend::cameras::CameraBackend + 'static,
    S: crate::backend::servers::ServerBackend + 'static,
> {
    feed: CameraFeed<C::Camera>,
    state: MainAppState,
    captured_photos: Vec<RgbaImage>,
    previews: Vec<iced::widget::image::Handle>,
    strip: Option<RgbaImage>,
    logo_handle: Handle,
    pub new_page: Option<Box<(AppPage<C, S>, Task<PhotoBoothMessage<C, S>>)>>,
}

impl<
        C: crate::backend::cameras::CameraBackend + 'static,
        S: crate::backend::servers::ServerBackend + 'static,
    > MainApp<C, S>
{
    pub fn new(feed: CameraFeed<C::Camera>) -> (Self, Task<MainAppMessage<S>>) {
        (
            Self {
                feed,
                state: MainAppState::PaymentRequired(PaymentRequired::new()),
                new_page: None,
                captured_photos: Vec::with_capacity(PHOTO_COUNT),
                previews: Vec::with_capacity(PHOTO_COUNT),
                logo_handle: Handle::from_bytes(include_bytes!("../../assets/banner.png").to_vec()),
                strip: None,
            },
            Task::none(),
        )
    }

    pub fn update(
        &mut self,
        message: MainAppMessage<S>,
        server_backend: S,
    ) -> Task<MainAppMessage<S>> {
        self.feed.update_options(
            if matches!(
                self.state,
                MainAppState::CapturePhotosPrepare { .. }
                    | MainAppState::CapturePhotos(_)
                    | MainAppState::Preview(_)
            ) {
                CameraFeedOptions {
                    blur: 1.0,
                    aspect_ratio: Some(PHOTO_ASPECT_RATIO),
                    mirror: true,
                    ..Default::default()
                }
            } else {
                CameraFeedOptions {
                    blur: 20.0, // 1/20th the resolution
                    aspect_ratio: None,
                    mirror: true,
                    ..Default::default()
                }
            },
        );

        match message {
            MainAppMessage::Camera(msg) => self.feed.update(msg).map(MainAppMessage::Camera),
            MainAppMessage::CaptureStill => {
                log::debug!("Capturing still image...");
                let image = self
                    .feed
                    .capture_still_sync(CameraFeedOptions {
                        aspect_ratio: Some(PHOTO_ASPECT_RATIO),
                        mirror: true,
                        ..Default::default()
                    })
                    .expect("failed to capture image");
                log::debug!("Image captured successfully.");
                self.captured_photos.push(image);
                Task::none()
            }
            MainAppMessage::Tick => match &mut self.state {
                MainAppState::CapturePhotosPrepare { ready_timeline } => {
                    if ready_timeline.update().is_completed() {
                        self.state = MainAppState::CapturePhotos(CapturePhotos::new());
                    };
                    Task::none()
                }
                MainAppState::CapturePhotos(_capture_photos) => {
                    Task::done(MainAppMessage::CapturePhotos(CapturePhotosMessage::Tick))
                }
                MainAppState::RenderedPreview(_rendered_preview) => Task::done(
                    MainAppMessage::RenderedPreview(RenderedPreviewMessage::Tick),
                ),
                MainAppState::Emailing(emailing) => {
                    if emailing.progress_timeline.update().is_completed() {
                        self.state = MainAppState::PaymentRequired(PaymentRequired::new());
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            MainAppMessage::Uploaded(result) => {
                log::debug!("Upload result received: {:?}", result);
                match result {
                    Ok(res) => {
                        // Update email entry with upload data
                        if let MainAppState::EmailEntry(ref mut email_entry) = self.state {
                            // Store the actual upload handle for later use
                            email_entry.upload_handle = Some(format!("{:?}", res)); // Convert to string representation for now
                            email_entry.set_qr_code_url(server_backend.get_link(res));
                        }
                        Task::none()
                    }
                    Err(err) => {
                        self.state = MainAppState::PaymentRequired(PaymentRequired::new());
                        log::error!("Error uploading photos: {}", err);
                        Task::none()
                    }
                }
            }
            MainAppMessage::KeyReleased(key) => {
                log::debug!("Key released: {:?}", key);
                match &mut self.state {
                    MainAppState::PaymentRequired(_) => match key {
                        KeyMessage::Up => Task::none(),
                        KeyMessage::Down => Task::none(),
                        KeyMessage::Space => {
                            self.state = MainAppState::Preview(Preview::new());
                            Task::none()
                        }
                        KeyMessage::Escape => iced::widget::text_input::focus("email_input"),
                    },
                    MainAppState::Preview(_) => {
                        self.state = MainAppState::CapturePhotosPrepare {
                            ready_timeline: animations::ready::animation().begin_animation(),
                        };
                        Task::none()
                    }
                    MainAppState::RenderedPreview(_) => Task::done(
                        MainAppMessage::RenderedPreview(RenderedPreviewMessage::Skip),
                    ),
                    MainAppState::EmailEntry(_) => iced::widget::text_input::focus("email_input"),
                    MainAppState::StudentIDEntry(_) => {
                        iced::widget::text_input::focus("student_id_input")
                    }
                    _ => Task::none(),
                }
            }
            MainAppMessage::OtherKeyPress => match self.state {
                MainAppState::EmailEntry(_) => iced::widget::text_input::focus("email_input"),
                MainAppState::StudentIDEntry(_) => {
                    iced::widget::text_input::focus("student_id_input")
                }
                _ => Task::none(),
            },
            MainAppMessage::EmailEntry(msg) => match &mut self.state {
                MainAppState::EmailEntry(email_entry) => {
                    let effect = email_entry.update(msg);

                    match effect {
                        Some(EmailEntryEffect::Submit { emails }) => {
                            if emails.is_empty() && email_entry.upload_handle.is_some() {
                                let mut student_id_entry = StudentIDEntry::new();
                                student_id_entry.strip_handle = email_entry.strip_handle.clone();
                                student_id_entry.upload_handle = email_entry.upload_handle.clone();
                                student_id_entry.emails = emails;
                                self.state = MainAppState::StudentIDEntry(student_id_entry);
                                iced::widget::text_input::focus("student_id_input")
                            } else if !emails.is_empty() {
                                // Store emails and proceed to student ID entry
                                let mut student_id_entry = StudentIDEntry::new();
                                student_id_entry.strip_handle = email_entry.strip_handle.clone();
                                student_id_entry.upload_handle = email_entry.upload_handle.clone();
                                student_id_entry.emails = emails;
                                self.state = MainAppState::StudentIDEntry(student_id_entry);
                                iced::widget::text_input::focus("student_id_input")
                            } else {
                                Task::none()
                            }
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::StudentIDEntry(msg) => match &mut self.state {
                MainAppState::StudentIDEntry(student_id_entry) => {
                    let (new_state, task) = student_id_entry.update(msg);
                    *student_id_entry = new_state;

                    match task {
                        Some(StudentIDEntryEffect::Submit { student_id: _ }) => {
                            // We need to get the upload handle from somewhere - let's add it to student_id_entry too
                            // For now, let's assume we have access to a saved upload handle
                            if let Some(_upload_handle_str) = &student_id_entry.upload_handle {
                                // This is a placeholder - we'll need to properly handle the upload handle
                                let emailing = Emailing::new();
                                self.state = MainAppState::Emailing(emailing);
                                self.strip = None;
                                log::trace!("Sending email with photos...");
                                // For now, return a completed task - this needs proper async handling
                                Task::done(MainAppMessage::Emailed(Ok(())))
                            } else {
                                log::error!("No upload handle available for emailing.");
                                self.state = MainAppState::PaymentRequired(PaymentRequired::new());
                                Task::none()
                            }
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::PaymentRequired(msg) => match &mut self.state {
                MainAppState::PaymentRequired(payment_required) => {
                    let task = payment_required.update(msg);

                    match task {
                        Some(PaymentRequiredEffect::StartSession) => {
                            self.state = MainAppState::Preview(Preview::new());
                            Task::none()
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::Preview(msg) => match &mut self.state {
                MainAppState::Preview(preview) => {
                    let (new_state, _effect) = preview.update(msg);
                    *preview = new_state;
                    Task::none()
                }
                _ => Task::none(),
            },
            MainAppMessage::CapturePhotos(msg) => match &mut self.state {
                MainAppState::CapturePhotos(capture_photos) => {
                    let task = capture_photos.update(msg, &mut self.captured_photos);

                    match task {
                        Some(CapturePhotosEffect::CaptureStill) => {
                            Task::done(MainAppMessage::CaptureStill)
                        }
                        Some(CapturePhotosEffect::PhotosComplete { photos }) => {
                            // Process captured photos
                            self.previews.clear();
                            for photo in &photos {
                                self.previews.push(iced::widget::image::Handle::from_rgba(
                                    photo.width(),
                                    photo.height(),
                                    photo.as_raw().clone(),
                                ));
                            }

                            self.strip = Some(render_take(photos.clone()));
                            let strip_handle = Some(Handle::from_rgba(
                                self.strip.as_ref().unwrap().width(),
                                self.strip.as_ref().unwrap().height(),
                                self.strip.as_ref().unwrap().as_raw().clone(),
                            ));

                            let mut rendered_preview = RenderedPreview::new();
                            rendered_preview.strip_handle = strip_handle;
                            self.state = MainAppState::RenderedPreview(rendered_preview);

                            let future = server_backend
                                .upload_photo(self.strip.as_ref().unwrap().clone(), photos);
                            Task::perform(future, |result| {
                                MainAppMessage::Uploaded(result.map_err(|x| x.to_string()))
                            })
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::RenderedPreview(msg) => match &mut self.state {
                MainAppState::RenderedPreview(rendered_preview) => {
                    let task = rendered_preview.update(msg);

                    match task {
                        Some(RenderedPreviewEffect::Complete) => {
                            let mut email_entry = EmailEntry::new();
                            email_entry.strip_handle = rendered_preview.strip_handle.clone();
                            self.state = MainAppState::EmailEntry(email_entry);
                            iced::widget::text_input::focus("email_input")
                        }
                        Some(RenderedPreviewEffect::UploadPhotos { .. }) => {
                            // This effect is not used in this context
                            Task::none()
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::Emailing(msg) => match &mut self.state {
                MainAppState::Emailing(emailing) => {
                    let task = emailing.update(msg);

                    match task {
                        Some(EmailingEffect::Complete) => {
                            self.state = MainAppState::PaymentRequired(PaymentRequired::new());
                            Task::none()
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::Emailed(result) => {
                log::debug!("Email result received: {:?}", result);
                match &mut self.state {
                    MainAppState::Emailing(emailing) => match result {
                        Ok(_) => {
                            emailing.progress_timeline =
                                anim::Options::new(emailing.progress_timeline.value(), 1.0)
                                    .duration(Duration::from_millis(1000))
                                    .easing(
                                        anim::easing::cubic_ease()
                                            .mode(anim::easing::EasingMode::InOut),
                                    )
                                    .begin_animation();
                            Task::none()
                        }
                        Err(err) => {
                            self.state = MainAppState::PaymentRequired(PaymentRequired::new());
                            log::error!("Error emailing photos: {}", err);
                            Task::none()
                        }
                    },
                    _ => Task::none(),
                }
            }
        }
    }

    pub fn view<'a>(&'a self, _server_backend: &'a S) -> Element<'a, MainAppMessage<S>> {
        iced::widget::stack([
            self.feed
                .view()
                .content_fit(
                    if matches!(
                        self.state,
                        MainAppState::CapturePhotosPrepare { .. }
                            | MainAppState::CapturePhotos(_)
                            | MainAppState::Preview(_)
                    ) {
                        ContentFit::Contain
                    } else {
                        ContentFit::Cover
                    },
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            match &self.state {
                MainAppState::PaymentRequired(payment_required) => payment_required
                    .view(None)
                    .map(MainAppMessage::PaymentRequired),
                MainAppState::Preview(preview) => preview.view().map(MainAppMessage::Preview),
                MainAppState::CapturePhotosPrepare { ready_timeline } => {
                    animations::ready::view(ready_timeline.value()).into()
                }
                MainAppState::CapturePhotos(capture_photos) => {
                    capture_photos.view().map(MainAppMessage::CapturePhotos)
                }
                MainAppState::RenderedPreview(rendered_preview) => {
                    rendered_preview.view().map(MainAppMessage::RenderedPreview)
                }
                MainAppState::EmailEntry(email_entry) => iced::widget::stack([
                    email_entry.view().map(MainAppMessage::EmailEntry).into(),
                    if email_entry.upload_handle.is_none() {
                        status_overlay::status_overlay(
                            row([
                                loading_spinners::Circular::new()
                                    .size(30.0)
                                    .bar_height(3.0)
                                    .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                                    .into(),
                                text("Uploading photos in the background...").into(),
                            ])
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .into()
                    } else {
                        "".into()
                    },
                ])
                .into(),
                MainAppState::StudentIDEntry(student_id_entry) => {
                    student_id_entry.view().map(MainAppMessage::StudentIDEntry)
                }
                MainAppState::Emailing(emailing) => emailing.view().map(MainAppMessage::Emailing),
            },
        ])
        .into()
    }
}
