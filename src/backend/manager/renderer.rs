use std::sync::Arc;

/// A renderer manager that handles rendering photos
#[derive(Debug, Clone)]
pub struct RendererManager {
    backend: Arc<tokio::sync::Mutex<Box<dyn crate::backend::renderer::RendererBackend>>>,
}

impl RendererManager {
    /// Create a new [`RendererManager`] with the given renderer backend
    pub fn new(renderer_backend: Box<dyn crate::backend::renderer::RendererBackend>) -> Self {
        RendererManager {
            backend: Arc::new(tokio::sync::Mutex::new(renderer_backend)),
        }
    }

    /// Whether the manager is busy rendering a photo
    ///
    /// Essentially, whether the mutex is currently locked.
    pub async fn busy(&self) -> bool {
        self.backend.try_lock().is_err()
    }

    /// Wait for the manager to finish rendering the current photo, if any.
    pub async fn wait(&self) {
        let _ = self.backend.lock().await;
    }

    /// Render a photo
    pub async fn render(
        &self,
        photos: Vec<image::RgbaImage>,
    ) -> Result<Vec<image::RgbaImage>, anyhow::Error> {
        let guard = self.backend.lock().await;
        guard.render(photos).await
    }
}
