use std::fmt::{Debug, Display};

#[cfg(feature = "camera_gphoto2")]
pub mod gphoto2;
#[cfg(feature = "mock")]
pub mod mock;
#[cfg(feature = "camera_nokhwa")]
pub mod nokhwa;

/// A camera backend
#[async_trait::async_trait]
pub trait CameraBackend: Debug + Send + 'static {
    /// Initialize this backend
    async fn initialize(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Enumerate available cameras attached to this backend
    async fn enumerate(&self) -> Result<Vec<Box<dyn CameraBackendHandle>>, anyhow::Error>;

    /// Opens the default camera provided by this backend, if any
    ///
    /// It is up to the backend to determine what the "default" camera is
    async fn open_default(&self) -> Result<Option<Box<dyn Camera>>, anyhow::Error> {
        Ok(None)
    }
}

/// A handle to open a camera
///
/// Its `Display` implementation should provide a user-friendly name for the camera.
pub trait CameraBackendHandle: Debug + Display {
    fn open(&self) -> Result<Box<dyn Camera>, anyhow::Error>;
}

/// A camera that can capture frames
pub trait Camera: Debug + Send {
    /// Capture a still frame from the camera
    ///
    /// This is expected to be a high-resolution frame suitable for saving to disk.
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error>;

    /// Capture a preview frame from the camera
    ///
    /// This is expected to be a lower-resolution frame suitable for real-time preview.
    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error>;
}
