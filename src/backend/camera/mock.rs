use std::fmt::Display;

// 4:3 aspect ratio size
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

/// A camera backend returning blank images for testing purposes
#[derive(Debug, Clone, Copy)]
pub struct MockCameraBackend {}

#[async_trait::async_trait]
impl super::CameraBackend for MockCameraBackend {
    async fn enumerate(&self) -> Result<Vec<Box<dyn super::CameraBackendHandle>>, anyhow::Error> {
        Ok(vec![
            Box::new(MockCameraHandle { integrated: true }) as Box<dyn super::CameraBackendHandle>,
            Box::new(MockCameraHandle { integrated: false }) as Box<dyn super::CameraBackendHandle>,
        ])
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Camera>>, anyhow::Error> {
        log::info!("Opening default mock camera");
        Ok(Some(Box::new(MockCamera {}) as Box<dyn super::Camera>))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockCameraHandle {
    integrated: bool,
}

impl Display for MockCameraHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.integrated {
            write!(f, "Mock Integrated Camera")
        } else {
            write!(f, "Mock External Camera")
        }
    }
}

impl super::CameraBackendHandle for MockCameraHandle {
    fn open(&self) -> Result<Box<dyn super::Camera>, anyhow::Error> {
        log::info!("Opening mock camera: {}", self);
        Ok(Box::new(MockCamera {}))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockCamera {}

impl super::Camera for MockCamera {
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        Ok(image::RgbaImage::new(WIDTH, HEIGHT))
    }

    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        Ok(image::RgbaImage::new(WIDTH, HEIGHT))
    }
}
