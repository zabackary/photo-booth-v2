use std::sync::Arc;

use image::buffer::ConvertBuffer;

use crate::backend::renderer::Renderer;

/// A renderer manager that handles rendering photos
#[derive(Debug, Clone)]
pub struct RendererManager {
    renderers: Arc<tokio::sync::Mutex<Vec<Renderer>>>,
}

impl RendererManager {
    /// Create a new [`RendererManager`] with the given renderer backend
    pub fn new(renderers: Vec<Renderer>) -> Self {
        RendererManager {
            renderers: Arc::new(tokio::sync::Mutex::new(renderers)),
        }
    }

    /// Whether the manager is busy rendering a photo
    ///
    /// Essentially, whether the mutex is currently locked.
    pub fn busy(&self) -> bool {
        self.renderers.try_lock().is_err()
    }

    /// Wait for the manager to finish rendering the current photo, if any.
    pub async fn wait(&self) {
        let _ = self.renderers.lock().await;
    }

    /// Render a photo
    pub async fn render(
        &self,
        photos: Vec<image::RgbaImage>,
    ) -> Result<Vec<image::RgbaImage>, anyhow::Error> {
        let renderers = self.renderers.lock().await;

        // Render the photo in parallel using futures::future::join_all
        futures::future::join_all(renderers.iter().map(|renderer| renderer.render(&photos)))
            .await
            .into_iter()
            .collect()
    }
}
