use iced::{
    Alignment, ContentFit, Element, Task,
    widget::{row, text},
};
use image::RgbaImage;

use super::camera_feed::{CameraFeed, CameraFeedOptions};

mod animations;

mod capture_photos;
mod capture_photos_prepare;
mod email_entry;
mod emailing;
mod error;
mod pick_strip;
mod preview;
mod print_pending;
mod qr_code;
mod rendering;
mod status_overlay;

#[derive(Debug)]
enum MainAppPage {
    Preview(preview::Preview),
    CapturePhotosPrepare(capture_photos_prepare::CapturePhotosPrepare),
    CapturePhotos(capture_photos::CapturePhotos),
    Rendering(rendering::Rendering),
    PickStrip(pick_strip::PickStrip),
    EmailEntry(email_entry::EmailEntry),
    QrCode(qr_code::QrCode),
    PrintPending(print_pending::PrintPending),
    Emailing(emailing::Emailing),
    Error(error::Error),
}

#[derive(Debug, Clone)]
pub enum MainAppMessage {
    OnRendered(Result<Vec<image::RgbaImage>, String>),
    OnUploaded(Result<crate::backend::storage::StorageHandle, String>),
    OnPrintWaitFinish,
    OnPrintFinish(Result<(), String>),
    OnEmailed(Result<(), String>),

    CameraFeed(super::camera_feed::CameraMessage),

    Preview(preview::PreviewMessage),
    CapturePhotosPrepare(capture_photos_prepare::CapturePhotosPrepareMessage),
    CapturePhotos(capture_photos::CapturePhotosMessage),
    Rendering(rendering::RenderingMessage),
    PickStrip(pick_strip::PickStripMessage),
    PrintPending(print_pending::PrintPendingMessage),
    EmailEntry(email_entry::EmailEntryMessage),
    QrCode(qr_code::QrCodeMessage),
    Emailing(emailing::EmailingMessage),
    Error(error::ErrorMessage),
}

#[derive(Debug)]
pub enum MainAppAction {
    None,
    Task(Task<MainAppMessage>),
}

/// State needed for the current session
#[derive(Debug)]
pub struct Session {
    captured_photos: Vec<RgbaImage>,
    selected_strip: Option<usize>,
    strips: Option<Vec<RgbaImage>>,
    storage_handle: Option<crate::backend::storage::StorageHandle>,
    num_copies: u32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            captured_photos: Vec::new(),
            selected_strip: None,
            strips: None,
            storage_handle: None,
            num_copies: 1,
        }
    }
}

#[derive(Debug)]
pub struct MainApp {
    feed: CameraFeed,
    page: MainAppPage,
    session: Session,

    manager: crate::backend::manager::BackendManager,
    config: &'static crate::config::Config,
}

impl MainApp {
    pub fn new(
        manager: crate::backend::manager::BackendManager,
        config: &'static crate::config::Config,
    ) -> (Self, Task<MainAppMessage>) {
        let (feed, feed_task) = CameraFeed::new(manager.clone(), Default::default());
        (
            Self {
                feed,
                page: MainAppPage::Preview(preview::Preview::new()),
                session: Session::default(),
                manager,
                config,
            },
            feed_task.map(MainAppMessage::CameraFeed),
        )
    }

    pub fn update(&mut self, message: MainAppMessage) -> MainAppAction {
        self.feed.update_options(
            if matches!(
                self.page,
                MainAppPage::CapturePhotosPrepare(_)
                    | MainAppPage::CapturePhotos(_)
                    | MainAppPage::Preview(_)
            ) {
                CameraFeedOptions {
                    blur: 1.0,
                    aspect_ratio: Some(self.config.camera.preview_aspect_ratio),
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
            MainAppMessage::OnRendered(result) => match result {
                Ok(strips) => {
                    self.session.strips = Some(strips.clone());
                    if let MainAppPage::Rendering(rendering) = &mut self.page {
                        rendering.finish();
                    }
                    MainAppAction::None
                }
                Err(err) => {
                    log::error!("Error rendering photos: {:?}", err);
                    self.page = MainAppPage::Error(error::Error::new(format!(
                        "Failed to render photos: {}",
                        err
                    )));
                    MainAppAction::None
                }
            },
            MainAppMessage::OnUploaded(result) => match result {
                Ok(handle) => {
                    log::debug!("Successfully uploaded strip with handle {:?}", handle);
                    self.session.storage_handle = Some(handle.clone());
                    if let MainAppPage::EmailEntry(email_entry) = &mut self.page {
                        email_entry.on_storage_finish(handle);
                    } else if let MainAppPage::QrCode(qr_code) = &mut self.page {
                        qr_code.on_storage_finish(handle);
                    }
                    MainAppAction::None
                }
                Err(err) => {
                    log::error!("Error uploading strip: {:?}", err);
                    self.page = MainAppPage::Error(error::Error::new(format!(
                        "Failed to upload strip: {}",
                        err
                    )));
                    MainAppAction::None
                }
            },
            MainAppMessage::OnPrintWaitFinish => {
                if let MainAppPage::PrintPending(print_pending) = &mut self.page {
                    print_pending.finish();
                }
                MainAppAction::None
            }
            MainAppMessage::OnPrintFinish(result) => match result {
                Ok(()) => {
                    log::info!("Successfully finished printing strip");
                    MainAppAction::None
                }
                Err(err) => {
                    log::warn!("Error printing strip: {:?}", err);
                    // could have been printing previous session's strip, so
                    // maybe only show a small icon?
                    MainAppAction::None
                }
            },
            MainAppMessage::OnEmailed(result) => match result {
                Ok(()) => {
                    log::info!("Successfully sent email");
                    if let MainAppPage::Emailing(emailing) = &mut self.page {
                        emailing.finish();
                    }
                    MainAppAction::None
                }
                Err(err) => {
                    log::error!("Error sending email: {:?}", err);
                    self.page = MainAppPage::Error(error::Error::new(format!(
                        "Failed to send email: {}",
                        err
                    )));
                    MainAppAction::None
                }
            },
            MainAppMessage::CameraFeed(msg) => {
                MainAppAction::Task(self.feed.update(msg).map(MainAppMessage::CameraFeed))
            }
            MainAppMessage::Preview(message) => {
                if let MainAppPage::Preview(preview) = &mut self.page {
                    match preview.update(message) {
                        preview::PreviewAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::Preview))
                        }
                        preview::PreviewAction::Complete => {
                            self.page = MainAppPage::CapturePhotosPrepare(
                                capture_photos_prepare::CapturePhotosPrepare::new(),
                            );
                            MainAppAction::None
                        }
                        preview::PreviewAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::CapturePhotosPrepare(message) => {
                if let MainAppPage::CapturePhotosPrepare(capture_photos_prepare) = &mut self.page {
                    match capture_photos_prepare.update(message) {
                        capture_photos_prepare::CapturePhotosPrepareAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::CapturePhotosPrepare))
                        }
                        capture_photos_prepare::CapturePhotosPrepareAction::Complete => {
                            self.page =
                                MainAppPage::CapturePhotos(capture_photos::CapturePhotos::new(
                                    self.manager.clone(),
                                    self.config,
                                ));
                            MainAppAction::None
                        }
                        capture_photos_prepare::CapturePhotosPrepareAction::None => {
                            MainAppAction::None
                        }
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::CapturePhotos(message) => {
                if let MainAppPage::CapturePhotos(capture_photos) = &mut self.page {
                    match capture_photos.update(message) {
                        capture_photos::CapturePhotosAction::PhotosComplete { photos } => {
                            self.session.captured_photos = photos.clone();
                            self.page = MainAppPage::Rendering(rendering::Rendering::new());
                            let renderer = self.manager.renderer_manager.clone();
                            MainAppAction::Task(iced::Task::perform(
                                async move { renderer.render(photos).await },
                                |result| {
                                    MainAppMessage::OnRendered(result.map_err(|x| x.to_string()))
                                },
                            ))
                        }
                        capture_photos::CapturePhotosAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::CapturePhotos))
                        }
                        capture_photos::CapturePhotosAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::Rendering(message) => {
                if let MainAppPage::Rendering(rendering) = &mut self.page {
                    match rendering.update(message) {
                        rendering::RenderingAction::Complete => {
                            self.page = MainAppPage::PickStrip(pick_strip::PickStrip::new(
                                self.session.strips.clone().expect("no strips rendered"),
                            ));
                            MainAppAction::None
                        }
                        rendering::RenderingAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::Rendering))
                        }
                        rendering::RenderingAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::PickStrip(message) => {
                if let MainAppPage::PickStrip(pick_strip) = &mut self.page {
                    match pick_strip.update(message) {
                        pick_strip::PickStripAction::Complete { selection } => {
                            log::debug!("User selected strip {}", selection);
                            self.session.selected_strip = Some(selection);
                            let strip = self.session.strips.as_ref().expect("no strips rendered")
                                [selection]
                                .clone();

                            // upload
                            let manager = self.manager.clone();
                            let photos = self.session.captured_photos.clone();
                            let upload_strip = strip.clone();
                            let upload_task = iced::Task::perform(
                                async move {
                                    // Start uploading the selected strip immediately
                                    manager
                                        .storage_manager
                                        .store(upload_strip, photos)
                                        .await
                                        .map_err(|err| {
                                            log::error!("Failed to upload strip: {:?}", err);
                                            err.to_string()
                                        })
                                },
                                MainAppMessage::OnUploaded,
                            );

                            // if there's a printer, wait for it
                            if let Some(printer_manager) = self.manager.printer_manager.clone() {
                                self.page =
                                    MainAppPage::PrintPending(print_pending::PrintPending::new());
                                let print_task = iced::Task::perform(
                                    async move {
                                        printer_manager.wait().await;
                                    },
                                    |_| MainAppMessage::OnPrintWaitFinish,
                                );
                                MainAppAction::Task(iced::Task::batch([print_task, upload_task]))
                            } else if self.manager.email_manager.is_some() {
                                let (email_entry, email_entry_task) = email_entry::EmailEntry::new(
                                    iced::widget::image::Handle::from_rgba(
                                        strip.width(),
                                        strip.height(),
                                        strip.clone().into_raw(),
                                    ),
                                    self.manager.clone(),
                                    self.session.storage_handle.clone(),
                                );
                                self.page = MainAppPage::EmailEntry(email_entry);
                                MainAppAction::Task(iced::Task::batch([
                                    email_entry_task.map(MainAppMessage::EmailEntry),
                                    upload_task,
                                ]))
                            } else {
                                // no printer or email, just show the QR code
                                let (qr_code, qr_code_task) = qr_code::QrCode::new(
                                    iced::widget::image::Handle::from_rgba(
                                        strip.width(),
                                        strip.height(),
                                        strip.clone().into_raw(),
                                    ),
                                    self.manager.clone(),
                                    self.session.storage_handle.clone(),
                                );
                                self.page = MainAppPage::QrCode(qr_code);
                                MainAppAction::Task(qr_code_task.map(MainAppMessage::QrCode))
                            }
                        }
                        pick_strip::PickStripAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::PickStrip))
                        }
                        pick_strip::PickStripAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::PrintPending(message) => {
                if let MainAppPage::PrintPending(print_pending) = &mut self.page {
                    match print_pending.update(message) {
                        print_pending::PrintPendingAction::Complete => {
                            let strip = self.session.strips.as_ref().expect("no strips rendered")
                                [self.session.selected_strip.expect("no strip selected")]
                            .clone();

                            let next_task = if self.manager.email_manager.is_some() {
                                let (email_entry, email_entry_task) = email_entry::EmailEntry::new(
                                    iced::widget::image::Handle::from_rgba(
                                        strip.width(),
                                        strip.height(),
                                        strip.clone().into_raw(),
                                    ),
                                    self.manager.clone(),
                                    self.session.storage_handle.clone(),
                                );
                                self.page = MainAppPage::EmailEntry(email_entry);
                                email_entry_task.map(MainAppMessage::EmailEntry)
                            } else {
                                let (qr_code, qr_code_task) = qr_code::QrCode::new(
                                    iced::widget::image::Handle::from_rgba(
                                        strip.width(),
                                        strip.height(),
                                        strip.clone().into_raw(),
                                    ),
                                    self.manager.clone(),
                                    self.session.storage_handle.clone(),
                                );
                                self.page = MainAppPage::QrCode(qr_code);
                                qr_code_task.map(MainAppMessage::QrCode)
                            };

                            let printer_manager = self
                                .manager
                                .printer_manager
                                .clone()
                                .expect("no printer manager");
                            let num_copies = self.session.num_copies;
                            let print_task = iced::Task::perform(
                                async move {
                                    printer_manager
                                        .print(strip, num_copies)
                                        .await
                                        .map_err(|err| {
                                            log::error!("Failed to print strip: {:?}", err);
                                            err.to_string()
                                        })
                                },
                                MainAppMessage::OnPrintFinish,
                            );

                            MainAppAction::Task(iced::Task::batch([print_task, next_task]))
                        }
                        print_pending::PrintPendingAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::PrintPending))
                        }
                        print_pending::PrintPendingAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::EmailEntry(message) => {
                if let MainAppPage::EmailEntry(email_entry) = &mut self.page {
                    match email_entry.update(message) {
                        email_entry::EmailEntryAction::Submit { emails } => {
                            log::debug!("User submitted emails: {:?}", emails);
                            let storage_handle = self
                                .session
                                .storage_handle
                                .clone()
                                .expect("no storage handle");
                            self.page = MainAppPage::Emailing(emailing::Emailing::new());
                            let manager = self.manager.clone();
                            MainAppAction::Task(iced::Task::perform(
                                async move {
                                    manager
                                        .email_manager
                                        .expect("email entry should not be possible without email manager")
                                        .send_email(storage_handle, emails)
                                        .await
                                        .map_err(|err| {
                                            log::error!("Failed to send email: {:?}", err);
                                            err.to_string()
                                        })
                                },
                                MainAppMessage::OnEmailed,
                            ))
                        }
                        email_entry::EmailEntryAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::EmailEntry))
                        }
                        email_entry::EmailEntryAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::QrCode(message) => {
                if let MainAppPage::QrCode(qr_code) = &mut self.page {
                    match qr_code.update(message) {
                        qr_code::QrCodeAction::Continue => {
                            self.page = MainAppPage::Preview(preview::Preview::new());
                            self.session = Session::default();
                            MainAppAction::None
                        }
                        qr_code::QrCodeAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::QrCode))
                        }
                        qr_code::QrCodeAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::Emailing(message) => {
                if let MainAppPage::Emailing(emailing) = &mut self.page {
                    match emailing.update(message) {
                        emailing::EmailingAction::Complete => {
                            self.page = MainAppPage::Preview(preview::Preview::new());
                            self.session = Session::default();
                            MainAppAction::None
                        }
                        emailing::EmailingAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::Emailing))
                        }
                        emailing::EmailingAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
            MainAppMessage::Error(message) => {
                if let MainAppPage::Error(error) = &mut self.page {
                    match error.update(message) {
                        error::ErrorAction::Complete => {
                            self.page = MainAppPage::Preview(preview::Preview::new());
                            self.session = Session::default();
                            MainAppAction::None
                        }
                        error::ErrorAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::Error))
                        }
                        error::ErrorAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<MainAppMessage> {
        iced::Subscription::batch([
            self.feed.subscription().map(MainAppMessage::CameraFeed),
            match &self.page {
                MainAppPage::Preview(preview) => {
                    preview.subscription().map(MainAppMessage::Preview)
                }
                MainAppPage::QrCode(qr_code) => qr_code.subscription().map(MainAppMessage::QrCode),
                MainAppPage::CapturePhotosPrepare(capture_photos_prepare) => capture_photos_prepare
                    .subscription()
                    .map(MainAppMessage::CapturePhotosPrepare),
                MainAppPage::CapturePhotos(capture_photos) => capture_photos
                    .subscription()
                    .map(MainAppMessage::CapturePhotos),
                MainAppPage::Rendering(rendering) => {
                    rendering.subscription().map(MainAppMessage::Rendering)
                }
                MainAppPage::PickStrip(pick_strip) => {
                    pick_strip.subscription().map(MainAppMessage::PickStrip)
                }
                MainAppPage::PrintPending(print_pending) => print_pending
                    .subscription()
                    .map(MainAppMessage::PrintPending),
                MainAppPage::EmailEntry(email_entry) => {
                    email_entry.subscription().map(MainAppMessage::EmailEntry)
                }
                MainAppPage::Emailing(emailing) => {
                    emailing.subscription().map(MainAppMessage::Emailing)
                }
                MainAppPage::Error(error) => error.subscription().map(MainAppMessage::Error),
            },
        ])
    }

    pub fn view(&self) -> Element<'_, MainAppMessage> {
        let camera_info = if self.manager.camera_manager.is_reconnecting() {
            "error, reconnecting"
        } else {
            "ok"
        };
        let printer_info = if let Some(printer_manager) = &self.manager.printer_manager {
            if printer_manager.is_reconnecting() {
                "error, reconnecting"
            } else if printer_manager.busy() {
                "busy"
            } else {
                "ok"
            }
        } else {
            "none"
        };
        let emailer_info = if let Some(email_manager) = &self.manager.email_manager {
            if email_manager.busy() { "busy" } else { "ok" }
        } else {
            "none"
        };
        let storage_info = if self.manager.storage_manager.busy() {
            "busy"
        } else {
            "ok"
        };
        let info = format!(
            "photo-booth-v2 v{} | camera: {} | printer: {} | emailer: {} | storage: {}",
            env!("CARGO_PKG_VERSION"),
            camera_info,
            printer_info,
            emailer_info,
            storage_info
        );
        iced::widget::stack([
            // Bottom layer: camera feed
            self.feed
                .view(
                    if matches!(
                        self.page,
                        MainAppPage::CapturePhotosPrepare { .. }
                            | MainAppPage::CapturePhotos(_)
                            | MainAppPage::Preview(_)
                    ) {
                        ContentFit::Contain
                    } else {
                        ContentFit::Cover
                    },
                )
                .map(MainAppMessage::CameraFeed),
            match &self.page {
                MainAppPage::Preview(preview) => preview.view().map(MainAppMessage::Preview),
                MainAppPage::CapturePhotosPrepare(capture_photos_prepare) => capture_photos_prepare
                    .view()
                    .map(MainAppMessage::CapturePhotosPrepare),
                MainAppPage::CapturePhotos(capture_photos) => {
                    capture_photos.view().map(MainAppMessage::CapturePhotos)
                }
                MainAppPage::Rendering(rendering) => {
                    rendering.view().map(MainAppMessage::Rendering)
                }
                MainAppPage::PickStrip(pick_strip) => {
                    pick_strip.view().map(MainAppMessage::PickStrip)
                }
                MainAppPage::PrintPending(print_pending) => {
                    print_pending.view().map(MainAppMessage::PrintPending)
                }
                MainAppPage::EmailEntry(email_entry) => {
                    email_entry.view().map(MainAppMessage::EmailEntry)
                }
                MainAppPage::QrCode(qr_code) => qr_code.view().map(MainAppMessage::QrCode),
                MainAppPage::Emailing(emailing) => emailing.view().map(MainAppMessage::Emailing),
                MainAppPage::Error(error) => error.view().map(MainAppMessage::Error),
            },
            if self.manager.storage_manager.busy() {
                status_overlay::status_overlay(
                    row([
                        super::loading_spinners::Circular::new()
                            .size(30.0)
                            .bar_height(3.0)
                            .easing(&super::loading_spinners::easing::STANDARD_DECELERATE)
                            .into(),
                        text("Uploading photos in the background...").into(),
                    ])
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .into()
            } else {
                iced::widget::Space::new().into()
            },
            iced::widget::bottom_right(iced::widget::text(info).size(12))
                .padding(4)
                .into(),
        ])
        .into()
    }
}
