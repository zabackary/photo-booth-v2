use iced::{
    Alignment, Border, Color, Element, Length, Padding,
    widget::{button, column, container, image, row, space, text},
};

use crate::frontend::{
    loading_spinners,
    main_app::status_overlay,
    title_overlay::{full_title_overlay, supporting_text, title_text},
};

const QR_CODE_QUIET_ZONE: usize = 2;
pub const QR_CODE_VERSION: iced::widget::qr_code::Version =
    iced::widget::qr_code::Version::Normal(5);
const QR_CODE_SIDE_LENGTH: usize = QR_CODE_QUIET_ZONE * 2 + (5 * 4 + 17);

#[derive(Debug)]
pub struct QrCode {
    qr_code_data: Option<iced::widget::qr_code::Data>,
    show_qr_code: bool,
    can_continue: bool,
    strip_handle: iced::widget::image::Handle,

    manager: crate::backend::manager::BackendManager,
}

#[derive(Debug, Clone)]
pub enum QrCodeMessage {
    ContinuePress,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum QrCodeAction {
    Continue,
    Task(iced::Task<QrCodeMessage>),
    None,
}

impl QrCode {
    pub fn new(
        strip_handle: iced::widget::image::Handle,
        manager: crate::backend::manager::BackendManager,
        storage_handle: Option<crate::backend::storage::StorageHandle>,
    ) -> (Self, iced::Task<QrCodeMessage>) {
        let mut new = Self {
            qr_code_data: None,
            show_qr_code: true,
            strip_handle,
            can_continue: false,
            manager,
        };
        if let Some(storage_handle) = storage_handle {
            new.on_storage_finish(storage_handle);
        }
        (new, iced::Task::none())
    }

    pub fn on_storage_finish(&mut self, storage_handle: crate::backend::storage::StorageHandle) {
        if let Some(url) = storage_handle.strip_link() {
            self.qr_code_data = iced::widget::qr_code::Data::with_version(
                &url,
                QR_CODE_VERSION,
                iced::widget::qr_code::ErrorCorrection::Medium,
            )
            .ok();
        } else {
            self.show_qr_code = false;
        }
        self.can_continue = true;
    }

    pub fn update(&mut self, message: QrCodeMessage) -> QrCodeAction {
        match message {
            QrCodeMessage::ContinuePress => {
                if !self.can_continue {
                    // still uploading, can't finish yet
                    return QrCodeAction::None;
                }
                QrCodeAction::Continue
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<QrCodeMessage> {
        iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyReleased {
                key:
                    iced::keyboard::Key::Named(
                        iced::keyboard::key::Named::Enter | iced::keyboard::key::Named::Space,
                    ),
                ..
            } => Some(QrCodeMessage::ContinuePress),
            _ => None,
        })
    }

    pub fn view<'a>(&'a self) -> Element<'a, QrCodeMessage> {
        iced::widget::stack([
            full_title_overlay(
                row([
                    column([
                        title_text("Your photos are printing").width(Length::Shrink).into(),
                        supporting_text(if self.show_qr_code {
                            "They'll be done soon. You can also scan the QR code below to download your original photos."
                        } else {
                            "They'll be done soon!"
                        }).width(Length::Shrink).into(),
                        space().height(12.0).into(),
                        if let Some(ref qr_code_data) = self.qr_code_data && self.show_qr_code {
                            container(
                                iced::widget::qr_code(qr_code_data).cell_size(8).style(|_|iced::widget::qr_code::Style {
                                    background: Color::WHITE,
                                    cell: Color::BLACK
                                })
                            ).width((QR_CODE_SIDE_LENGTH * 8) as f32).height((QR_CODE_SIDE_LENGTH * 8) as f32).padding(8).into()
                        } else if self.show_qr_code {
                            container(column([
                                        loading_spinners::Circular::new()
                                            .size(30.0)
                                            .bar_height(3.0)
                                            .easing(&loading_spinners::easing::STANDARD_DECELERATE)
                                            .into(),
                                        iced::widget::text("Uploading and generating code...").into()
                                    ])
                                    .align_x(iced::Alignment::Center)
                                    .spacing(8)
                            ).style(|_| container::background(Color::WHITE)).padding(8).center((QR_CODE_SIDE_LENGTH * 8) as f32).into()
                        } else {
                            text("Unfortunately, a QR code is not available.").into()
                        },
                        space().height(12.0).into(),
                        button(text(
                            "Press [Enter] to continue")
                        .size(24))
                        .style(|theme: &iced::Theme, status| {
                            let mut normal = button::primary(theme, status);
                            normal.border.radius = 999.0.into();
                            normal
                        })
                        .padding(Padding { bottom: 10.0, left: 24.0, right: 24.0, top: 10.0 })
                        .on_press_maybe(self.can_continue.then_some(QrCodeMessage::ContinuePress))
                        .padding(10)
                        .into()
                    ])
                    .padding(100)
                    .align_x(iced::Alignment::Center)
                    .width(Length::Fill)
                    .into(),
                    space().width(12.0).into(),
                    container(
                        column([
                            supporting_text("Your photos").width(Length::Shrink).into(),
                            space().height(12.0).into(),
                            image(self.strip_handle.clone())
                                .height(Length::Fill)
                                .content_fit(iced::ContentFit::Contain)
                                .into(),
                        ])
                        .align_x(iced::Alignment::Center)
                        .padding(30)
                    ).style(|theme: &iced::Theme| container::Style {
                        background: Some(
                            theme.extended_palette().background.base.color.scale_alpha(0.8).into(),
                        ),
                        border: Border::default().rounded(iced::border::Radius {
                            bottom_left: 24.0,
                            bottom_right: 0.0,
                            top_left: 24.0,
                            top_right: 0.0,
                        }),
                        ..Default::default()
                    }).into()
                ])
                .align_y(iced::Alignment::Center),
            ),
            if self.manager.storage_manager.busy() {
                status_overlay::status_overlay(
                    row([
                        loading_spinners::Circular::new()
                            .size(30.0)
                            .bar_height(3.0)
                            .easing(&loading_spinners::easing::STANDARD_DECELERATE)
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
        ]).into()
    }
}
