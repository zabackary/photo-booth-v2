use anyhow::Context;
use nokhwa::{
    self, Camera,
    pixel_format::RgbAFormat,
    utils::{CameraInfo, RequestedFormat},
};
use tokio::sync::oneshot;

/// A camera backend using nokhwa to read from webcams
#[derive(Debug, Clone, Copy)]
pub struct NokhwaCameraBackend {
    fast_capture: bool,
}

async fn initialize_nokhwa() -> Result<(), anyhow::Error> {
    let (tx, rx) = oneshot::channel::<bool>();
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    nokhwa::nokhwa_initialize(move |success| {
        log::trace!(
            "Nokhwa initialization callback called with success={}",
            success
        );
        // presumably this callback is only called once, but it has an `Fn`
        // signature so according to the type system it could be called
        // multiple times
        if let Some(tx) = tx.lock().unwrap().take() {
            let _ = tx.send(success);
        }
    });

    rx.await
        .unwrap()
        .then_some(())
        .with_context(|| "failed to initialize nokhwa camera backend")
}

impl NokhwaCameraBackend {
    pub async fn new(fast_capture: bool) -> Result<Self, anyhow::Error> {
        initialize_nokhwa().await?;
        Ok(NokhwaCameraBackend { fast_capture })
    }
}

#[async_trait::async_trait]
impl super::CameraBackend for NokhwaCameraBackend {
    async fn enumerate(&self) -> Result<Vec<Box<dyn super::CameraBackendHandle>>, anyhow::Error> {
        let cameras = nokhwa::query(nokhwa::utils::ApiBackend::Auto)?;
        let handles: Vec<Box<dyn super::CameraBackendHandle>> = cameras
            .into_iter()
            .map(|info| {
                Box::new(NokhwaCameraHandle {
                    info,
                    fast_capture: self.fast_capture,
                }) as Box<dyn super::CameraBackendHandle>
            })
            .collect();
        Ok(handles)
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Camera>>, anyhow::Error> {
        // Use heuristics to find the "default" camera.
        // We prefer non-"integrated" cameras and pick the lowest index.
        let cameras = nokhwa::query(nokhwa::utils::ApiBackend::Auto)?;
        let default_camera = cameras.into_iter().min_by_key(|info| {
            let integrated = info.human_name().to_lowercase().contains("integrated");
            (integrated, info.index().as_index().unwrap_or(0))
        });
        Ok(default_camera.map(|info| {
            Box::new(NokhwaCamera::new(info, self.fast_capture)) as Box<dyn super::Camera>
        }))
    }
}

#[derive(Debug, Clone)]
pub struct NokhwaCameraHandle {
    info: CameraInfo,
    fast_capture: bool,
}

impl std::fmt::Display for NokhwaCameraHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl super::CameraBackendHandle for NokhwaCameraHandle {
    fn open(&self) -> std::result::Result<Box<dyn super::Camera>, anyhow::Error> {
        Ok(Box::new(NokhwaCamera::new(
            self.info.clone(),
            self.fast_capture,
        )))
    }
}

pub struct NokhwaCamera {
    info: CameraInfo,
    video_camera: Option<Camera>,
    still_camera: Option<Camera>,
    fast_capture: bool,
}

impl std::fmt::Debug for NokhwaCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NokhwaCamera(info={}, video_camera={}, still_camera={})",
            self.info,
            self.video_camera.is_some(),
            self.still_camera.is_some()
        )
    }
}

impl NokhwaCamera {
    fn new(info: CameraInfo, fast_capture: bool) -> Self {
        NokhwaCamera {
            info,
            video_camera: None,
            still_camera: None,
            fast_capture,
        }
    }
}

impl super::Camera for NokhwaCamera {
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        if self.fast_capture {
            // If fast_capture is enabled, we only use the video camera
            // This means that the captured frame may worse
            return self.frame_preview();
        }
        if self.still_camera.is_none() {
            self.video_camera = None; // drop the fast-taking video camera
            let mut camera = Camera::new(
                self.info.index().clone(),
                RequestedFormat::new::<RgbAFormat>(
                    nokhwa::utils::RequestedFormatType::AbsoluteHighestResolution,
                ),
            )?;
            camera.open_stream()?;
            self.still_camera = Some(camera);
        }
        let camera = self.still_camera.as_mut().unwrap();
        camera
            .frame()
            .with_context(|| "couldn't capture still frame")?
            .decode_image::<RgbAFormat>()
            .with_context(|| "couldn't decode still frame")
    }

    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        if self.video_camera.is_none() {
            self.still_camera = None; // drop the high-res still camera
            let mut camera = Camera::new(
                self.info.index().clone(),
                RequestedFormat::new::<RgbAFormat>(if self.fast_capture {
                    nokhwa::utils::RequestedFormatType::AbsoluteHighestResolution
                } else {
                    nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate
                }),
            )?;
            camera.open_stream()?;
            self.video_camera = Some(camera);
        }
        let camera = self.video_camera.as_mut().unwrap();
        camera
            .frame()
            .with_context(|| "couldn't capture preview frame")?
            .decode_image::<RgbAFormat>()
            .with_context(|| "couldn't decode preview frame")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_nokhwa() -> Result<(), anyhow::Error> {
        initialize_nokhwa().await?;
        Ok(())
    }
}
