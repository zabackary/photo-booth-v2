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
    manager: crate::backend::manager::camera::CameraManager,
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
        manager: crate::backend::manager::camera::CameraManager,
        options: CameraFeedOptions,
    ) -> (Self, Task<CameraMessage>) {
        (
            CameraFeed {
                manager: manager.camera_manager.clone(),
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
        if let Some(frame) = self.manager.take_frame_preview() {
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
        Task::none()
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
