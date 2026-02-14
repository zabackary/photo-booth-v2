use std::{io::Cursor, path::Path};

use anyhow::Context;
use image::RgbaImage;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::Part,
    Client,
};
use serde_json::json;
use tokio::try_join;

/// A storage backend using Google Drive and a service account to upload photos
#[derive(Debug, Clone)]
pub struct GoogleDriveStorageBackend {
    http_client: Client,
    auth_manager: GoogleAuthenticationManager,
    folder_id: String,
}

impl GoogleDriveStorageBackend {
    /// Create a new Google Drive storage backend with the given folder ID and token-generating auth manager
    ///
    /// The folder ID is the ID of the folder in Google Drive where photos will
    /// be uploaded. It should be shared with the service account email address
    /// with Editor permissions.
    pub async fn new(
        folder_id: String,
        auth_manager: GoogleAuthenticationManager,
    ) -> Result<Self, anyhow::Error> {
        let http_client = reqwest::ClientBuilder::new()
            .build()
            .with_context(|| "could not build http client")?;

        Ok(GoogleDriveStorageBackend {
            http_client,
            auth_manager,
            folder_id,
        })
    }

    /// The scopes required for Google Drive API access
    pub const OAUTH_SCOPES: &'static [&'static str] = &["https://www.googleapis.com/auth/drive"];
}

#[async_trait::async_trait]
impl super::StorageBackend for GoogleDriveStorageBackend {
    /// Uploads a photo to Google Drive and returns the URL of the strip.
    ///
    /// Creates a new folder within the specified folder in Google Drive,
    /// uploads the strip as strip.png, and uploads the individual photos as
    /// photo_1.png, photo_2.png, etc.
    /// Uploads the emails in a newline-separated text file called emails.txt.
    async fn upload(
        &self,
        strip: RgbaImage,
        photos: Vec<RgbaImage>,
    ) -> Result<super::StorageHandle, anyhow::Error> {
        // sleep(Duration::from_secs(4)).await;
        let token = self
            .auth_manager
            .token()
            .await
            .with_context(|| "could not get authentication token")?;
        let now = chrono::offset::Local::now().to_string();

        // Create a new folder in Google Drive
        log::debug!(
            "Creating folder in Google Drive in folder {}",
            self.folder_id
        );
        let folder_name = now.clone();
        let folder_metadata = json!({
            "name": folder_name,
            "mimeType": "application/vnd.google-apps.folder",
            "parents": [self.folder_id.clone()],
            "description": format!("Uploaded at {} by photo-booth-v2", now.clone())
        });
        let request = self
            .http_client
            .post("https://www.googleapis.com/drive/v3/files")
            .query(&[("supportsAllDrives", "true")])
            .body(folder_metadata.to_string())
            .header(
                "Content-Type",
                HeaderValue::from_static("application/json;charset=UTF-8"),
            )
            .header("Authorization", format!("Bearer {}", token.as_str()));
        let folder: PartialFileMetadata = request
            .send()
            .await
            .with_context(|| "failed to send request to create folder")?
            .error_for_status()
            .with_context(|| "failed to create folder in Google Drive")?
            .json()
            .await
            .with_context(|| "failed to parse response from Google Drive when creating folder")?;
        let folder_id = folder.id;

        log::debug!("Uploaded folder");
        log::debug!("New folder ID: {}", folder_id);

        let (strip_id, _) = try_join!(
            async {
                // Upload the strip
                let mut encoded = Vec::new();
                let mut encoded_cursor = Cursor::new(&mut encoded);
                strip
                    .write_to(&mut encoded_cursor, image::ImageFormat::Png)
                    .with_context(|| "failed to encode strip image")?;
                let token = self
                    .auth_manager
                    .token()
                    .await
                    .with_context(|| "could not get authentication token")?;
                let file = upload_file(
                    encoded,
                    "strip.png".to_string(),
                    "image/png",
                    folder_id.clone(),
                    self.http_client.clone(),
                    token,
                )
                .await?;

                // Make the strip publicly accessible
                let strip_id = file.id;
                let res = self
                    .http_client
                    .post(format!(
                        "https://www.googleapis.com/drive/v3/files/{}/permissions",
                        strip_id
                    ))
                    .body(
                        json!({
                            "type": "anyone",
                            "role": "reader"
                        })
                        .to_string(),
                    )
                    .header(
                        "Content-Type",
                        HeaderValue::from_static("application/json;charset=UTF-8"),
                    )
                    .header("Authorization", format!("Bearer {}", token.as_str()))
                    .send()
                    .await
                    .with_context(|| "failed to send request to set permissions")?;
                log::debug!("Permissions res: {:?}", res.text().await);
                log::debug!("Uploaded strip and permissions");
                Ok(strip_id)
            },
            async {
                // Upload the photos in parallel
                let token = self
                    .auth_manager
                    .token()
                    .await
                    .with_context(|| "could not get authentication token")?;
                let futures = photos.into_iter().enumerate().map(|(i, photo)| {
                    let folder_id = folder_id.clone();
                    let client = self.http_client.clone();
                    async move {
                        let mut encoded = Vec::new();
                        let mut encoded_cursor = Cursor::new(&mut encoded);
                        // Convert the photo to RGB since JPEG doesn't do alpha
                        let photo: image::RgbImage = photo.convert();
                        photo
                            .write_to(&mut encoded_cursor, image::ImageFormat::Jpeg)
                            .with_context(|| "failed to encode photo")?;
                        upload_file(
                            encoded,
                            format!("photo_{}.jpg", i + 1),
                            "image/jpeg",
                            folder_id,
                            client,
                            token,
                        )
                        .await?;
                        Ok(())
                    }
                });

                let mut handles = Vec::with_capacity(futures.len());

                for fut in futures {
                    handles.push(tokio::spawn(fut));
                }

                let mut results = Vec::with_capacity(handles.len());
                for handle in handles {
                    results.push(handle.await.unwrap()?);
                }
                Ok(())
            }
        )?;

        Ok(super::StorageHandle::GoogleDriveFolder {
            folder_id,
            strip_file_id: strip_id,
        })
    }
}

/// An authentication manager
///
/// For now, this is just a thin wrapper around gcp_auth::CustomServiceAccount,
/// but it may evolve to support OAuth 2.0 or other authentication methods in
/// the future.
#[derive(Debug, Clone)]
pub struct GoogleAuthenticationManager {
    service_account: gcp_auth::CustomServiceAccount,
    scopes: Vec<String>,
}

impl GoogleAuthenticationManager {
    /// Create a new GoogleAuthenticationManager from a service account key file and scopes
    pub async fn from_service_account_key(scopes: Vec<String>, file_path: &Path) -> Self {
        let content = tokio::fs::read_to_string(file_path)
            .await
            .expect("could not read service account key file");
        let service_account = gcp_auth::CustomServiceAccount::from_json(&content)
            .with_context(|| "could not create service account from JSON")?;
        Self {
            service_account,
            scopes,
        }
    }

    /// Get an authentication token
    ///
    /// This should not be cached, as tokens are cached and refreshed automatically.
    pub async fn token(&self) -> Result<gcp_auth::Token, anyhow::Error> {
        self.service_account
            .token(&self.scopes)
            .await
            .map_err(|e| anyhow::anyhow!("failed to get token: {}", e))
    }
}

/// Returned file metadata from Google Drive API
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PartialFileMetadata {
    id: String,
}

/// Upload a file to Google Drive
pub(crate) async fn upload_file(
    content: Vec<u8>,
    name: String,
    content_type: &'static str,
    parent_folder_id: String,
    http_client: Client,
    token: gcp_auth::Token,
) -> Result<PartialFileMetadata, anyhow::Error> {
    log::trace!("Uploading file: {}", name);
    log::trace!("Content type: {}", content_type);
    log::trace!("Parent folder ID: {}", parent_folder_id);
    let mut metadata_headers = HeaderMap::with_capacity(1);
    metadata_headers.append(
        "Content-Type",
        HeaderValue::from_static("application/json;charset=UTF-8"),
    );
    let mut content_headers = HeaderMap::with_capacity(1);
    content_headers.append("Content-Type", HeaderValue::from_static(content_type));
    let form = reqwest::multipart::Form::new()
            .part("", Part::text(json!({
            "parents": [parent_folder_id],
            "name": name,
            "description": format!("Uploaded at {} by photo-booth-v2", chrono::offset::Local::now())
            }).to_string()).headers(metadata_headers))
            .part("", Part::bytes(content).headers(content_headers));
    let request = http_client
        .post("https://www.googleapis.com/upload/drive/v3/files")
        .query(&[("uploadType", "multipart")])
        .multipart(form)
        .header(
            "Content-Type",
            HeaderValue::from_static("multipart/related"),
        )
        .header("Authorization", format!("Bearer {}", token.as_str()));
    let file: PartialFileMetadata = request
        .send()
        .await
        .with_context(|| "failed to send request to upload file")?
        .error_for_status()
        .with_context(|| "failed to upload file")?
        .json()
        .await
        .with_context(|| "failed to parse response from Google Drive when uploading file")?;

    log::debug!("Uploaded file");
    log::debug!("File ID: {}", file.id);

    Ok(file)
}
