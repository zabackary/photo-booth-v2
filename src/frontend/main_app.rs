use anim::Animation;
use iced::{widget::image::Handle, ContentFit, Element, Length, Task};
use image::RgbaImage;

use crate::{backend::render_take::render_take, AppPage, KeyMessage, PhotoBoothMessage, PALETTE};

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

enum MainAppState<UH: Clone> {
    PaymentRequired(PaymentRequired),
    Preview(Preview),
    CapturePhotosPrepare {
        ready_timeline: anim::Timeline<animations::ready::AnimationState>,
    },
    CapturePhotos(CapturePhotos),
    RenderedPreview(RenderedPreview),
    EmailEntry(EmailEntry<UH>),
    Emailing(Emailing),
    StudentIDEntry(StudentIDEntry<UH>),
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
    state: MainAppState<S::UploadHandle>,
    captured_photos: Vec<RgbaImage>,
    previews: Vec<iced::widget::image::Handle>,
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
                    if let Some(effect) = emailing.update(EmailingMessage::Tick) {
                        match effect {
                            EmailingEffect::Complete => {
                                self.state = MainAppState::PaymentRequired(PaymentRequired::new());
                            }
                        }
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
                            email_entry.set_upload_handle(res.clone());
                            email_entry.set_qr_code_url(server_backend.get_link(res));
                        }
                        Task::none()
                    }
                    Err(err) => {
                        self.state = MainAppState::PaymentRequired(PaymentRequired::with_error(
                            format!("Failed to upload photos: {}", err),
                        ));
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
                        Some(EmailEntryEffect::Submit {
                            emails,
                            upload_handle,
                        }) => {
                            if emails.is_empty() {
                                let student_id_entry = StudentIDEntry::new(
                                    email_entry.strip_handle.clone(),
                                    upload_handle,
                                    emails,
                                );
                                self.state = MainAppState::StudentIDEntry(student_id_entry);
                                iced::widget::text_input::focus("student_id_input")
                            } else if !emails.is_empty() {
                                // Store emails and proceed to student ID entry
                                let student_id_entry = StudentIDEntry::new(
                                    email_entry.strip_handle.clone(),
                                    upload_handle,
                                    emails,
                                );
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
            MainAppMessage::StudentIDEntry(msg) => {
                // To avoid borrowing issues, extract needed data first
                let (upload_handle, emails, task) = match &mut self.state {
                    MainAppState::StudentIDEntry(student_id_entry) => {
                        let task = student_id_entry.update(msg);
                        (
                            student_id_entry.upload_handle.clone(),
                            student_id_entry.emails.clone(),
                            task,
                        )
                    }
                    _ => return Task::none(),
                };

                match task {
                    Some(StudentIDEntryEffect::Submit { student_id }) => {
                        let emailing = Emailing::new();
                        self.state = MainAppState::Emailing(emailing);
                        log::trace!("Sending email with photos...");
                        Task::perform(
                            server_backend.send_email(
                                upload_handle,
                                emails,
                                student_id,
                                iced::theme::palette::Extended::generate(PALETTE),
                            ),
                            |result| MainAppMessage::Emailed(result.map_err(|x| x.to_string())),
                        )
                    }
                    None => Task::none(),
                }
            }
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
                    let _effect = preview.update(msg);
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

                            let strip = render_take(photos.clone());
                            let strip_handle = Handle::from_rgba(
                                strip.width(),
                                strip.height(),
                                strip.as_raw().clone(),
                            );

                            let rendered_preview = RenderedPreview::new(strip_handle);
                            self.state = MainAppState::RenderedPreview(rendered_preview);

                            let future = server_backend.upload_photo(strip.clone(), photos);
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
                            let email_entry =
                                EmailEntry::new(rendered_preview.strip_handle.clone());
                            self.state = MainAppState::EmailEntry(email_entry);
                            iced::widget::text_input::focus("email_input")
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
                            emailing.finish();
                            Task::none()
                        }
                        Err(err) => {
                            self.state =
                                MainAppState::PaymentRequired(PaymentRequired::with_error(
                                    format!("Failed to email photos: {}", err),
                                ));
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
                MainAppState::PaymentRequired(payment_required) => {
                    payment_required.view().map(MainAppMessage::PaymentRequired)
                }
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
                MainAppState::EmailEntry(email_entry) => {
                    email_entry.view().map(MainAppMessage::EmailEntry).into()
                }
                MainAppState::StudentIDEntry(student_id_entry) => {
                    student_id_entry.view().map(MainAppMessage::StudentIDEntry)
                }
                MainAppState::Emailing(emailing) => emailing.view().map(MainAppMessage::Emailing),
            },
        ])
        .into()
    }
}
