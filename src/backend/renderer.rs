use std::fmt::Debug;

#[cfg(feature = "renderer_simple")]
pub mod simple;

/// A backend for rendering a photo strip from individual photos
///
/// Backends may choose to render the photo strip in a different style, or to
/// apply プリクラ-style filters to the photos before rendering the strip.
#[async_trait::async_trait]
pub trait RendererBackend: Debug + Send + Sync + 'static {
    /// Render a photo strip from the given individual photos
    async fn render(&self, photos: &[image::RgbaImage]) -> Result<image::RgbaImage, anyhow::Error>;
}
