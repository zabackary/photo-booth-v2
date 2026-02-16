use iced::{ContentFit, Element, Length, Task};
use image::RgbaImage;

use super::camera_feed::{CameraFeed, CameraFeedOptions};

mod animations;

mod capture_photos;
mod capture_photos_prepare;
mod email_entry;
// mod emailing;
mod pick_strip;
mod preview;
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
    // Emailing(Emailing),
    // StudentIDEntry(StudentIDEntry),
}

#[derive(Debug, Clone)]
pub enum MainAppMessage {
    OnRendered(Result<Vec<image::RgbaImage>, String>),
    OnUploaded(Result<crate::backend::storage::StorageHandle, String>),

    CameraFeed(super::camera_feed::CameraMessage),

    Preview(preview::PreviewMessage),
    CapturePhotosPrepare(capture_photos_prepare::CapturePhotosPrepareMessage),
    CapturePhotos(capture_photos::CapturePhotosMessage),
    Rendering(rendering::RenderingMessage),
    PickStrip(pick_strip::PickStripMessage),
    EmailEntry(email_entry::EmailEntryMessage),
    // PaymentRequired(PaymentRequiredMessage),
    // Emailing(EmailingMessage),
    // RenderedPreview(RenderedPreviewMessage),
}

#[derive(Debug)]
pub enum MainAppAction {
    None,
    Task(Task<MainAppMessage>),
}

/// State needed for the current session
#[derive(Debug, Default)]
pub struct Session {
    captured_photos: Vec<RgbaImage>,
    selected_strip: Option<usize>,
    strips: Option<Vec<RgbaImage>>,
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
                    todo!("show error to user");
                    MainAppAction::None
                }
            },
            MainAppMessage::OnUploaded(result) => match result {
                Ok(handle) => {
                    log::debug!("Successfully uploaded strip with handle {:?}", handle);
                    if let MainAppPage::EmailEntry(email_entry) = &mut self.page {
                        email_entry.on_storage_finish(handle);
                    }
                    MainAppAction::None
                }
                Err(err) => {
                    log::error!("Error uploading strip: {:?}", err);
                    todo!("show error to user");
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
                            let (email_entry, email_entry_task) = email_entry::EmailEntry::new(
                                iced::widget::image::Handle::from_rgba(
                                    strip.width(),
                                    strip.height(),
                                    strip.clone().into_raw(),
                                ),
                                self.manager.clone(),
                            );
                            self.page = MainAppPage::EmailEntry(email_entry);
                            let manager = self.manager.clone();
                            let photos = self.session.captured_photos.clone();
                            MainAppAction::Task(iced::Task::batch([
                                email_entry_task.map(MainAppMessage::EmailEntry),
                                iced::Task::perform(
                                    async move {
                                        // Start uploading the selected strip immediately
                                        manager.storage_manager.store(strip, photos).await.map_err(
                                            |err| {
                                                log::error!("Failed to upload strip: {:?}", err);
                                                err.to_string()
                                            },
                                        )
                                    },
                                    MainAppMessage::OnUploaded,
                                ),
                            ]))
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
            MainAppMessage::EmailEntry(message) => {
                if let MainAppPage::EmailEntry(email_entry) = &mut self.page {
                    match email_entry.update(message) {
                        email_entry::EmailEntryAction::Submit { emails } => {
                            todo!("handle email entry completion")
                        }
                        email_entry::EmailEntryAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::EmailEntry))
                        }
                        email_entry::EmailEntryAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            } // MainAppMessage::Emailing(message) => {
              //     if let MainAppPage::Emailing(emailing) = &mut self.page {
              //         match emailing.update(message) {
              //             emailing::EmailingAction::Complete => {
              //                 todo!("handle emailing completion")
              //             }
              //             emailing::EmailingAction::Task(task) => {
              //                 MainAppAction::Task(task.map(MainAppMessage::Emailing))
              //             }
              //             emailing::EmailingAction::None => MainAppAction::None,
              //         }
              //     } else {
              //         MainAppAction::None
              //     }
              // MainAppMessage::CapturePhotos(msg) => match &mut self.page {
              //     MainAppPage::CapturePhotos(capture_photos) => {
              //         let task = capture_photos.update(msg, &mut self.captured_photos);

              //         match task {
              //             Some(CapturePhotosEffect::CaptureStill) => {
              //                 Task::done(MainAppMessage::CaptureStill)
              //             }
              //             Some(CapturePhotosEffect::PhotosComplete { photos }) => {
              //                 // Process captured photos
              //                 self.previews.clear();
              //                 for photo in &photos {
              //                     self.previews.push(iced::widget::image::Handle::from_rgba(
              //                         photo.width(),
              //                         photo.height(),
              //                         photo.as_raw().clone(),
              //                     ));
              //                 }

              //                 let strip = render_take(photos.clone());
              //                 let strip_handle = Handle::from_rgba(
              //                     strip.width(),
              //                     strip.height(),
              //                     strip.as_raw().clone(),
              //                 );

              //                 let rendered_preview = RenderedPreview::new(strip_handle);
              //                 self.page = MainAppPage::RenderedPreview(rendered_preview);

              //                 let future = server_backend.upload_photo(strip.clone(), photos);
              //                 Task::perform(future, |result| {
              //                     MainAppMessage::UploadFinished(result.map_err(|x| x.to_string()))
              //                 })
              //             }
              //             None => Task::none(),
              //         }
              //     }
              //     _ => Task::none(),
              // },
              // MainAppMessage::RenderedPreview(msg) => match &mut self.page {
              //     MainAppPage::RenderedPreview(rendered_preview) => {
              //         let task = rendered_preview.update(msg);

              //         match task {
              //             Some(RenderedPreviewEffect::Complete) => {
              //                 // let email_entry =
              //                 //     EmailEntry::new(rendered_preview.strip_handle.clone());
              //                 let email_entry = todo!();
              //                 self.page = MainAppPage::EmailEntry(email_entry);
              //                 iced::widget::text_input::focus("email_input")
              //             }
              //             None => Task::none(),
              //         }
              //     }
              //     _ => Task::none(),
              // },
              // MainAppMessage::Emailing(msg) => match &mut self.page {
              //     MainAppPage::Emailing(emailing) => {
              //         let task = emailing.update(msg);

              //         match task {
              //             Some(EmailingEffect::Complete) => {
              //                 self.page = MainAppPage::PaymentRequired(PaymentRequired::new());
              //                 Task::none()
              //             }
              //             None => Task::none(),
              //         }
              //     }
              //     _ => Task::none(),
              // },
              // MainAppMessage::EmailFinished(result) => {
              //     log::debug!("Email result received: {:?}", result);
              //     match &mut self.page {
              //         MainAppPage::Emailing(emailing) => match result {
              //             Ok(_) => {
              //                 emailing.finish();
              //                 Task::none()
              //             }
              //             Err(err) => {
              //                 self.page = MainAppPage::PaymentRequired(PaymentRequired::with_error(
              //                     format!("Failed to email photos: {}", err),
              //                 ));
              //                 log::error!("Error emailing photos: {}", err);
              //                 Task::none()
              //             }
              //         },
              //         _ => Task::none(),
              //     }
              // }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<MainAppMessage> {
        iced::Subscription::batch([
            self.feed.subscription().map(MainAppMessage::CameraFeed),
            match &self.page {
                MainAppPage::Preview(preview) => {
                    preview.subscription().map(MainAppMessage::Preview)
                }
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
                MainAppPage::EmailEntry(email_entry) => {
                    email_entry.subscription().map(MainAppMessage::EmailEntry)
                } // MainAppPage::Emailing(emailing) => emailing.subscription().map(MainAppMessage::Emailing),
            },
        ])
    }

    pub fn view(&self) -> Element<'_, MainAppMessage> {
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
                MainAppPage::EmailEntry(email_entry) => {
                    email_entry.view().map(MainAppMessage::EmailEntry).into()
                } // MainAppPage::Emailing(emailing) => emailing.view().map(MainAppMessage::Emailing),
            },
        ])
        .into()
    }
}
