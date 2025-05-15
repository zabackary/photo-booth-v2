use std::time::Duration;

use anim::Animation;
use dotenv_codegen::dotenv;
use iced::{
    border::Radius,
    widget::{
        column, container, horizontal_space, image::Handle, progress_bar, row, text,
        vertical_space, Space,
    },
    Alignment, Border, Color, ContentFit, Element, Length, Padding, Task,
};
use image::RgbaImage;

use crate::{backend::render_take::render_take, AppPage, KeyMessage, PhotoBoothMessage, PALETTE};

use super::{
    camera_feed::{CameraFeed, CameraFeedOptions},
    loading_spinners,
    title_overlay::{supporting_text, title_overlay, title_text},
};

mod animations;
mod status_overlay;

const PHOTO_ASPECT_RATIO: f32 = 3.0 / 2.0;
const PHOTO_COUNT: usize = 4;

const QR_CODE_QUIET_ZONE: usize = 2;
const QR_CODE_VERSION: iced::widget::qr_code::Version = iced::widget::qr_code::Version::Normal(5);
const QR_CODE_SIDE_LENGTH: usize = QR_CODE_QUIET_ZONE * 2 + (5 * 4 + 17);

enum CapturePhotosState {
    Countdown {
        current: usize,
        countdown_timeline: anim::Timeline<animations::countdown_circle::AnimationState>,
    },
    Capture {
        capture_timeline: anim::Timeline<animations::capture_flash::AnimationState>,
    },
    Preview {
        preview_timeline: anim::Timeline<animations::capture_preview::AnimationState>,
        captured_handle: Handle,
    },
}

enum MainAppState {
    PaymentRequired {
        error: Option<String>,
    },
    Preview,
    CapturePhotosPrepare {
        ready_timeline: anim::Timeline<animations::ready::AnimationState>,
    },
    CapturePhotos {
        current: usize,
        state: CapturePhotosState,
    },
    RenderedPreview {
        progress_timeline: anim::Timeline<f32>,
        template_preview_timeline: anim::Timeline<animations::upsell_templates::AnimationState>,
    },
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

    EmailInput(String),
    EmailSubmit,
    StudentIDInput(String),
    StudentIDSubmit,
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
    emails: Vec<String>,
    student_id: String,
    upload_handle: Option<S::UploadHandle>,
    qr_code_data: Option<iced::widget::qr_code::Data>,
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
                state: MainAppState::PaymentRequired { error: None },
                new_page: None,
                captured_photos: Vec::with_capacity(PHOTO_COUNT),
                previews: Vec::with_capacity(PHOTO_COUNT),
                logo_handle: Handle::from_bytes(include_bytes!("../../assets/banner.png").to_vec()),
                strip: None,
                strip_handle: None,
                qr_code_data: None,
                student_id: String::new(),

                emails: Vec::new(),
                upload_handle: None,
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
                match &mut self.state {
                    MainAppState::CapturePhotos { state, .. } => {
                        *state = CapturePhotosState::Capture {
                            capture_timeline: animations::capture_flash::animation()
                                .begin_animation(),
                        }
                    }
                    _ => (),
                }
                Task::none()
            }
            MainAppMessage::Tick => match &mut self.state {
                MainAppState::CapturePhotosPrepare { ready_timeline } => {
                    if ready_timeline.update().is_completed() {
                        self.state = MainAppState::CapturePhotos {
                            current: 0,
                            state: CapturePhotosState::Countdown {
                                current: 3,
                                countdown_timeline: animations::countdown_circle::animation()
                                    .begin_animation(),
                            },
                        }
                    };
                    Task::none()
                }
                MainAppState::CapturePhotos { state, current } => match state {
                    CapturePhotosState::Countdown {
                        current,
                        countdown_timeline,
                    } => {
                        if countdown_timeline.update().is_completed() {
                            *current -= 1;
                            if *current == 0 {
                                *state = CapturePhotosState::Capture {
                                    capture_timeline: animations::capture_flash::animation()
                                        .to_timeline(),
                                };
                                return Task::done(MainAppMessage::CaptureStill);
                            } else {
                                *countdown_timeline =
                                    animations::countdown_circle::animation().begin_animation();
                            }
                        };
                        Task::none()
                    }
                    CapturePhotosState::Capture { capture_timeline } => {
                        if capture_timeline.update().is_completed() {
                            let last_photo = self
                                .captured_photos
                                .last()
                                .expect("capture didn't complete")
                                .clone();
                            *state = CapturePhotosState::Preview {
                                preview_timeline: animations::capture_preview::animation()
                                    .begin_animation(),
                                captured_handle: Handle::from_rgba(
                                    last_photo.width(),
                                    last_photo.height(),
                                    last_photo.into_raw(),
                                ),
                            }
                        };
                        Task::none()
                    }
                    CapturePhotosState::Preview {
                        preview_timeline, ..
                    } => {
                        if preview_timeline.update().is_completed() {
                            *current += 1;
                            if *current < PHOTO_COUNT {
                                *state = CapturePhotosState::Countdown {
                                    current: 3,
                                    countdown_timeline: animations::countdown_circle::animation()
                                        .begin_animation(),
                                };
                                Task::none()
                            } else {
                                let old = self.captured_photos.drain(..).collect::<Vec<_>>();
                                self.previews.clear();
                                for photo in &old {
                                    self.previews.push(iced::widget::image::Handle::from_rgba(
                                        photo.width(),
                                        photo.height(),
                                        photo.as_raw().clone(),
                                    ));
                                }
                                self.strip = Some(render_take(old.clone()));
                                self.strip_handle = Some(Handle::from_rgba(
                                    self.strip.as_ref().unwrap().width(),
                                    self.strip.as_ref().unwrap().height(),
                                    self.strip.as_ref().unwrap().as_raw().clone(),
                                ));
                                self.upload_handle = None;
                                self.qr_code_data = None;
                                self.state = MainAppState::RenderedPreview {
                                    progress_timeline: anim::Options::new(0.0, 1.0)
                                        .duration(Duration::from_millis(
                                            animations::upsell_templates::ANIMATION_LENGTH,
                                        ))
                                        .easing(anim::easing::linear())
                                        .begin_animation(),
                                    template_preview_timeline:
                                        animations::upsell_templates::animation().begin_animation(),
                                };
                                let future = server_backend
                                    .upload_photo(self.strip.as_ref().unwrap().clone(), old);
                                Task::perform(future, |result| {
                                    MainAppMessage::Uploaded(result.map_err(|x| x.to_string()))
                                })
                            }
                        } else {
                            Task::none()
                        }
                    }
                },
                MainAppState::RenderedPreview {
                    progress_timeline,
                    template_preview_timeline,
                } => {
                    template_preview_timeline.update();
                    if progress_timeline.update().is_completed()
                        && template_preview_timeline.update().is_completed()
                    {
                        self.state = MainAppState::EmailEntry;
                        self.emails = vec!["".to_string(); 1];
                        iced::widget::text_input::focus("email_input")
                    } else {
                        Task::none()
                    }
                }
                MainAppState::Emailing { progress_timeline } => {
                    if progress_timeline.update().is_completed() {
                        self.state = MainAppState::PaymentRequired { error: None };
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
                        self.state = MainAppState::PaymentRequired {
                            error: Some(
                                "The photos could not be uploaded. Please try again.".to_string(),
                            ),
                        };
                        log::error!("Error uploading photos: {}", err);
                        Task::none()
                    }
                }
            }
            MainAppMessage::KeyReleased(key) => {
                log::debug!("Key released: {:?}", key);
                match &mut self.state {
                    MainAppState::PaymentRequired { .. } => match key {
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
                    MainAppState::RenderedPreview {
                        progress_timeline, ..
                    } => {
                        *progress_timeline = anim::Options::new(progress_timeline.value(), 1.0)
                            .duration(Duration::from_millis(1000))
                            .easing(
                                anim::easing::cubic_ease().mode(anim::easing::EasingMode::InOut),
                            )
                            .begin_animation();
                        Task::none()
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
            MainAppMessage::EmailInput(email) => {
                if self.emails.is_empty() {
                    self.emails.push(email);
                } else {
                    self.emails[0] = email;
                }
                Task::none()
            }
            MainAppMessage::EmailSubmit => {
                log::debug!("Email submit triggered. Current emails: {:?}", self.emails);
                if self.emails[0].len() > 0 {
                    self.emails.splice(0..0, ["".to_string()]);
                    Task::none()
                } else {
                    if self.upload_handle.is_none() {
                        log::warn!("Didn't finish uploading.");
                        return Task::none();
                    }
                    self.emails.splice(0..1, []);
                    self.student_id = String::new();
                    self.state = MainAppState::StudentIDEntry;
                    iced::widget::text_input::focus("student_id_input")
                }
            }
            MainAppMessage::StudentIDInput(student_id) => {
                self.student_id = student_id;
                Task::none()
            }
            MainAppMessage::StudentIDSubmit => {
                if let Some(upload_handle) = self.upload_handle.take() {
                    let future = server_backend.send_email(
                        upload_handle,
                        self.emails.clone(),
                        if self.student_id.is_empty() {
                            None
                        } else {
                            Some(self.student_id.clone())
                        },
                        iced::theme::palette::Extended::generate(PALETTE),
                    );
                    self.state = MainAppState::Emailing {
                        progress_timeline: anim::Options::new(0.0, 0.8)
                            .duration(Duration::from_millis(15000))
                            .easing(
                                anim::easing::cubic_ease().mode(anim::easing::EasingMode::InOut),
                            )
                            .begin_animation(),
                    };
                    self.emails.clear();
                    self.strip_handle = None;
                    self.strip = None;
                    log::trace!("Sending email with photos...");
                    Task::perform(future, |result| {
                        MainAppMessage::Emailed(result.map_err(|x| x.to_string()))
                    })
                } else {
                    log::error!("No upload handle available for emailing.");
                    self.state = MainAppState::PaymentRequired {
                        error: Some(
                            "The photos could not be emailed. Please try again.".to_string(),
                        ),
                    };
                    Task::none()
                }
            }
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
                            self.state = MainAppState::PaymentRequired {
                                error: Some(
                                    format!("We had a problem sending your photos. They have been saved; please contact {} for assistance. {}", env!("CONTACT_EMAIL"), err),
                                ),
                            };
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
                            | MainAppState::CapturePhotos { .. }
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
                MainAppState::PaymentRequired { error } => title_overlay(
                    container(
                        container(
                            column([
                                vertical_space().height(6).into(),
                                iced::widget::image(self.logo_handle.clone())
                                    .width(800)
                                    .height(300)
                                    .content_fit(ContentFit::Contain)
                                    .into(),
                                vertical_space().height(6).into(),
                                iced::widget::text("Press [SPACE] to get started.")
                                    .size(24)
                                    .into(),
                                    vertical_space().height(12).into(),
                                    iced::widget::text(dotenv!("PRIVACY_NOTE"))
                                        .size(18)
                                        .into(),
                                vertical_space().height(12).into(),
                                if let Some(error_message) = error {
                                    column([
                                        vertical_space().height(12).into(),
                                        container(column([iced::widget::text(
                                            error_message
                                        )
                                        .shaping(
                                            iced::widget::text::Shaping::Advanced,
                                        )
                                        .size(16)
                                        .into()]))
                                        .style(|theme: &iced::Theme| container::Style {
                                            border: iced::Border::default().rounded(4.0).color(
                                                theme.extended_palette().danger.strong.color,
                                            ).width(1.0),
                                            background: Some(
                                                theme.extended_palette().danger.weak.color.into(),
                                            ),
                                            text_color: Some(
                                                theme.extended_palette().danger.weak.text,
                                            ),
                                            ..Default::default()
                                        })
                                        .padding(8)
                                        .into(),
                                    ])
                                    .into()
                                } else {
                                    Space::new(0, 0).into()
                                },
                            ])
                            .align_x(Alignment::Center),
                        )
                        .max_width(780)
                        .padding(18)
                        .style(|theme: &iced::Theme| container::Style {
                            border: iced::Border::default().rounded(28),
                            background: Some(theme.extended_palette().primary.base.color.into()),
                            text_color: Some(Color::from_rgb8(0xff, 0xff, 0xff)),
                            ..Default::default()
                        }),
                    )
                    .center(Length::Fill),
                    false,
                )
                .into(),
                MainAppState::Preview => title_overlay(
                    column([
                        title_text("Get ready to take your pictures").into(),
                        supporting_text("Press [SPACE] to start when you're ready.").into(),
                        vertical_space().height(12.0).into(),
                    ]),
                    true,
                ),
                MainAppState::CapturePhotosPrepare { ready_timeline } => {
                    animations::ready::view(ready_timeline.value()).into()
                }
                MainAppState::CapturePhotos { current, state } => iced::widget::stack([
                    status_overlay::status_overlay(text(format!("photo {} of {PHOTO_COUNT}", current + 1)).size(24)).into(),
                    match state {
                        CapturePhotosState::Countdown {
                            current,
                            countdown_timeline,
                        } => animations::countdown_circle::view(*current, countdown_timeline.value())
                            .into(),
                        CapturePhotosState::Capture { capture_timeline } => {
                            animations::capture_flash::view(capture_timeline.value()).into()
                        }
                        CapturePhotosState::Preview {
                            preview_timeline,
                            captured_handle,
                        } => {
                            animations::capture_preview::view(captured_handle, preview_timeline.value())
                                .into()
                        }
                    }
                ]).into(),
                MainAppState::RenderedPreview {
                    progress_timeline,
                    template_preview_timeline,
                } => iced::widget::stack([
                    title_overlay(
                        column([
                            animations::upsell_templates::view(
                                &self.strip_handle.as_ref().unwrap(),
                                template_preview_timeline.value(),
                            )
                            .into(),
                            title_text("Your photos are ready!").into(),
                            supporting_text("On the next screen, enter your emails.").into(),
                            vertical_space().height(12.0).into(),
                            progress_bar(0.0..=1.0, progress_timeline.value())
                                .height(4.0)
                                .into(),
                        ]),
                        false,
                    )
                    .into(),
                    status_overlay::status_overlay(row([
                        loading_spinners::Circular::new()
                            .size(30.0)
                            .bar_height(3.0)
                            .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                            .into(),
                        text("Uploading photos in the background...").into()
                    ]).spacing(8).align_y(Alignment::Center)).into()
                ]).into(),
                MainAppState::EmailEntry => iced::widget::stack([
                    title_overlay(
                        row([
                            column([
                                title_text("Enter your email addresses").width(Length::Shrink).into(),
                                supporting_text("Start typing to add an email.").width(Length::Shrink).into(),
                                vertical_space().height(12.0).into(),
                                container(
                                    column([
                                        row([
                                            iced::widget::text_input(
                                                "Enter an email",
                                                self.emails[0].as_str(),
                                            )
                                            .on_input(MainAppMessage::EmailInput)
                                            .on_submit(MainAppMessage::EmailSubmit)
                                            .padding(10)
                                            .size(24)
                                            .id("email_input")
                                            .into(),
                                            horizontal_space().width(6.0).into(),
                                            iced::widget::button(iced::widget::text(if self.emails[0].len() > 0 {
                                                "Press [Enter] to add"
                                            } else {
                                                "Press [Enter] to finish"
                                            })
                                            .size(24))
                                            .on_press_maybe(
                                                if self.upload_handle.is_none() && self.emails[0].len() == 0 {
                                                    None
                                                } else {
                                                    Some(MainAppMessage::EmailSubmit)
                                                }
                                            )
                                            .padding(10)
                                            .into(),
                                        ])
                                        .into(),
                                        vertical_space().height(12.0).into(),
                                        container(
                                            if self.emails.len() <= 1 {
                                                Element::from(column([
                                                    text("You can also scan the QR code to download your photos!").into(),
                                                    Element::from(if let Some(ref qr_code_data) = self.qr_code_data {
                                                        container(
                                                            iced::widget::qr_code(qr_code_data).cell_size(8).style(|_|iced::widget::qr_code::Style {
                                                                background: Color::WHITE,
                                                                cell: Color::BLACK
                                                            })
                                                        ).width((QR_CODE_SIDE_LENGTH * 8) as u16).height((QR_CODE_SIDE_LENGTH * 8) as u16).padding(8)
                                                    } else {
                                                        container(
                                                            column([
                                                                loading_spinners::Circular::new()
                                                                    .size(40.0)
                                                                    .bar_height(4.0)
                                                                    .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                                                                    .into(),
                                                                text("Uploading and generating code...").into()
                                                            ])
                                                            .align_x(Alignment::Center)
                                                            .spacing(8)
                                                        ).style(|_| container::background(Color::WHITE)).padding(8).center((QR_CODE_SIDE_LENGTH * 8) as u16)
                                                    })
                                                ]).spacing(16).padding(4).align_x(Alignment::Center))
                                            } else {
                                                column(
                                                    self.emails
                                                        .iter()
                                                        .skip(1)
                                                        .map(|email| {
                                                            iced::widget::container(
                                                                iced::widget::text(email.as_str())
                                                                    .size(24)
                                                            ).width(Length::Fill)
                                                                .padding(Padding {
                                                                    bottom: 10.0,
                                                                    left: 16.0,
                                                                    right: 16.0,
                                                                    top: 10.0,
                                                                })
                                                                .style(|theme: &iced::Theme| container::Style {
                                                                    background: Some(
                                                                        theme.extended_palette().background.strong.color.into(),
                                                                    ),
                                                                    text_color: Some(
                                                                        theme.extended_palette().background.strong.text,
                                                                    ),
                                                                    border: Border::default().rounded(999.0),
                                                                    ..Default::default()
                                                                }).into()
                                                        }),
                                                ).push(vertical_space()).spacing(8).into()
                                            },
                                        )
                                        .padding(12)
                                        .style(|theme: &iced::Theme| container::Style {
                                            background: Some(
                                                theme.extended_palette().background.base.color.scale_alpha(0.3).into(),
                                            ),
                                            border: Border::default().rounded(36.0),
                                            ..Default::default()
                                        })
                                        .width(Length::Fill)
                                        .center(Length::Fill)
                                        .into(),
                                        vertical_space().height(12.0).into(),
                                        container(
                                            column([
                                                iced::widget::text("Make sure your email provider accepts emails from photobooth@caj.ac.jp.")
                                                    .size(18)
                                                    .into(),
                                            ]).align_x(Alignment::Center)
                                        ).into()
                                    ])
                                    .align_x(Alignment::Center),
                                )
                                .center(Length::Fill)
                                .max_width(700.0)
                                .into(),
                            ])
                            .padding(100)
                            .align_x(Alignment::Center)
                            .width(Length::Fill)
                            .into(),
                            horizontal_space().width(12.0).into(),
                            container(
                                column([
                                    supporting_text("Your photos").width(Length::Shrink).into(),
                                    vertical_space().height(12.0).into(),
                                    iced::widget::image(self.strip_handle.as_ref().unwrap().clone())
                                        .height(Length::Fill)
                                        .content_fit(ContentFit::Contain)
                                        .into(),
                                ])
                                .align_x(Alignment::Center)
                                .padding(30)
                            ).style(|theme: &iced::Theme| container::Style {
                                background: Some(
                                    theme.extended_palette().background.base.color.scale_alpha(0.8).into(),
                                ),
                                border: Border::default().rounded(Radius {
                                    bottom_left: 24.0,
                                    bottom_right: 0.0,
                                    top_left: 24.0,
                                    top_right: 0.0,
                                }),
                                ..Default::default()
                            })
                            .into(),
                        ]).align_y(Alignment::Center),
                        false,
                    ).into(),
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
                MainAppState::StudentIDEntry => title_overlay(
                    row([
                        column([
                            iced::widget::image(self.strip_handle.as_ref().unwrap().clone())
                                .height(Length::Fill)
                                .content_fit(ContentFit::Contain)
                                .into(),
                            vertical_space().height(12.0).into(),
                            title_text("Would you like it printed?").width(Length::Shrink).into(),
                            supporting_text("We'll deliver two copies of your photo to you next week for only 300 yen, billed to your student account. If you would prefer not to purchase one, press [Enter] without entering anything.").width(Length::Shrink).into(),
                            vertical_space().height(12.0).into(),
                            container(
                                row([
                                    iced::widget::text_input(
                                        "Enter your student ID",
                                        &self.student_id,
                                    )
                                    .on_input(MainAppMessage::StudentIDInput)
                                    .on_submit(MainAppMessage::StudentIDSubmit)
                                    .padding(10)
                                    .size(24)
                                    .id("student_id_input")
                                    .into(),
                                    horizontal_space().width(6.0).into(),
                                    iced::widget::button(iced::widget::text(if !self.student_id.is_empty() {
                                        "[Enter] to confirm"
                                    } else {
                                        "[Enter] to cancel"
                                    })
                                    .size(24))
                                    .on_press(MainAppMessage::StudentIDSubmit)
                                    .padding(10)
                                    .into(),
                                ]),
                            )
                            .max_width(700.0)
                            .into(),
                        ])
                        .padding(100)
                        .align_x(Alignment::Center)
                        .width(Length::Fill)
                        .into(),
                    ])
                    .align_y(Alignment::Center),
                    false,
                ).into(),
                MainAppState::Emailing { progress_timeline } => title_overlay(
                    iced::widget::column([
                        container(
                            loading_spinners::Circular::new()
                                .size(90.0)
                                .bar_height(9.0)
                                .easing(&loading_spinners::easing::STANDARD_DECELERATE),
                        )
                        .center(Length::Fill)
                        .into(),
                        title_text("We're processing your photos now.").into(),
                        supporting_text("If you entered your email, check your inbox to download your pictures.").into(),
                        vertical_space().height(12.0).into(),
                        progress_bar(0.0..=1.0, progress_timeline.value())
                            .height(8.0)
                            .into(),
                    ]),
                    false,
                )
                .into(),
            },
        ])
        .into()
    }
}
