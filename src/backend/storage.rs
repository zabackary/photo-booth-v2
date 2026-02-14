use std::{fmt::Debug, path::PathBuf};

use image::RgbaImage;

#[cfg(feature = "storage_google_drive")]
pub mod google_drive;
#[cfg(feature = "storage_local_filesystem")]
pub mod local_filesystem;
#[cfg(feature = "mock")]
pub mod mock;

/// A storage backend for uploading photos
#[async_trait::async_trait]
pub trait StorageBackend {
    /// Upload a photo strip and individual photos, returning a handle to the uploaded content
    async fn upload(
        &self,
        strip: RgbaImage,
        photos: Vec<RgbaImage>,
    ) -> Result<StorageBackendHandle, anyhow::Error>;
}

/// A handle to a storage backend
#[derive(Debug)]
#[non_exhaustive]
pub enum StorageBackendHandle {
    /// A reference to a Google Drive folder and the strip it contains, both by
    /// ID
    GoogleDriveFolder {
        folder_id: String,
        strip_file_id: String,
    },
    /// A reference to the local filesystem path where photos are stored
    LocalFilesystem { path: PathBuf },
}

impl StorageBackendHandle {
    /// Get a shareable link to the photo strip, if available
    pub fn strip_link(&self) -> Option<String> {
        match self {
            StorageBackendHandle::GoogleDriveFolder { strip_file_id, .. } => Some(format!(
                "https://drive.google.com/uc?id={}&export=download",
                strip_file_id
            )),
            StorageBackendHandle::LocalFilesystem { path } => None,
        }
    }
}
