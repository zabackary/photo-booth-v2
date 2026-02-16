use std::{fmt::Debug, path::PathBuf};

use image::RgbImage;

#[cfg(feature = "storage_google_drive")]
pub mod google_drive;
#[cfg(feature = "storage_local_filesystem")]
pub mod local_filesystem;
#[cfg(feature = "mock")]
pub mod mock;

/// A storage backend for uploading photos
#[async_trait::async_trait]
pub trait StorageBackend: Debug + Send + Sync + 'static {
    /// Upload a photo strip and individual photos, returning a handle to the uploaded content
    async fn upload(
        &self,
        strip: RgbImage,
        photos: Vec<RgbImage>,
    ) -> Result<StorageHandle, anyhow::Error>;
}

/// A handle to a stored photo strip and its associated photos
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum StorageHandle {
    /// A reference to a Google Drive folder and the strip it contains, both by
    /// ID
    GoogleDriveFolder {
        folder_id: String,
        strip_file_id: String,
    },
    /// A reference to the local filesystem path where photos are stored
    LocalFilesystem { path: PathBuf },
}

impl StorageHandle {
    /// Get a shareable link to the photo strip, if available
    pub fn strip_link(&self) -> Option<String> {
        match self {
            StorageHandle::GoogleDriveFolder { strip_file_id, .. } => Some(format!(
                "https://drive.google.com/uc?id={}&export=download",
                strip_file_id
            )),
            StorageHandle::LocalFilesystem { .. } => None,
        }
    }
}
