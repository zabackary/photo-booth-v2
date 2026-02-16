use std::path::PathBuf;

use image::RgbImage;

#[cfg(any(target_os = "linux", target_os = "macos"))]
const MOCK_PATH: &str = "/dev/null";
#[cfg(target_os = "windows")]
const MOCK_PATH: &str = "NUL";

/// A mock storage backend for testing purposes
#[derive(Debug, Clone, Copy)]
pub struct MockStorageBackend {}

#[async_trait::async_trait]
impl super::StorageBackend for MockStorageBackend {
    async fn upload(
        &self,
        _strip: RgbImage,
        _photos: Vec<RgbImage>,
    ) -> Result<super::StorageHandle, anyhow::Error> {
        log::info!("Uploading to mock storage backend");
        Ok(super::StorageHandle::LocalFilesystem {
            path: PathBuf::from(MOCK_PATH),
        })
    }
}
