use std::fmt::Display;

// 4:3 aspect ratio size
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

/// A camera backend returning blank images for testing purposes
#[derive(Debug, Clone, Copy)]
pub struct MockCameraBackend {}

#[async_trait::async_trait]
impl super::CameraBackend for MockCameraBackend {
    type Error = anyhow::Error;

    async fn enumerate(&self) -> Result<Vec<dyn super::CameraBackendHandle>, Self::Error> {
        let mut vec = Vec::<dyn super::CameraBackendHandle>::with_capacity(2);

        vec.push(MockCameraHandle { integrated: true });
        vec.push(MockCameraHandle { integrated: false });

        Ok(vec)
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Camera>>, Self::Error> {
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
        Ok(Box::new(MockCamera {}))
    }
}

pub struct MockCamera {}

impl super::Camera for MockCamera {
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        Ok(image::RgbaImage::new(WIDTH, HEIGHT))
    }

    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        Ok(image::RgbaImage::new(WIDTH, HEIGHT))
    }
}
