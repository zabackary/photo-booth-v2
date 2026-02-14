use std::fmt::Display;

use anyhow::Context as _;
use gphoto2::{Camera, Context, list::CameraDescriptor};

// A camera backend using gphoto2 to read from supported cameras
#[derive(Clone)]
pub struct GPhoto2CameraBackend {
    context: Context,
}

impl std::fmt::Debug for GPhoto2CameraBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPhoto2CameraBackend")
    }
}

impl GPhoto2CameraBackend {
    pub fn new() -> Self {
        GPhoto2CameraBackend {
            context: Context::new().expect("failed to create gphoto2 context"),
        }
    }
}

impl Default for GPhoto2CameraBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl super::CameraBackend for GPhoto2CameraBackend {
    async fn enumerate(&self) -> Result<Vec<Box<dyn super::CameraBackendHandle>>, anyhow::Error> {
        let descriptors = self
            .context
            .list_cameras()
            .await
            .with_context(|| "couldn't list cameras")?;
        let handles: Vec<Box<dyn super::CameraBackendHandle>> = descriptors
            .into_iter()
            .map(|desc| {
                Box::new(CameraDescriptorWrapper(desc)) as Box<dyn super::CameraBackendHandle>
            })
            .collect();
        Ok(handles)
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Camera>>, anyhow::Error> {
        Ok(self.context.autodetect_camera().await.ok().map(|camera| {
            Box::new(GPhoto2Camera::new(self.context.clone(), camera)) as Box<dyn super::Camera>
        }))
    }
}

/// A wrapper around gphoto2's CameraDescriptor as a camera handle
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraDescriptorWrapper(CameraDescriptor);

impl super::CameraBackendHandle for CameraDescriptorWrapper {
    fn open(&self) -> Result<Box<dyn super::Camera>, anyhow::Error> {
        todo!();
    }
}

impl Display for CameraDescriptorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.0.model, self.0.port)
    }
}

/// A camera using gphoto2
#[derive(Clone)]
pub struct GPhoto2Camera {
    camera: Camera,
    context: Context,
}

impl std::fmt::Debug for GPhoto2Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPhoto2Camera")
    }
}

impl GPhoto2Camera {
    pub fn new(context: Context, camera: Camera) -> Self {
        GPhoto2Camera { camera, context }
    }
}

impl super::Camera for GPhoto2Camera {
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        // capture image to camera's internal storage (e.g., SD card)
        let path = self.camera.capture_image().wait()?;
        let fs = self.camera.fs();
        // download and decode that file from the camera's internal storage
        let img = image::load_from_memory(
            &fs.download(&path.folder(), &path.name())
                .wait()
                .with_context(|| "failed to download still image")?
                .get_data(&self.context)
                .wait()
                .with_context(|| "failed to get still image data")?,
        )
        .with_context(|| "failed to decode still image")?;
        Ok(img.to_rgba8())
    }

    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        // decode from memory
        let img = image::load_from_memory(
            &self
                .camera
                // capture a preview image to the camera's internal buffer
                .capture_preview()
                .wait()
                .with_context(|| "failed to capture preview frame")?
                // read that buffer
                .get_data(&self.context)
                .wait()
                .with_context(|| "failed to get preview image data")?,
        )
        .with_context(|| "failed to decode preview image")?;
        Ok(img.to_rgba8())
    }
}
