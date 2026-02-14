use anyhow::Context;
#[cfg(feature = "storage_google_drive")]
use anyhow::Context as _;

#[cfg(feature = "storage_google_drive")]
use crate::backend::storage::google_drive::GoogleAuthenticationManager;
use crate::backend::{
    camera::CameraBackend, email::EmailBackend, printer::PrinterBackend, renderer::RendererBackend,
    storage::StorageBackend,
};

mod camera;
mod email;
mod printer;
mod renderer;
mod storage;

/// A manager that handles connections to various backends for the rest of the application
///
/// It's responsible for a variety of tasks:
///
/// * Initializing backends from a configuration
/// * Recovering from backend errors (e.g. connection loss to a camera or printer)
/// * Handling the print queue and retrying failed prints
/// * Handling the email queue
/// * Handling the storage of photos and metadata
#[derive(Debug, Clone)]
pub struct BackendManager {
    pub camera_manager: camera::CameraManager,
    pub printer_manager: Option<printer::PrinterManager>,
    pub email_manager: Option<email::EmailManager>,
    pub storage_manager: storage::StorageManager,
    pub renderer_manager: renderer::RendererManager,

    camera_frame_rx: std::sync::Arc<std::sync::Mutex<tokio::sync::mpsc::Receiver<()>>>,
}

impl BackendManager {
    /// Create a new [`BackendManager`] with the given backends
    pub async fn new(
        camera_backend: Box<dyn CameraBackend>,
        storage_backend: Box<dyn StorageBackend>,
        renderer_backend: Box<dyn RendererBackend>,
        printer_backend: Option<(Box<dyn PrinterBackend>, printer::PrinterManagerConfig)>,
        email_backend: Option<(Box<dyn EmailBackend>, email::EmailManagerConfig)>,
    ) -> Result<Self, anyhow::Error> {
        let (camera_manager, camera_frame_rx) = camera::CameraManager::new(camera_backend).await?;
        Ok(BackendManager {
            camera_manager,
            camera_frame_rx: std::sync::Arc::new(std::sync::Mutex::new(camera_frame_rx)),
            printer_manager: match printer_backend {
                Some((backend, config)) => Some(
                    printer::PrinterManager::new(backend, config)
                        .await
                        .context("failed to initialize printer manager")?,
                ),
                None => None,
            },
            email_manager: email_backend
                .map(|(backend, config)| email::EmailManager::new(backend, config)),
            storage_manager: storage::StorageManager::new(storage_backend),
            renderer_manager: renderer::RendererManager::new(renderer_backend),
        })
    }

    /// Create a new [`BackendManager`] from a user-provided configuration
    ///
    /// This initializes all backends specified in the configuration, and
    /// returns an error if any backend fails to initialize.
    pub async fn from_config(config: &crate::config::Config) -> Result<Self, anyhow::Error> {
        let camera_backend = match config.camera {
            #[cfg(feature = "camera_gphoto2")]
            crate::config::CameraConfig::GPhoto2 => {
                Box::new(crate::backend::camera::gphoto2::GPhoto2CameraBackend::new())
                    as Box<dyn CameraBackend>
            }
            #[cfg(feature = "camera_nokhwa")]
            crate::config::CameraConfig::Nokhwa => {
                Box::new(crate::backend::camera::nokhwa::NokhwaCameraBackend::new().await?)
                    as Box<dyn CameraBackend>
            }
            #[cfg(feature = "mock")]
            crate::config::CameraConfig::Mock => {
                Box::new(crate::backend::camera::mock::MockCameraBackend {})
                    as Box<dyn CameraBackend>
            }
        };
        let storage_backend = match config.storage {
            #[cfg(feature = "storage_google_drive")]
            crate::config::StorageConfig::GoogleDrive { ref folder_id } => Box::new(
                crate::backend::storage::google_drive::GoogleDriveStorageBackend::new(
                    folder_id.clone(),
                    GoogleAuthenticationManager::from_service_account_key(
                        crate::backend::storage::google_drive::GoogleDriveStorageBackend::OAUTH_SCOPES
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        config.google_service_account_key_file
                            .as_ref()
                            .context("google_service_account_key_file must be provided for Google Drive storage backend")?,
                    ).await?,
                )
                .await?
            ) as Box<dyn StorageBackend>,
            #[cfg(feature = "storage_local_filesystem")]
            crate::config::StorageConfig::LocalFilesystem { ref path } => Box::new(
                crate::backend::storage::local_filesystem::LocalFilesystemStorageBackend::new(
                    path.clone(),
                )
            ) as Box<dyn StorageBackend>,
            #[cfg(feature = "mock")]
            crate::config::StorageConfig::Mock => Box::new(crate::backend::storage::mock::MockStorageBackend {}),
        };
        let renderer_backend = match config.renderer {
            #[cfg(feature = "renderer_simple")]
            crate::config::RendererConfig::Simple { ref templates } => Box::new(
                crate::backend::renderer::simple::SimpleRendererBackend::new(templates.clone()),
            )
                as Box<dyn RendererBackend>,
        };
        let printer_backend = match &config.printer {
            Some(config_printer) => {
                let backend = match config_printer.backend {
                    #[cfg(feature = "mock")]
                    crate::config::PrinterBackendConfig::Mock => {
                        Box::new(crate::backend::printer::mock::MockPrinterBackend {})
                            as Box<dyn PrinterBackend>
                    }
                };
                Some((
                    backend,
                    printer::PrinterManagerConfig {
                        auto_format: config_printer.auto_format,
                        horizontal_resolution: config_printer.horizontal_resolution,
                        vertical_resolution: config_printer.vertical_resolution,
                        scale: config_printer.scale,
                    },
                ))
            }
            None => None,
        };
        let email_backend = match &config.email {
            Some(config_email) => Some((match config_email {
                #[cfg(feature = "email_gapps_script_webhook")]
                crate::config::EmailConfig::GappsScriptWebhook { endpoint } => Box::new(
                    crate::backend::email::gapps_script_webhook::GappsScriptWebhookEmailBackend::new(
                        reqwest::Url::parse(endpoint)
                            .context("invalid URL for Google Apps Script webhook email backend")?,
                        GoogleAuthenticationManager::from_service_account_key(
                            crate::backend::storage::google_drive::GoogleDriveStorageBackend::OAUTH_SCOPES
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                            config.google_service_account_key_file
                                .as_ref()
                                .context("google_service_account_key_file must be provided for Google Drive storage backend")?,
                        ).await?)
                ) as Box<dyn EmailBackend>,
                #[cfg(feature = "mock")]
                crate::config::EmailConfig::Mock => Box::new(crate::backend::email::mock::MockEmailBackend {}) as Box<dyn EmailBackend>,
            }, email::EmailManagerConfig {
                contact_email: config.contact_email.clone().unwrap_or_default(),
                description: config.description.clone().unwrap_or_default(),
                event_name: config.event_name.clone(),
                palette: iced::theme::palette::Extended::generate(config.theme.into()),
            })),
            None => None
        };

        BackendManager::new(
            camera_backend,
            storage_backend,
            renderer_backend,
            printer_backend,
            email_backend,
        )
        .await
    }
}
