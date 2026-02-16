use std::path::PathBuf;

use anyhow::Context as _;
use image::RgbImage;

/// A storage backend that saves photos to the local filesystem
#[derive(Debug, Clone)]
pub struct LocalFilesystemStorageBackend {
    base_path: PathBuf,
}

impl LocalFilesystemStorageBackend {
    /// Create a new local filesystem storage backend with the given base path
    ///
    /// This may be mounted to a network drive or shared folder for access from
    /// other devices as file reads/writes are non-blocking and aren't sensitive
    /// to latency.
    pub fn new(base_path: PathBuf) -> Self {
        LocalFilesystemStorageBackend { base_path }
    }
}

#[async_trait::async_trait]
impl super::StorageBackend for LocalFilesystemStorageBackend {
    async fn upload(
        &self,
        strip: RgbImage,
        photos: Vec<RgbImage>,
    ) -> Result<super::StorageHandle, anyhow::Error> {
        // Create a unique subdirectory for this upload using a timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
        let upload_dir = self.base_path.join(format!("upload_{}", timestamp));
        tokio::fs::create_dir_all(&upload_dir)
            .await
            .with_context(|| format!("failed to create upload directory at {:?}", upload_dir))?;

        // Save the strip and photos to the upload directory
        let strip_path = upload_dir.join("strip.png");
        strip
            .save(&strip_path)
            .with_context(|| format!("failed to save photo strip to {:?}", strip_path))?;

        for (i, photo) in photos.into_iter().enumerate() {
            let photo_path = upload_dir.join(format!("photo_{}.jpg", i + 1));
            photo
                .save(&photo_path)
                .with_context(|| format!("failed to save photo {} to {:?}", i + 1, photo_path))?;
        }

        Ok(super::StorageHandle::LocalFilesystem { path: upload_dir })
    }
}
