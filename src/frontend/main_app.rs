use iced::{ContentFit, Element, Length, Task};
use image::RgbaImage;

use super::camera_feed::{CameraFeed, CameraFeedOptions};

mod animations;

mod capture_photos;
mod capture_photos_prepare;
// mod email_entry;
// mod emailing;
mod preview;
// mod rendered_preview;
mod status_overlay;

#[derive(Debug)]
enum MainAppPage {
    Preview(preview::Preview),
    CapturePhotosPrepare(capture_photos_prepare::CapturePhotosPrepare),
    CapturePhotos(capture_photos::CapturePhotos),
    // RenderedPreview(RenderedPreview),
    // EmailEntry(EmailEntry),
    // Emailing(Emailing),
    // StudentIDEntry(StudentIDEntry),
}

#[derive(Debug, Clone)]
pub enum MainAppMessage {
    CameraFeed(super::camera_feed::CameraMessage),

    // UploadFinished(Result<crate::backend::storage::StorageHandle, String>),
    // EmailFinished(Result<(), String>),
    // PrintFinished(Result<(), String>),
    Preview(preview::PreviewMessage),
    CapturePhotosPrepare(capture_photos_prepare::CapturePhotosPrepareMessage),
    CapturePhotos(capture_photos::CapturePhotosMessage),
    // EmailEntry(EmailEntryMessage),
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
    strips: Option<Vec<RgbaImage>>,
    strip_handles: Option<Vec<iced::widget::image::Handle>>,
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
            MainAppMessage::CameraFeed(msg) => {
                MainAppAction::Task(self.feed.update(msg).map(MainAppMessage::CameraFeed))
            }
            // MainAppMessage::Tick => match &mut self.page {
            //     MainAppPage::CapturePhotosPrepare { ready_timeline } => {
            //         if ready_timeline.update().is_completed() {
            //             self.page = MainAppPage::CapturePhotos(CapturePhotos::new());
            //         };
            //         Task::none()
            //     }
            //     MainAppPage::CapturePhotos(_capture_photos) => {
            //         Task::done(MainAppMessage::CapturePhotos(CapturePhotosMessage::Tick))
            //     }
            //     MainAppPage::RenderedPreview(_rendered_preview) => Task::done(
            //         MainAppMessage::RenderedPreview(RenderedPreviewMessage::Tick),
            //     ),
            //     MainAppPage::Emailing(emailing) => {
            //         if let Some(effect) = emailing.update(EmailingMessage::Tick) {
            //             match effect {
            //                 EmailingEffect::Complete => {
            //                     self.page = MainAppPage::PaymentRequired(PaymentRequired::new());
            //                 }
            //             }
            //         }
            //         Task::none()
            //     }
            //     _ => Task::none(),
            // },
            // MainAppMessage::UploadFinished(result) => {
            //     log::debug!("Upload result received: {:?}", result);
            //     match result {
            //         Ok(res) => {
            //             // Update email entry with upload data
            //             if let MainAppPage::EmailEntry(ref mut email_entry) = self.page {
            //                 // Store the actual upload handle for later use
            //                 email_entry.set_upload_handle(res.clone());
            //                 email_entry.set_qr_code_url(server_backend.get_link(res));
            //             }
            //             Task::none()
            //         }
            //         Err(err) => {
            //             self.page = MainAppPage::PaymentRequired(PaymentRequired::with_error(
            //                 format!("Failed to upload photos: {}", err),
            //             ));
            //             log::error!("Error uploading photos: {}", err);
            //             Task::none()
            //         }
            //     }
            // }
            // MainAppMessage::KeyReleased(key) => {
            //     log::debug!("Key released: {:?}", key);
            //     match &mut self.page {
            //         MainAppPage::PaymentRequired(_) => match key {
            //             KeyMessage::Up => Task::none(),
            //             KeyMessage::Down => Task::none(),
            //             KeyMessage::Space => {
            //                 self.page = MainAppPage::Preview(Preview::new());
            //                 Task::none()
            //             }
            //             KeyMessage::Escape => iced::widget::text_input::focus("email_input"),
            //         },
            //         MainAppPage::Preview(_) => {
            //             self.page = MainAppPage::CapturePhotosPrepare {
            //                 ready_timeline: animations::ready::animation().begin_animation(),
            //             };
            //             Task::none()
            //         }
            //         MainAppPage::RenderedPreview(_) => Task::done(MainAppMessage::RenderedPreview(
            //             RenderedPreviewMessage::Skip,
            //         )),
            //         MainAppPage::EmailEntry(_) => iced::widget::text_input::focus("email_input"),
            //         MainAppPage::StudentIDEntry(_) => {
            //             iced::widget::text_input::focus("student_id_input")
            //         }
            //         _ => Task::none(),
            //     }
            // }
            // MainAppMessage::OtherKeyPress => match self.page {
            //     MainAppPage::EmailEntry(_) => iced::widget::text_input::focus("email_input"),
            //     MainAppPage::StudentIDEntry(_) => {
            //         iced::widget::text_input::focus("student_id_input")
            //     }
            //     _ => Task::none(),
            // },
            // MainAppMessage::EmailEntry(msg) => match &mut self.page {
            //     MainAppPage::EmailEntry(email_entry) => {
            //         let effect = email_entry.update(msg);

            //         match effect {
            //             Some(EmailEntryEffect::Submit {
            //                 emails,
            //                 // upload_handle,
            //             }) => {
            //                 let emailing = Emailing::new();
            //                 self.page = MainAppPage::Emailing(emailing);
            //                 log::trace!("Sending email with photos...");
            //                 Task::perform(
            //                     server_backend.send_email(
            //                         upload_handle,
            //                         emails,
            //                         None,
            //                         iced::theme::palette::Extended::generate(PALETTE),
            //                     ),
            //                     |result| {
            //                         MainAppMessage::EmailFinished(result.map_err(|x| x.to_string()))
            //                     },
            //                 )
            //             }
            //             None => Task::none(),
            //         }
            //     }
            //     _ => Task::none(),
            // },
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
                            // TODO: process captured photos into strips and move to preview page

                            // // Process captured photos
                            // self.session.captured_photos = photos;
                            // let strip = crate::backend::photo_processing::render_take(
                            //     self.session.captured_photos.clone(),
                            // );
                            // self.session.strips = Some(vec![strip]);
                            // let strip_handle = iced::widget::image::Handle::from_rgba(
                            //     self.session.strips.as_ref().unwrap()[0].width(),
                            //     self.session.strips.as_ref().unwrap()[0].height(),
                            //     self.session.strips.as_ref().unwrap()[0].as_raw().clone(),
                            // );
                            // self.session.strip_handles =
                            //     Some(vec![strip_handle.clone()]);

                            // Move to rendered preview page
                            // self.page = MainAppPage::RenderedPreview(RenderedPreview::new(strip_handle));
                            MainAppAction::None
                        }
                        capture_photos::CapturePhotosAction::Task(task) => {
                            MainAppAction::Task(task.map(MainAppMessage::CapturePhotos))
                        }
                        capture_photos::CapturePhotosAction::None => MainAppAction::None,
                    }
                } else {
                    MainAppAction::None
                }
            } // MainAppMessage::CapturePhotos(msg) => match &mut self.page {
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
                _ => iced::Subscription::none(),
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
                } // MainAppPage::RenderedPreview(rendered_preview) => {
                  //     rendered_preview.view().map(MainAppMessage::RenderedPreview)
                  // }
                  // MainAppPage::EmailEntry(email_entry) => {
                  //     email_entry.view().map(MainAppMessage::EmailEntry).into()
                  // }
                  // MainAppPage::Emailing(emailing) => emailing.view().map(MainAppMessage::Emailing),
            },
        ])
        .into()
    }
}
