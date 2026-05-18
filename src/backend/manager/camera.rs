use std::sync::Arc;

use image::RgbaImage;

use crate::backend::camera::{Camera, CameraBackend};

const ERROR_BACKOFF_TIME: std::time::Duration = std::time::Duration::from_secs(1);

/// A manager that handles connections to the camera backend.
///
/// Cloning is cheap and shares the same underlying camera backend.
#[derive(Debug, Clone)]
pub struct CameraManager {
    current_frame: Arc<std::sync::Mutex<Option<image::RgbaImage>>>,
    reconnecting: Arc<std::sync::Mutex<bool>>,
    frame_still_command_tx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Sender<()>>>,
    frame_still_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<RgbaImage>>>,
    config: CameraManagerConfig,
    _task: Arc<tokio::task::JoinHandle<()>>,
}

/// Configration for a camera manager
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CameraManagerConfig {
    /// Whether to not mirror the captured photos
    #[serde(default)]
    pub no_mirror_capture: bool,
}

impl CameraManager {
    /// Create a new [`CameraManager`] with the given camera backend and starts
    /// the worker
    pub async fn new(
        camera_backend: Box<dyn CameraBackend>,
        config: CameraManagerConfig,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<()>), anyhow::Error> {
        let initial_camera = match camera_backend.open_default().await {
            Ok(Some(camera)) => camera,
            Ok(None) => {
                log::error!("Camera backend does not have a default camera");
                anyhow::bail!("Camera backend does not have a default camera");
            }
            Err(e) => {
                log::error!("Failed to open default camera from backend: {:?}", e);
                anyhow::bail!("Failed to open default camera from backend: {:?}", e);
            }
        };
        Ok(Self::with_camera(camera_backend, initial_camera, config))
    }

    /// Starts the worker that manages the camera connection and frame
    /// and initializes the camera manager
    ///
    /// It returns a receiver that outputs `()` when a new preview frame is
    /// available.
    pub fn with_camera(
        camera_backend: Box<dyn CameraBackend>,
        initial_camera: Box<dyn Camera>,
        config: CameraManagerConfig,
    ) -> (Self, tokio::sync::mpsc::Receiver<()>) {
        let current_frame = Arc::new(std::sync::Mutex::new(None));
        // TODO: reconnecting seems to not reset to false after a successful reconnection
        // might be a race condition with the `select!`
        let reconnecting = Arc::new(std::sync::Mutex::new(false));
        let (frame_still_command_tx, mut frame_still_command_rx) = tokio::sync::mpsc::channel(1);
        let (frame_still_tx, frame_still_rx) = tokio::sync::mpsc::channel(1);

        let (tx, rx) = tokio::sync::mpsc::channel(10);
        (
            Self {
                current_frame: current_frame.clone(),
                reconnecting: reconnecting.clone(),
                frame_still_command_tx: Arc::new(tokio::sync::Mutex::new(frame_still_command_tx)),
                frame_still_rx: Arc::new(tokio::sync::Mutex::new(frame_still_rx)),
                config,
                // It's very clunky to have a worker task for this, but Camera
                // is not Sync, so we can't share it across threads without a
                // worker task to manage it. (this is mostly a limitation of nokhwa)
                _task: std::sync::Arc::new(tokio::task::spawn(async move {
                    let mut camera = initial_camera;
                    loop {
                        tokio::select! {
                            // A command to capture a still frame was received
                            Some(_) = frame_still_command_rx.recv() => {
                                let frame = loop {
                                    // capturing a frame is a blocking operation
                                    match tokio::task::block_in_place(|| camera.frame_still()) {
                                        Ok(frame) => {
                                            break frame;
                                        }
                                        Err(e) => {
                                            log::error!("Failed to capture still frame from camera: {:?}", e);
                                            *reconnecting.lock().unwrap() = true;
                                            tokio::time::sleep(ERROR_BACKOFF_TIME).await;
                                            match camera_backend.open_default().await {
                                                Ok(Some(new_camera)) => {
                                                    log::info!("Successfully reconnected to camera backend");
                                                    *reconnecting.lock().unwrap() = false;
                                                    camera = new_camera;
                                                },
                                                Ok(None) => {
                                                    log::error!("Camera backend does not have a default camera");
                                                    continue;
                                                }
                                                Err(e) => {
                                                    log::error!("Failed to open default camera from backend: {:?}", e);
                                                    continue;
                                                }
                                            };
                                        }
                                    }
                                };
                                log::trace!("Captured still frame from camera, sending to channel");
                                let _ = frame_still_tx.send(frame).await;
                            }

                            // Capture a preview frame and send a message that a new preview is available
                            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {
                                // capturing a frame is a blocking operation
                                match tokio::task::block_in_place(|| camera.frame_preview()) {
                                    Ok(frame) => {
                                        *current_frame.lock().unwrap() = Some(frame);
                                        let _ = tx.send(()).await;
                                    }
                                    Err(e) => {
                                        log::error!("Failed to capture preview frame from camera: {:?}", e);
                                        *reconnecting.lock().unwrap() = true;
                                        tokio::time::sleep(ERROR_BACKOFF_TIME).await;
                                        match camera_backend.open_default().await {
                                            Ok(Some(new_camera)) => {
                                                log::info!("Successfully reconnected to camera backend");
                                                *reconnecting.lock().unwrap() = false;
                                                camera = new_camera;
                                            },
                                            Ok(None) => {
                                                log::error!("Camera backend does not have a default camera");
                                                continue;
                                            }
                                            Err(e) => {
                                                log::error!("Failed to open default camera from backend: {:?}", e);
                                                continue;
                                            }
                                        };
                                    }
                                }
                            }
                        }
                    }
                })),
            },
            rx,
        )
    }

    /// Capture a still frame from the camera and return it
    ///
    /// Internally, this sends a command to the camera worker to capture a still
    /// frame, and waits for the result to be sent back through a channel.
    pub async fn frame_still(&self) -> Result<image::RgbaImage, anyhow::Error> {
        let command_tx = self.frame_still_command_tx.lock().await;
        command_tx.send(()).await?;
        let mut frame_still_rx = self.frame_still_rx.lock().await;
        let mut frame = frame_still_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("camera worker task was unexpectedly closed"))?;
        if !self.config.no_mirror_capture {
            image::imageops::flip_horizontal_in_place(&mut frame);
        }
        Ok(frame)
    }

    /// Take the latest preview frame from the camera, if available
    ///
    /// Note that this is a "take" operation that consumes the frame, so
    /// subsequent calls will return None until a new preview frame is available.
    pub fn take_frame_preview(&self) -> Option<image::RgbaImage> {
        self.current_frame.lock().unwrap().take()
    }

    /// Get whether the camera backend is currently trying to reconnect
    pub fn is_reconnecting(&self) -> bool {
        *self.reconnecting.lock().unwrap()
    }
}
