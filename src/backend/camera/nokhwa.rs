use anyhow::Context;
use nokhwa::{
    self,
    pixel_format::RgbAFormat,
    utils::{CameraInfo, RequestedFormat},
    Camera, NokhwaError,
};
use tokio::sync::oneshot;

/// A camera backend using nokhwa to read from webcams
#[derive(Debug, Clone, Copy)]
pub struct NokhwaCameraBackend {}

#[async_trait::async_trait]
impl super::CameraBackend for NokhwaCameraBackend {
    type Error = NokhwaError;

    async fn initialize(&self) -> Result<(), Self::Error> {
        let (tx, rx) = oneshot::channel::<bool>();

        nokhwa::nokhwa_initialize(move |success| {
            let _ = tx.send(success);
        });

        Ok(rx
            .await
            .unwrap()
            .then(|| ())
            .map_err(|_| NokhwaError::GeneralError("failed to initialize backend".into()))?)
    }

    fn enumerate(&self) -> Result<Vec<dyn super::CameraBackendHandle>, Self::Error> {
        if !nokhwa::nokhwa_check() {
            return Err(NokhwaError::UnitializedError);
        }
        let cameras = nokhwa::query(nokhwa::utils::ApiBackend::Auto)?;
        let handles: Vec<dyn super::CameraBackendHandle> = cameras
            .into_iter()
            .map(|info| NokhwaCameraHandle { info })
            .collect();
        Ok(handles)
    }

    fn open_default(&self) -> Result<Option<Box<dyn super::Camera>>, Self::Error> {
        // Use heuristics to find the "default" camera.
        // We prefer non-"integrated" cameras and pick the lowest index.
        let cameras = self.enumerate()?;
        let default_camera = cameras.into_iter().min_by_key(|handle| {
            let nokhwa_handle = handle
                .as_any()
                .downcast_ref::<NokhwaCameraHandle>()
                .unwrap();
            let info = &nokhwa_handle.info;
            let integrated_penalty = if info.friendly_name.to_lowercase().contains("integrated") {
                1
            } else {
                0
            };
            (integrated_penalty, info.index().index)
        });
        Ok(default_camera.map(|c| Box::new(c)))
    }
}

pub struct NokhwaCameraHandle {
    info: CameraInfo,
}

impl super::CameraBackendHandle for NokhwaCameraHandle {
    fn open(&self) -> std::result::Result<Box<dyn super::Camera>, anyhow::Error> {
        Ok(Box::new(NokhwaCamera::new(self.info.clone())))
    }
}

pub struct NokhwaCamera {
    info: CameraInfo,
    video_camera: Option<Camera>,
    still_camera: Option<Camera>,
}

impl NokhwaCamera {
    fn new(info: CameraInfo) -> Self {
        NokhwaCamera {
            info,
            video_camera: None,
            still_camera: None,
        }
    }
}

impl super::Camera for NokhwaCamera {
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
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
            .with_context("couldn't capture still frame")?
            .decode_image::<RgbAFormat>()
            .with_context("couldn't decode still frame")
    }

    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        if self.video_camera.is_none() {
            self.still_camera = None; // drop the high-res still camera
            let mut camera = Camera::new(
                self.info.index().clone(),
                RequestedFormat::new::<RgbAFormat>(
                    nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
                ),
            )?;
            camera.open_stream()?;
            self.video_camera = Some(camera);
        }
        let camera = self.video_camera.as_mut().unwrap();
        camera
            .frame()
            .with_context("couldn't capture preview frame")?
            .decode_image::<RgbAFormat>()
            .with_context("couldn't decode preview frame")
    }
}
