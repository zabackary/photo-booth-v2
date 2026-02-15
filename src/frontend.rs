pub mod camera_feed;
pub mod loading_spinners;
pub mod main_app;
pub mod setup;
pub mod title_overlay;

use std::time::Instant;

use main_app::{MainApp, MainAppMessage};
use setup::{Setup, SetupMessage};

enum AppPage {
    Setup(Setup),
    MainApp(MainApp),
}

pub struct PhotoBoothApplication {
    page: AppPage,
    config: &'static crate::config::Config,
    now: Instant,
}

#[derive(Debug, Clone)]
pub enum PhotoBoothMessage {
    Setup(SetupMessage),
    MainApp(MainAppMessage),
    ToggleFullscreen,
    Quit,
}

impl PhotoBoothApplication {
    pub fn new(config: &'static crate::config::Config) -> (Self, iced::Task<PhotoBoothMessage>) {
        let (setup_page, setup_task) = Setup::new(config);
        (
            PhotoBoothApplication {
                page: AppPage::Setup(setup_page),
                config,
                now: Instant::now(),
            },
            setup_task.map(PhotoBoothMessage::Setup),
        )
    }

    pub fn update(
        &mut self,
        message: PhotoBoothMessage,
        now: Instant,
    ) -> iced::Task<PhotoBoothMessage> {
        self.now = now;

        match message {
            PhotoBoothMessage::Setup(message) => {
                if let AppPage::Setup(page) = &mut self.page {
                    match page.update(message) {
                        setup::SetupAction::None => iced::Task::none(),
                        setup::SetupAction::Task(task) => task.map(PhotoBoothMessage::Setup),
                        setup::SetupAction::StartMainApp { manager } => {
                            let (main_app_page, main_app_task) = MainApp::new(manager, self.config);
                            self.page = AppPage::MainApp(main_app_page);
                            main_app_task.map(PhotoBoothMessage::MainApp)
                        }
                    }
                } else {
                    iced::Task::none()
                }
            }
            PhotoBoothMessage::MainApp(msg) => {
                if let AppPage::MainApp(page) = &mut self.page {
                    match page.update(msg) {
                        main_app::MainAppAction::None => iced::Task::none(),
                        main_app::MainAppAction::Task(task) => task.map(PhotoBoothMessage::MainApp),
                    }
                } else {
                    iced::Task::none()
                }
            }
            PhotoBoothMessage::ToggleFullscreen => iced::window::oldest().then(|id| {
                iced::window::mode(id.unwrap()).then(move |mode| {
                    if mode == iced::window::Mode::Fullscreen {
                        iced::window::set_mode(id.unwrap(), iced::window::Mode::Windowed)
                    } else {
                        iced::window::set_mode(id.unwrap(), iced::window::Mode::Fullscreen)
                    }
                })
            }),
            PhotoBoothMessage::Quit => iced::exit(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, PhotoBoothMessage> {
        match &self.page {
            AppPage::MainApp(page) => page.view().map(PhotoBoothMessage::MainApp),
            AppPage::Setup(page) => page.view().map(PhotoBoothMessage::Setup),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<PhotoBoothMessage> {
        iced::Subscription::batch([
            match &self.page {
                AppPage::MainApp(page) => page.subscription().map(PhotoBoothMessage::MainApp),
                AppPage::Setup(page) => page.subscription().map(PhotoBoothMessage::Setup),
            },
            iced::keyboard::listen().filter_map(|event| {
                if let iced::keyboard::Event::KeyReleased { key, modifiers, .. } = event {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::F11) {
                        log::debug!("Toggling fullscreen mode");
                        Some(PhotoBoothMessage::ToggleFullscreen)
                    } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)
                        && modifiers.control()
                    {
                        log::debug!("Quitting application");
                        Some(PhotoBoothMessage::Quit)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
        ])
    }
}
