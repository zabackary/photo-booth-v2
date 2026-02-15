use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, container, text},
};

/// An internal message for the setup page.
#[derive(Debug, Clone)]
pub enum SetupMessage {
    StartPressed,
    BackendInitialized(Result<crate::backend::manager::BackendManager, String>),
}

/// An action performed by an update to [`Setup`].
#[derive(Debug)]
pub enum SetupAction {
    None,
    Task(Task<SetupMessage>),
    StartMainApp {
        manager: crate::backend::manager::BackendManager,
    },
}

pub struct Setup {
    starting: bool,
    config: &'static crate::config::Config,
}

impl Setup {
    pub fn new(config: &'static crate::config::Config) -> (Self, Task<SetupMessage>) {
        (
            Setup {
                starting: false,
                config,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: SetupMessage) -> SetupAction {
        match message {
            // SetupMessage::CameraSelected(new) => {
            //     self.camera_option = Some(new);
            //     Task::none()
            // }
            SetupMessage::StartPressed => SetupAction::Task(iced::window::oldest().then(|id| {
                iced::Task::batch([
                    iced::window::set_mode(id.unwrap(), iced::window::Mode::Fullscreen),
                    iced::window::toggle_decorations(id.unwrap()),
                    Task::perform(
                        crate::backend::manager::BackendManager::from_config(self.config),
                        |result| {
                            SetupMessage::BackendInitialized(result.map_err(|err| {
                                log::error!("Failed to initialize backends: {:?}", err);
                                err.to_string()
                            }))
                        },
                    ),
                ])
            })),
            SetupMessage::BackendInitialized(Ok(manager)) => {
                log::info!("Successfully initialized backends, starting app");
                SetupAction::StartMainApp { manager }
            }
            SetupMessage::BackendInitialized(Err(err)) => {
                log::error!("Failed to initialize backends: {}", err);
                SetupAction::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, SetupMessage> {
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
                        .on_press_maybe(Some(SetupMessage::StartPressed))
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

    pub fn subscription(&self) -> iced::Subscription<SetupMessage> {
        iced::Subscription::none()
    }
}
