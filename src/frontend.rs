pub mod camera_feed;
pub mod loading_spinners;
pub mod main_app;
pub mod setup;
pub mod title_overlay;

use std::time::Duration;

use super::backend::manager::BackendManager;
use iced::{Font, Task, keyboard::Key, theme::Palette};
use main_app::{MainApp, MainAppMessage};
use setup::{Setup, SetupMessage};

enum AppPage {
    Setup(Setup),
    MainApp(MainApp),
}

pub struct PhotoBoothApplication {
    page: AppPage,
    manager: BackendManager,
}

#[derive(Debug, Clone)]
pub enum PhotoBoothMessage {
    Setup(SetupMessage),
    MainApp(MainAppMessage),
    Tick,
    SpaceReleased,
    EscapeReleased,
    UpReleased,
    DownReleased,
    OtherKeyRelease,
}

#[derive(Debug, Clone, Copy)]
enum KeyMessage {
    Space,
    Up,
    Down,
    Escape,
}

impl PhotoBoothApplication {
    pub fn new(manager: BackendManager) -> Self {
        Self {
            page: AppPage::Setup(Setup::new()),
            manager,
        }
    }

    pub fn update(&mut self, message: PhotoBoothMessage) -> Task<PhotoBoothMessage> {
        match message {
            PhotoBoothMessage::Setup(msg) => match &mut self.page {
                AppPage::Setup(page) => {
                    let update_task = page.update(msg).map(PhotoBoothMessage::Setup);
                    if let Some(new_page) = page.new_page.take() {
                        let (new_page, new_task) = *new_page;
                        self.page = new_page;
                        update_task.chain(new_task)
                    } else {
                        update_task
                    }
                }
                _ => Task::none(),
            },
            PhotoBoothMessage::MainApp(msg) => match &mut self.page {
                AppPage::MainApp(page) => {
                    let update_task = page
                        .update(msg, self.server_backend.clone())
                        .map(PhotoBoothMessage::MainApp);
                    if let Some(new_page) = page.new_page.take() {
                        let (new_page, new_task) = *new_page;
                        self.page = new_page;
                        update_task.chain(new_task)
                    } else {
                        update_task
                    }
                }
                _ => Task::none(),
            },
            PhotoBoothMessage::Tick => match &mut self.page {
                AppPage::MainApp(page) => page
                    .update(MainAppMessage::Tick, self.server_backend.clone())
                    .map(PhotoBoothMessage::MainApp),
                _ => Task::none(),
            },
            PhotoBoothMessage::SpaceReleased
            | PhotoBoothMessage::DownReleased
            | PhotoBoothMessage::UpReleased
            | PhotoBoothMessage::EscapeReleased => match &mut self.page {
                AppPage::MainApp(page) => page
                    .update(
                        MainAppMessage::KeyReleased(match message {
                            PhotoBoothMessage::SpaceReleased => KeyMessage::Space,
                            PhotoBoothMessage::DownReleased => KeyMessage::Down,
                            PhotoBoothMessage::UpReleased => KeyMessage::Up,
                            PhotoBoothMessage::EscapeReleased => KeyMessage::Escape,
                            _ => unreachable!(),
                        }),
                        self.server_backend.clone(),
                    )
                    .map(PhotoBoothMessage::MainApp),
                _ => Task::none(),
            },
            PhotoBoothMessage::OtherKeyRelease => match &mut self.page {
                AppPage::MainApp(page) => page
                    .update(MainAppMessage::OtherKeyPress, self.server_backend.clone())
                    .map(PhotoBoothMessage::MainApp),
                _ => Task::none(),
            },
        }
    }

    pub fn view(&self) -> iced::Element<PhotoBoothMessage> {
        match &self.page {
            AppPage::MainApp(page) => page
                .view(&self.server_backend)
                .map(PhotoBoothMessage::MainApp),
            AppPage::Setup(page) => page.view().map(PhotoBoothMessage::Setup),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<PhotoBoothMessage> {
        const FPS: f32 = 30.0;
        iced::Subscription::batch([
            iced::time::every(Duration::from_secs_f32(1.0 / FPS))
                .map(|_tick| PhotoBoothMessage::Tick),
            iced::keyboard::on_key_press(|key, _modifiers| match key {
                Key::Named(iced::keyboard::key::Named::Space)
                | Key::Named(iced::keyboard::key::Named::Enter) => {
                    Some(PhotoBoothMessage::SpaceReleased)
                }
                Key::Named(iced::keyboard::key::Named::Escape) => {
                    Some(PhotoBoothMessage::EscapeReleased)
                }
                Key::Named(iced::keyboard::key::Named::PageUp)
                | Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                    Some(PhotoBoothMessage::UpReleased)
                }
                Key::Named(iced::keyboard::key::Named::PageDown)
                | Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                    Some(PhotoBoothMessage::DownReleased)
                }
                _ => Some(PhotoBoothMessage::OtherKeyRelease),
            }),
        ])
    }
}
