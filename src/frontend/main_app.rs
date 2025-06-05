use std::time::Duration;

use anim::Animation;
use iced::{
    widget::{
        image::Handle, row, text,
    },
    Alignment, ContentFit, Element, Length, Task,
};
use image::RgbaImage;

use crate::{backend::render_take::render_take, AppPage, KeyMessage, PhotoBoothMessage, PALETTE};

use super::{
    camera_feed::{CameraFeed, CameraFeedOptions},
    loading_spinners,
};

mod animations;
mod status_overlay;
mod email_entry;
mod student_id_entry;
mod payment_required;
mod emailing;
mod capture_photos;
mod preview;
mod rendered_preview;

use email_entry::{EmailEntry, EmailEntryMessage, EmailEntryEffect};
use student_id_entry::{StudentIDEntry, StudentIDEntryMessage, StudentIDEntryEffect};
use payment_required::{PaymentRequired, PaymentRequiredMessage, PaymentRequiredEffect};
use emailing::{Emailing, EmailingMessage, EmailingEffect};
use capture_photos::{CapturePhotos, CapturePhotosMessage, CapturePhotosEffect};
use preview::{Preview, PreviewMessage};
use rendered_preview::{RenderedPreview, RenderedPreviewMessage, RenderedPreviewEffect};

const PHOTO_ASPECT_RATIO: f32 = 3.0 / 2.0;
const PHOTO_COUNT: usize = 4;

const QR_CODE_QUIET_ZONE: usize = 2;
const QR_CODE_VERSION: iced::widget::qr_code::Version = iced::widget::qr_code::Version::Normal(5);
const QR_CODE_SIDE_LENGTH: usize = QR_CODE_QUIET_ZONE * 2 + (5 * 4 + 17);

const EMAIL_REGEX: &str = r"^([a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+)@([a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$";

enum MainAppState {
    PaymentRequired,
    Preview,
    CapturePhotosPrepare {
        ready_timeline: anim::Timeline<animations::ready::AnimationState>,
    },
    CapturePhotos,
    RenderedPreview,
    EmailEntry,
    Emailing {
        progress_timeline: anim::Timeline<f32>,
    },
    StudentIDEntry,
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
    strip_handle: Option<Handle>,
    logo_handle: Handle,
    upload_handle: Option<S::UploadHandle>,
    qr_code_data: Option<iced::widget::qr_code::Data>,
    pub new_page: Option<Box<(AppPage<C, S>, Task<PhotoBoothMessage<C, S>>)>>,

    // Component states
    email_entry: EmailEntry,
    student_id_entry: StudentIDEntry,
    payment_required: PaymentRequired,
    emailing: Emailing,
    capture_photos: CapturePhotos,
    preview: Preview,
    rendered_preview: RenderedPreview,
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
                state: MainAppState::PaymentRequired,
                new_page: None,
                captured_photos: Vec::with_capacity(PHOTO_COUNT),
                previews: Vec::with_capacity(PHOTO_COUNT),
                logo_handle: Handle::from_bytes(include_bytes!("../../assets/banner.png").to_vec()),
                strip: None,
                strip_handle: None,
                qr_code_data: None,
                upload_handle: None,

                // Initialize component states
                email_entry: EmailEntry::new(),
                student_id_entry: StudentIDEntry::new(),
                payment_required: PaymentRequired::new(),
                emailing: Emailing::new(),
                capture_photos: CapturePhotos::new(),
                preview: Preview::new(),
                rendered_preview: RenderedPreview::new(),
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
                    | MainAppState::CapturePhotos { .. }
                    | MainAppState::Preview
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
                        self.state = MainAppState::CapturePhotos;
                        self.capture_photos = CapturePhotos::new();
                    };
                    Task::none()
                }
                MainAppState::CapturePhotos => {
                    Task::done(MainAppMessage::CapturePhotos(CapturePhotosMessage::Tick))
                }
                MainAppState::RenderedPreview => {
                    Task::done(MainAppMessage::RenderedPreview(RenderedPreviewMessage::Tick))
                }
                MainAppState::Emailing { progress_timeline } => {
                    if progress_timeline.update().is_completed() {
                        self.state = MainAppState::PaymentRequired;
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            MainAppMessage::Uploaded(result) => {
                log::debug!("Upload result received: {:?}", result);
                match result {
                    Ok(res) => {
                        self.upload_handle = Some(res);
                        self.qr_code_data = Some(
                            iced::widget::qr_code::Data::with_version(
                                server_backend
                                    .get_link(self.upload_handle.as_ref().unwrap().clone()),
                                QR_CODE_VERSION,
                                iced::widget::qr_code::ErrorCorrection::Medium,
                            )
                            .expect("could not create qr code"),
                        );
                        Task::none()
                    }
                    Err(err) => {
                        self.state = MainAppState::PaymentRequired;
                        log::error!("Error uploading photos: {}", err);
                        Task::none()
                    }
                }
            }
            MainAppMessage::KeyReleased(key) => {
                log::debug!("Key released: {:?}", key);
                match &mut self.state {
                    MainAppState::PaymentRequired => match key {
                        KeyMessage::Up => Task::none(),
                        KeyMessage::Down => Task::none(),
                        KeyMessage::Space => {
                            self.state = MainAppState::Preview;
                            Task::none()
                        }
                        KeyMessage::Escape => iced::widget::text_input::focus("email_input"),
                    },
                    MainAppState::Preview => {
                        self.state = MainAppState::CapturePhotosPrepare {
                            ready_timeline: animations::ready::animation().begin_animation(),
                        };
                        Task::none()
                    }
                    MainAppState::RenderedPreview => {
                        Task::done(MainAppMessage::RenderedPreview(RenderedPreviewMessage::Skip))
                    }
                    MainAppState::EmailEntry => iced::widget::text_input::focus("email_input"),
                    MainAppState::StudentIDEntry => {
                        iced::widget::text_input::focus("student_id_input")
                    }
                    _ => Task::none(),
                }
            }
            MainAppMessage::OtherKeyPress => match self.state {
                MainAppState::EmailEntry => iced::widget::text_input::focus("email_input"),
                MainAppState::StudentIDEntry => iced::widget::text_input::focus("student_id_input"),
                _ => Task::none(),
            },
            MainAppMessage::EmailEntry(msg) => match &self.state {
                MainAppState::EmailEntry => {
                    let (new_state, task) = self.email_entry.update(msg);
                    self.email_entry = new_state;

                    match task {
                        Some(EmailEntryEffect::Submit { emails }) => {
                            if emails.is_empty() && self.upload_handle.is_some() {
                                self.state = MainAppState::StudentIDEntry;
                                iced::widget::text_input::focus("student_id_input")
                            } else if !emails.is_empty() {
                                // Store emails and proceed to student ID entry
                                self.state = MainAppState::StudentIDEntry;
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
            MainAppMessage::StudentIDEntry(msg) => match &self.state {
                MainAppState::StudentIDEntry => {
                    let (new_state, task) = self.student_id_entry.update(msg);
                    self.student_id_entry = new_state;

                    match task {
                        Some(StudentIDEntryEffect::Submit { student_id }) => {
                            if let Some(upload_handle) = self.upload_handle.take() {
                                let emails = self.email_entry.get_emails();
                                let future = server_backend.send_email(
                                    upload_handle,
                                    emails,
                                    Some(student_id),
                                    iced::theme::palette::Extended::generate(PALETTE),
                                );
                                self.state = MainAppState::Emailing {
                                    progress_timeline: anim::Options::new(0.0, 0.8)
                                        .duration(Duration::from_millis(15000))
                                        .easing(
                                            anim::easing::cubic_ease()
                                                .mode(anim::easing::EasingMode::InOut),
                                        )
                                        .begin_animation(),
                                };
                                self.strip_handle = None;
                                self.strip = None;
                                log::trace!("Sending email with photos...");
                                Task::perform(future, |result| {
                                    MainAppMessage::Emailed(result.map_err(|x| x.to_string()))
                                })
                            } else {
                                log::error!("No upload handle available for emailing.");
                                self.state = MainAppState::PaymentRequired;
                                Task::none()
                            }
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::PaymentRequired(msg) => match &self.state {
                MainAppState::PaymentRequired => {
                    let (new_state, task) = self.payment_required.update(msg);
                    self.payment_required = new_state;

                    match task {
                        Some(PaymentRequiredEffect::StartSession) => {
                            self.state = MainAppState::Preview;
                            Task::none()
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::Preview(msg) => match &self.state {
                MainAppState::Preview => {
                    let (new_state, _effect) = self.preview.update(msg);
                    self.preview = new_state;
                    Task::none()
                }
                _ => Task::none(),
            },
            MainAppMessage::CapturePhotos(msg) => match &self.state {
                MainAppState::CapturePhotos => {
                    let task = self.capture_photos.update(msg, &mut self.captured_photos);

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
                            self.strip_handle = Some(Handle::from_rgba(
                                self.strip.as_ref().unwrap().width(),
                                self.strip.as_ref().unwrap().height(),
                                self.strip.as_ref().unwrap().as_raw().clone(),
                            ));
                            
                            self.upload_handle = None;
                            self.qr_code_data = None;
                            self.email_entry = EmailEntry::new();
                            self.rendered_preview = RenderedPreview::new();
                            self.state = MainAppState::RenderedPreview;
                            
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
            MainAppMessage::RenderedPreview(msg) => match &self.state {
                MainAppState::RenderedPreview => {
                    let task = self.rendered_preview.update(msg);

                    match task {
                        Some(RenderedPreviewEffect::Complete) => {
                            self.state = MainAppState::EmailEntry;
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
            MainAppMessage::Emailing(msg) => match &self.state {
                MainAppState::Emailing { .. } => {
                    let (new_state, task) = self.emailing.update(msg);
                    self.emailing = new_state;

                    match task {
                        Some(EmailingEffect::Complete) => {
                            self.state = MainAppState::PaymentRequired;
                            Task::none()
                        }
                        None => Task::none(),
                    }
                }
                _ => Task::none(),
            },
            MainAppMessage::Emailed(result) => {
                log::debug!("Email result received: {:?}", result);
                match self.state {
                    MainAppState::Emailing {
                        ref mut progress_timeline,
                    } => match result {
                        Ok(_) => {
                            *progress_timeline = anim::Options::new(progress_timeline.value(), 1.0)
                                .duration(Duration::from_millis(1000))
                                .easing(
                                    anim::easing::cubic_ease()
                                        .mode(anim::easing::EasingMode::InOut),
                                )
                                .begin_animation();
                            Task::none()
                        }
                        Err(err) => {
                            self.state = MainAppState::PaymentRequired;
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
                            | MainAppState::CapturePhotos
                            | MainAppState::Preview
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
                MainAppState::PaymentRequired => {
                    self.payment_required.view(None).map(MainAppMessage::PaymentRequired)
                },
                MainAppState::Preview => {
                    self.preview.view().map(MainAppMessage::Preview)
                }
                MainAppState::CapturePhotosPrepare { ready_timeline } => {
                    animations::ready::view(ready_timeline.value()).into()
                }
                MainAppState::CapturePhotos => {
                    self.capture_photos.view().map(MainAppMessage::CapturePhotos)
                },
                MainAppState::RenderedPreview => {
                    self.rendered_preview.view(self.strip_handle.as_ref()).map(MainAppMessage::RenderedPreview)
                },
                MainAppState::EmailEntry => iced::widget::stack([
                    self.email_entry.view(
                        self.upload_handle.as_ref(),
                        self.qr_code_data.as_ref(),
                        self.strip_handle.as_ref()
                    ).map(MainAppMessage::EmailEntry).into(),
                    if self.upload_handle.is_none() {
                        status_overlay::status_overlay(row([
                            loading_spinners::Circular::new()
                                .size(30.0)
                                .bar_height(3.0)
                                .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                                .into(),
                            text("Uploading photos in the background...").into()
                        ]).spacing(8).align_y(Alignment::Center)).into()
                    } else {
                        "".into()
                    }
                ]).into(),
                MainAppState::StudentIDEntry => {
                    self.student_id_entry.view(self.strip_handle.as_ref()).map(MainAppMessage::StudentIDEntry)
                },
                MainAppState::Emailing { progress_timeline } => {
                    self.emailing.view(progress_timeline.value()).map(MainAppMessage::Emailing)
                },
            },
        ])
        .into()
    }
}
