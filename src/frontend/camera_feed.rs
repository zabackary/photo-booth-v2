mod border_radius;

use iced::Task;
use iced::border::Radius;
use iced::widget::image::Handle;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum CameraMessage {
    FrameReceived,
}

/// Camera feed.
#[derive(Debug, Clone)]
pub struct CameraFeed {
    current_frame: Arc<Mutex<Option<Handle>>>,
    options: Arc<Mutex<CameraFeedOptions>>,
    manager: crate::backend::manager::BackendManager,
    hashable_manager: HashableManager,
}

/// A wrapper around the manager that makes it hashable
#[derive(Debug, Clone)]
struct HashableManager(crate::backend::manager::BackendManager);

impl std::hash::Hash for HashableManager {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(0); // just a placeholder, we don't actually care about the hash value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraFeedOptions {
    pub radius: Radius,
    pub mirror: bool,
    pub aspect_ratio: Option<f32>,
    pub blur: f32,
}

impl Default for CameraFeedOptions {
    fn default() -> Self {
        Self {
            radius: Radius::from(0),
            mirror: false,
            aspect_ratio: None,
            blur: 0.0,
        }
    }
}

impl CameraFeed {
    /// Create the camera feed.
    ///
    /// The caller is responsible for ensuring `update` is called by the
    /// manager's rx for camera frames.
    pub fn new(
        manager: crate::backend::manager::BackendManager,
        options: CameraFeedOptions,
    ) -> (Self, Task<CameraMessage>) {
        (
            CameraFeed {
                hashable_manager: HashableManager(manager.clone()),
                manager,
                current_frame: Arc::new(Mutex::new(None)),
                options: Arc::new(Mutex::new(options)),
            },
            Task::none(),
        )
    }

    pub fn options(&self) -> CameraFeedOptions {
        *self.options.lock().expect("failed to lock options mutex")
    }

    pub fn update_options(&mut self, options: CameraFeedOptions) {
        *self.options.lock().expect("failed to lock options mutex") = options;
    }

    /// Get the image handle of the current frame.
    pub fn handle(&self) -> Handle {
        if let Some(frame) = self.manager.camera_manager.take_frame_preview() {
            let frame = image_postprocessing(frame, self.options());
            let handle = Handle::from_rgba(frame.width(), frame.height(), frame.into_raw());
            *self.current_frame.lock().expect("failed to lock frame") = Some(handle.clone());
            return handle;
        }
        self.current_frame
            .lock()
            .expect("failed to lock frame")
            .clone()
            .unwrap_or_else(|| Handle::from_rgba(0, 0, vec![]))
    }

    /// Wrap the output of `frame_image` in an `Image` widget.
    pub fn view(&self) -> iced::widget::image::Image<Handle> {
        iced::widget::Image::new(self.handle())
    }

    pub fn update(&mut self, _message: CameraMessage) -> Task<CameraMessage> {
        // `update` implicitly rerenders in iced. this might change later, though.
        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<CameraMessage> {
        iced::Subscription::run_with(self.hashable_manager.clone(), |manager| {
            let rx = manager
                .0
                .take_camera_frame_rx()
                .expect("could not get camera frame stream");
            futures::stream::unfold(rx, |mut rx| async {
                rx.recv().await.map(|item| (item, rx))
            })
        })
        .map(|_| CameraMessage::FrameReceived)
    }
}

fn image_postprocessing(
    frame: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    options: CameraFeedOptions,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    // crop the frame to meet the aspect ratio
    let mut frame = if let Some(aspect_ratio) = options.aspect_ratio {
        let frame_aspect_ratio = frame.width() as f32 / frame.height() as f32;
        let new_width;
        let new_height;
        let left_offset;
        let top_offset;
        if aspect_ratio < frame_aspect_ratio {
            // trim off left and right
            new_height = frame.height();
            new_width = (frame.height() as f32 * aspect_ratio) as u32;
            left_offset = (frame.width() - new_width) / 2;
            top_offset = 0;
        } else if aspect_ratio > frame_aspect_ratio {
            // trim off top and bottom
            new_width = frame.width();
            new_height = (frame.width() as f32 / aspect_ratio) as u32;
            top_offset = (frame.height() - new_height) / 2;
            left_offset = 0;
        } else {
            // perfect aspect ratio!
            new_width = frame.width();
            new_height = frame.height();
            top_offset = 0;
            left_offset = 0;
        }
        image::imageops::crop_imm(&frame, left_offset, top_offset, new_width, new_height).to_image()
    // this might be pricy...
    } else {
        frame
    };

    // mirror the frame
    if options.mirror {
        image::imageops::flip_horizontal_in_place(&mut frame);
    }

    // apply border radius
    border_radius::round(&mut frame, &options.radius);

    // apply blur
    if options.blur > 0.0 {
        frame = image::imageops::thumbnail(
            &frame,
            (frame.width() as f32 / options.blur) as u32,
            (frame.height() as f32 / options.blur) as u32,
        )
        // We could do:
        // frame = image::imageops::blur(&frame, options.blur);
        // but the performance hit is too high for this kind of application
    }
    image::imageops::resize(
        &frame,
        ((frame.width() as f64) / 1.4) as u32,
        ((frame.height() as f64) / 1.4) as u32,
        image::imageops::FilterType::Triangle,
    )
}
