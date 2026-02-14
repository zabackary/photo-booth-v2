use std::sync::Arc;

use crate::backend::storage::StorageHandle;

/// A storage manager that handles storing photos and providing storage handles for them
#[derive(Debug, Clone)]
pub struct StorageManager {
    backend: Arc<tokio::sync::Mutex<Box<dyn crate::backend::storage::StorageBackend>>>,
}

impl StorageManager {
    /// Create a new [`StorageManager`] with the given storage backend
    pub fn new(storage_backend: Box<dyn crate::backend::storage::StorageBackend>) -> Self {
        StorageManager {
            backend: Arc::new(tokio::sync::Mutex::new(storage_backend)),
        }
    }

    /// Whether the manager is busy storing a photo
    ///
    /// Essentially, whether the mutex is currently locked.
    pub async fn busy(&self) -> bool {
        self.backend.try_lock().is_err()
    }

    /// Wait for the manager to finish storing the current photo, if any.
    ///
    /// Note that [`Self::store`] may have not yet returned a storage handle for
    /// the current photo when this returns, so external state may not have a
    /// storage handle for the current photo until a short time after this returns.
    pub async fn wait(&self) {
        let _ = self.backend.lock().await;
    }

    /// Store and upload a photo, returning a storage handle for it
    pub async fn store(
        &self,
        strip: image::RgbaImage,
        photos: Vec<image::RgbaImage>,
    ) -> Result<StorageHandle, anyhow::Error> {
        let guard = self.backend.lock().await;
        guard.upload(strip, photos).await
    }
}
