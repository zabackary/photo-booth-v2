use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, container, text},
};

use crate::frontend::{AppPage, PhotoBoothMessage};

use super::{camera_feed::CameraFeed, main_app::MainApp};

#[derive(Debug, Clone)]
pub enum SetupMessage {
    StartPressed,
}

pub struct Setup {
    pub new_page: Option<Box<(AppPage, Task<PhotoBoothMessage>)>>,

    manager: crate::backend::manager::BackendManager,
}

impl Setup {
    pub fn new(manager: crate::backend::manager::BackendManager) -> Self {
        Self {
            new_page: None,
            manager,
        }
    }

    pub fn update(&mut self, message: SetupMessage) -> Task<SetupMessage> {
        match message {
            // SetupMessage::CameraSelected(new) => {
            //     self.camera_option = Some(new);
            //     Task::none()
            // }
            SetupMessage::StartPressed => {
                let (feed, task) =
                    CameraFeed::new(self.manager.camera_manager.clone(), Default::default());
                let (app, app_task) = MainApp::new(feed);
                self.new_page = Some(Box::new((
                    AppPage::MainApp(app),
                    Task::batch([
                        // task.map(MainAppMessage::Camera)
                        //     .map(PhotoBoothMessage::MainApp),
                        app_task.map(PhotoBoothMessage::MainApp),
                    ]),
                )));
                iced::window::get_latest().then(|id| {
                    iced::Task::batch([
                        iced::window::change_mode(id.unwrap(), iced::window::Mode::Fullscreen),
                        iced::window::toggle_decorations(id.unwrap()),
                    ])
                })
            }
        }
    }

    pub fn view(&self) -> Element<SetupMessage> {
        container(
            container(
                column([
                    text("Setup").size(32).into(),
                    // pick_list(
                    //     self.camera_options.as_ref(),
                    //     self.camera_option.as_ref(),
                    //     SetupMessage::CameraSelected,
                    // )
                    // .into(),
                    button("Start")
                        .on_press_maybe(
                            self.camera_option
                                .is_some()
                                .then_some(SetupMessage::StartPressed),
                        )
                        .into(),
                ])
                .align_x(Alignment::Center)
                .spacing(8),
            )
            .padding(8)
            .style(container::rounded_box),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}
