use std::fmt::Debug;

#[cfg(feature = "renderer_simple")]
pub mod simple;

/// A backend for rendering photo strips from individual photos
///
/// Backends may choose to render the photo strip in a different style, or to
/// apply プリクラ-style filters to the photos before rendering the strip. They
/// may also choose to render multiple strips with different styles and and UI
/// will allow the user to select which one they want to print and/or share.
#[async_trait::async_trait]
pub trait RendererBackend: Debug + Send + Sync + 'static {
    /// Render photo strips from the given individual photos
    ///
    /// The returned Vec must have at least one strip.
    async fn render(
        &self,
        photos: Vec<image::RgbaImage>,
    ) -> Result<Vec<image::RgbaImage>, anyhow::Error>;
}
