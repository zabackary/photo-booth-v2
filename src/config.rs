#[cfg(feature = "storage_google_drive")]
use std::path::PathBuf;

/// The configration for the application, including which backends to use and their settings
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// The camera backend to use, along with its settings
    pub camera: CameraConfig,
    /// The printer backend to use, along with its settings
    #[serde(default)]
    pub printer: PrinterConfig,
    /// The email backend to use, along with its settings
    #[serde(default)]
    pub email: EmailConfig,
    /// The storage backend to use, along with its settings
    pub storage: StorageConfig,
    /// The renderer backend to use, along with its settings
    pub renderer: RendererConfig,

    /// JSON key file for the service account used to initialize Google API
    /// credentials
    ///
    /// Required for the Google Drive storage backend and the associated Google
    /// Apps Script webhook email backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[cfg(feature = "storage_google_drive")]
    pub google_service_account_key_file: Option<PathBuf>,

    /// The number of photos to take in each photo strip
    ///
    /// This must be compatible with the templates used by the renderer.
    pub photos_per_strip: usize,

    /// The theme to use for the frontend
    pub theme: ThemeConfig,
    /// The name of the event
    pub event_name: String,
    /// A description or privacy message to include in the email and frontend, if applicable
    pub description: Option<String>,
    /// Contact information to include in the email and frontend, if applicable
    pub contact_email: Option<String>,
}

/// The camera backend and its configration
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CameraConfig {
    /// The gphoto2 camera backend, which uses the gphoto2 library to connect to cameras
    #[cfg(feature = "camera_gphoto2")]
    GPhoto2,
    /// The nokhwa camera backend, which uses the nokhwa library to connect to webcams
    #[cfg(feature = "camera_nokhwa")]
    Nokhwa,
    /// A mock camera backend that simulates a camera for testing and development
    #[cfg(feature = "mock")]
    Mock,
}

/// The printer backend and its configration
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PrinterConfig {
    /// A mock printer backend that simulates a printer for testing and development
    #[cfg(feature = "mock")]
    Mock,
    /// Don't print photos
    #[default]
    None,
}

/// The email backend and its configration
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EmailConfig {
    /// An email backend that sends photos via email using a Google Apps Script webhook
    #[cfg(feature = "email_gapps_script_webhook")]
    GappsScriptWebhook {
        /// The URL of the Google Apps Script webhook that will handle sending emails
        endpoint: String,
    },
    /// A mock email backend that simulates an email service for testing and development
    #[cfg(feature = "mock")]
    Mock,
    /// Don't send emails
    #[default]
    None,
}

/// The storage backend and its configration
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StorageConfig {
    /// The Google Drive storage backend, which uses the Google Drive API to store photos and photo strips
    #[cfg(feature = "storage_google_drive")]
    GoogleDrive {
        /// The ID of the Google Drive folder where photos and photo strips are saved
        folder_id: String,
    },
    /// A storage backend that saves locally to disk.
    #[cfg(feature = "storage_local_filesystem")]
    LocalFilesystem {
        /// The path to the directory where photos and photo strips are saved
        path: PathBuf,
    },
    /// A mock storage backend that simulates a storage service for testing and development
    #[cfg(feature = "mock")]
    Mock,
}

/// The renderer backend and its configration
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RendererConfig {
    /// A simple backend that superimposes a template on the captured images
    #[cfg(feature = "renderer_simple")]
    Simple {
        /// The templates to use for rendering
        templates: Vec<crate::backend::renderer::simple::Template>,
    },
}

/// The theme to use for the frontend
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeConfig {
    background: hex_color::HexColor,
    text: hex_color::HexColor,
    primary: hex_color::HexColor,
    success: hex_color::HexColor,
    danger: hex_color::HexColor,
}

impl From<ThemeConfig> for iced::theme::palette::Palette {
    fn from(config: ThemeConfig) -> Self {
        Self {
            background: iced::Color::from_rgba8(
                config.background.r,
                config.background.g,
                config.background.b,
                config.background.a as f32 / 255.0,
            ),
            text: iced::Color::from_rgba8(
                config.text.r,
                config.text.g,
                config.text.b,
                config.text.a as f32 / 255.0,
            ),
            primary: iced::Color::from_rgba8(
                config.primary.r,
                config.primary.g,
                config.primary.b,
                config.primary.a as f32 / 255.0,
            ),
            success: iced::Color::from_rgba8(
                config.success.r,
                config.success.g,
                config.success.b,
                config.success.a as f32 / 255.0,
            ),
            danger: iced::Color::from_rgba8(
                config.danger.r,
                config.danger.g,
                config.danger.b,
                config.danger.a as f32 / 255.0,
            ),
        }
    }
}
