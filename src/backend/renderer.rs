mod filter;

use std::path::PathBuf;

use anyhow::Context as _;
use image::{GenericImage, buffer::ConvertBuffer};
use serde::{Deserialize, Serialize};
use tokio::task;

pub use filter::Filter;

/// A simple renderer which overlays template images on top of the photos
///
/// The template image must be an N-frame strip with transparent areas where the
/// photos should be placed.
#[derive(Debug, Clone)]
pub struct Renderer {
    template: Template,
    filters: Vec<Filter>,
}

impl Renderer {
    /// Creates a new [`Renderer`] with the given template and filters.
    pub fn new(template: Template, filters: Vec<filter::Filter>) -> Self {
        Self { template, filters }
    }

    /// Renders a photo strip from the given photos using the given template.
    async fn render_template(
        overlay: &image::RgbaImage,
        photos: &[(Frame, image::RgbaImage)],
    ) -> Result<image::RgbaImage, anyhow::Error> {
        let mut strip = image::RgbaImage::new(overlay.width(), overlay.height());

        // Fill the strip with black pixels
        for pixel in strip.pixels_mut() {
            *pixel = image::Rgba([0, 0, 0, 255]);
        }

        for (frame, photo) in photos {
            let x = frame.x as u32;
            let y = frame.y as u32;

            // First, crop the photo to the aspect ratio of the frame
            let photo = if frame.width / frame.height > photo.width() as f32 / photo.height() as f32
            {
                // The frame is wider than the photo, so we need to crop the top and bottom of the photo
                let new_height = (photo.width() as f32 * frame.height / frame.width) as u32;
                let y_offset = (photo.height() - new_height) / 2;
                image::imageops::crop_imm(photo, 0, y_offset, photo.width(), new_height).to_image()
            } else {
                // The frame is taller than the photo, so we need to crop the left and right of the photo
                let new_width = (photo.height() as f32 * frame.width / frame.height) as u32;
                let x_offset = (photo.width() - new_width) / 2;
                image::imageops::crop_imm(photo, x_offset, 0, new_width, photo.height()).to_image()
            };
            let photo =
                image::imageops::resize(&photo, 540, 360, image::imageops::FilterType::Lanczos3);
            strip.copy_from(&photo, x, y).with_context(|| {
                format!("failed to copy photo into strip at position ({}, {})", x, y)
            })?;
        }

        image::imageops::overlay(&mut strip, overlay, 0, 0);

        Ok(strip)
    }

    /// Renders a template given the path to the template image and the frames for each photo
    async fn render_template_from_path(
        photos: &[(Frame, image::RgbaImage)],
        template_path: &PathBuf,
    ) -> Result<image::RgbaImage, anyhow::Error> {
        let overlay = tokio::fs::read(template_path)
            .await
            .with_context(|| {
                format!(
                    "failed to read template image from path {:?}",
                    template_path
                )
            })
            .and_then(|data| {
                image::load_from_memory(&data).map_err(|e| {
                    anyhow::anyhow!(
                        "failed to decode template image from path {:?}: {}",
                        template_path,
                        e
                    )
                })
            })?;
        Self::render_template(&overlay.to_rgba8(), photos).await
    }

    pub async fn render(
        &self,
        photos: &[image::RgbaImage],
    ) -> Result<image::RgbaImage, anyhow::Error> {
        let join_handles = photos
            .iter()
            .zip(&self.template.frames)
            .map(|(photo, frame)| {
                let photo = photo.clone();
                let frame = frame.clone();
                let filters = self.filters.clone();

                task::spawn_blocking(move || {
                    let photo = filters
                        .iter()
                        .fold(photo.convert(), |photo, filter| filter.apply(photo));
                    (frame, photo.convert())
                })
            })
            .collect::<Vec<_>>();

        let mut photos = Vec::with_capacity(join_handles.len());
        for photo in join_handles {
            photos.push(photo.await?);
        }

        Self::render_template_from_path(&photos, &self.template.image_path).await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub image_path: PathBuf,
    pub frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    /// The x coordinate of the top-left corner of the frame, in pixels
    pub x: f32,
    /// The y coordinate of the top-left corner of the frame, in pixels
    pub y: f32,
    /// The width of the frame, in pixels
    pub width: f32,
    /// The height of the frame, in pixels
    pub height: f32,
}
