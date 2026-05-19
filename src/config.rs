#[cfg(feature = "storage_google_drive")]
use std::path::PathBuf;

/// The configuration for the application, including which backends to use and their settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// The camera backend to use, along with its settings
    pub camera: CameraConfig,
    /// The printer backend to use, along with its settings
    #[serde(default)]
    pub printer: Option<PrinterConfig>,
    /// The email backend to use, along with its settings
    #[serde(default)]
    pub email: Option<EmailConfig>,
    /// The storage backend to use, along with its settings
    pub storage: StorageConfig,
    /// A list of renders to draw from the captured photos
    ///
    /// Renders can be customized by providing frames and filters.
    pub renders: Vec<RenderConfig>,

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

/// The type of camera backend and its configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraConfig {
    /// The type of camera backend and its configuration
    #[serde(flatten)]
    pub backend: CameraBackendConfig,

    /// The aspect ratio to use when previewing the camera
    pub preview_aspect_ratio: f32,

    /// The backend configuration
    #[serde(flatten)]
    pub manager_config: crate::backend::manager::camera::CameraManagerConfig,
}

/// The camera backend and its configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CameraBackendConfig {
    /// The gphoto2 camera backend, which uses the gphoto2 library to connect to cameras
    #[cfg(feature = "camera_gphoto2")]
    GPhoto2,
    /// The nokhwa camera backend, which uses the nokhwa library to connect to webcams
    #[cfg(feature = "camera_nokhwa")]
    Nokhwa {
        /// Whether to use the same webcam profile for both preview and capture
        #[serde(default, rename = "fastCapture")]
        fast_capture: bool,
    },
    /// A mock camera backend that simulates a camera for testing and development
    #[cfg(feature = "mock")]
    Mock,
}

/// The type of printer backend and its related configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterConfig {
    /// The type of printer backend and its related configuration
    #[serde(flatten)]
    pub backend: PrinterBackendConfig,

    /// Whether to prompt the user to choose how many copies to print
    pub copies_prompt: bool,

    /// Default number of copies to print for each photo strip.
    #[serde(default = "default_copies")]
    pub copies: u32,

    /// Minimum number of copies to allow when prompting the user for how many copies to print
    pub copies_min: Option<u32>,

    /// Maximum number of copies to allow when prompting the user for how many copies to print
    pub copies_max: Option<u32>,

    /// Whether to automatically duplicate a photo strip with a aspect ratio
    /// less than half of the width of the paper to fill the paper when printing
    pub auto_format: bool,

    /// The horizontal resolution of the image to send to the printer
    ///
    /// For the Canon Selphy CP1500 printer printing Postcard, this should be
    /// set to 300 dpi * 4 inches = 1179 pixels (using mm).
    pub horizontal_resolution: u32,

    /// The vertical resolution of the image to send to the printer
    ///
    /// For the Canon Selphy CP1500 printer printing Postcard, this should be
    /// set to 300 dpi * 6 inches = 1746 pixels (using mm).
    pub vertical_resolution: u32,

    /// How much to scale the photo strip when printing, as a percentage
    /// of the original size. This can be used to fit the photo strip better on
    /// the paper
    ///
    /// The output resolution sent to the printer will be the same, but the
    /// actual print will be scaled by this factor
    #[serde(default = "default_scale")]
    pub scale: f32,

    /// A file to log print quantities to for billing or analytics purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_log_file: Option<PathBuf>,
}

fn default_copies() -> u32 {
    1
}

fn default_scale() -> f32 {
    1.0
}

/// The type of printer backend and its related configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PrinterBackendConfig {
    /// The CUPS printer backend, which uses the CUPS library to connect to printers
    #[cfg(feature = "printer_cups")]
    Cups {
        /// The name of the default printer to use, if any. If not specified, the default printer configured in CUPS will be used.
        #[serde(skip_serializing_if = "Option::is_none", rename = "defaultPrinterName")]
        default_printer_name: Option<String>,
        /// The media to use when printing, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        media: Option<String>,
    },
    /// A mock printer backend that simulates a printer for testing and development
    #[cfg(feature = "mock")]
    Mock,
}

/// The email backend and its configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
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
}

/// The storage backend and its configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageConfig {
    /// The Google Drive storage backend, which uses the Google Drive API to store photos and photo strips
    #[cfg(feature = "storage_google_drive")]
    GoogleDrive {
        /// The ID of the Google Drive folder where photos and photo strips are saved
        #[serde(rename = "folderId")]
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

/// The configuration to use while rendering photos
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderConfig {
    /// The template to use to render
    #[serde(flatten)]
    pub template: crate::backend::renderer::Template,

    /// The filters to apply to the photos when rendering
    #[serde(default)]
    pub filters: Vec<crate::backend::renderer::Filter>,
}

/// The theme to use for the frontend
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ThemeConfig {
    background: hex_color::HexColor,
    text: hex_color::HexColor,
    primary: hex_color::HexColor,
    success: hex_color::HexColor,
    danger: hex_color::HexColor,
    warning: hex_color::HexColor,
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
            warning: iced::Color::from_rgba8(
                config.warning.r,
                config.warning.g,
                config.warning.b,
                config.warning.a as f32 / 255.0,
            ),
        }
    }
}
