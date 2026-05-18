use std::sync::Arc;

/// A renderer manager that handles rendering photos
#[derive(Debug, Clone)]
pub struct RendererManager {
    backends: Arc<tokio::sync::Mutex<Vec<Box<dyn crate::backend::renderer::RendererBackend>>>>,
}

impl RendererManager {
    /// Create a new [`RendererManager`] with the given renderer backend
    pub fn new(renderer_backends: Vec<Box<dyn crate::backend::renderer::RendererBackend>>) -> Self {
        RendererManager {
            backends: Arc::new(tokio::sync::Mutex::new(renderer_backends)),
        }
    }

    /// Whether the manager is busy rendering a photo
    ///
    /// Essentially, whether the mutex is currently locked.
    pub fn busy(&self) -> bool {
        self.backends.try_lock().is_err()
    }

    /// Wait for the manager to finish rendering the current photo, if any.
    pub async fn wait(&self) {
        let _ = self.backends.lock().await;
    }

    /// Render a photo
    pub async fn render(
        &self,
        photos: Vec<image::RgbaImage>,
    ) -> Result<Vec<image::RgbaImage>, anyhow::Error> {
        let backends = self.backends.lock().await;
        if backends.is_empty() {
            return Ok(photos);
        }
        // Render the photo with all backends in parallel using futures::future::join_all
        futures::future::join_all(backends.iter().map(|backend| backend.render(&photos)))
            .await
            .into_iter()
            .collect()
    }
}
