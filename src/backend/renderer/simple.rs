use std::path::PathBuf;

use anyhow::Context as _;
use image::GenericImage;
use serde::{Deserialize, Serialize};

/// A simple renderer which overlays template images on top of the photos
///
/// The template image must be an N-frame strip with transparent areas where the
/// photos should be placed.
#[derive(Debug, Clone)]
pub struct SimpleRendererBackend {
    templates: Vec<Template>,
}

impl SimpleRendererBackend {
    /// Creates a new [`SimpleRendererBackend`] with the given templates.
    pub fn new(templates: Vec<Template>) -> Self {
        Self { templates }
    }

    /// Renders a photo strip from the given photos using the given template.
    async fn render_template(
        &self,
        overlay: &image::RgbaImage,
        photos: &[(Frame, &image::RgbaImage)],
    ) -> Result<image::RgbaImage, anyhow::Error> {
        let mut strip = image::RgbaImage::new(overlay.width(), overlay.height());

        for &(ref frame, photo) in photos {
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
}

#[async_trait::async_trait]
impl super::RendererBackend for SimpleRendererBackend {
    async fn render(
        &self,
        photos: Vec<image::RgbaImage>,
    ) -> Result<Vec<image::RgbaImage>, anyhow::Error> {
        futures::future::join_all(self.templates.iter().map(|template| {
            let photos = photos
                .iter()
                .zip(&template.frames)
                .map(|(photo, frame)| (frame.clone(), photo))
                .collect::<Vec<_>>();
            async move {
                let overlay = tokio::fs::read(&template.image_path)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to read template image from path {:?}",
                            template.image_path
                        )
                    })
                    .and_then(|data| {
                        image::load_from_memory(&data).map_err(|e| {
                            anyhow::anyhow!(
                                "failed to decode template image from path {:?}: {}",
                                template.image_path,
                                e
                            )
                        })
                    })?;
                self.render_template(&overlay.to_rgba8(), &photos).await
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
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
