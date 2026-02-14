use anyhow::Context as _;
use serde_json::json;

/// An email backend for sending photos via email using a Google Apps Script webhook
///
/// This email backend is heavily tied to the Google Drive storage backend.
#[derive(Debug)]
pub struct GappsScriptWebhookEmailBackend {
    client: reqwest::Client,
    auth_manager: crate::backend::storage::google_drive::GoogleAuthenticationManager,
    endpoint: reqwest::Url,
}

impl GappsScriptWebhookEmailBackend {
    /// Create a new [`GappsScriptWebhookEmailBackend`] with the given [`GoogleAuthenticationManager`] and endpoint
    ///
    /// The endpoint is the URL of the Google Apps Script webhook that will
    /// handle sending emails. The endpoint should be configured to accept POST
    /// requests with a JSON body containing the folder ID and other metadata,
    /// and should read files from that folder to send the email with the photo
    /// strip and photos attached.
    ///
    /// Endpoint source code is provided in a different repository.
    ///
    /// [`GoogleAuthenticationManager`]: crate::backend::storage::google_drive::GoogleAuthenticationManager
    pub fn new(
        endpoint: reqwest::Url,
        auth_manager: crate::backend::storage::google_drive::GoogleAuthenticationManager,
    ) -> Self {
        let client = reqwest::ClientBuilder::new()
            .build()
            .expect("could not build http client");
        GappsScriptWebhookEmailBackend {
            client,
            auth_manager,
            endpoint,
        }
    }

    pub const OAUTH_SCOPES: &'static [&'static str] =
        crate::backend::storage::google_drive::GoogleDriveStorageBackend::OAUTH_SCOPES;
}

/// Utility function to convert an [`iced::Color`] to a CSS hex string
fn color_hex(color: iced::Color) -> String {
    let r = (color.r * 255.0) as u32;
    let g = (color.g * 255.0) as u32;
    let b = (color.b * 255.0) as u32;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// A partial representation of the email metadata response from the Google Apps
/// Script webhook
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PartialEmailMetadata {
    status: String,
    #[serde(rename = "failedAddresses")]
    failed_addresses: Option<Vec<String>>,
    message: Option<String>,
}

impl PartialEmailMetadata {
    /// Check if the email was sent successfully based on the status field
    fn success(&self) -> bool {
        self.status == "success"
    }

    /// A user-friendly error message
    fn error_message(&self) -> Option<String> {
        if self.status == "error" {
            Some(self.message.clone().unwrap_or_default())
        } else if self.status == "partial" {
            Some(format!(
                "Some email addresses provided could not be reached: {}",
                self.failed_addresses
                    .as_ref()
                    .expect("no address list for failure")
                    .join(", ")
            ))
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl super::EmailBackend for GappsScriptWebhookEmailBackend {
    async fn send_email(self, payload: super::EmailPayload) -> Result<(), anyhow::Error> {
        let crate::backend::storage::StorageHandle::GoogleDriveFolder { folder_id, .. } =
            payload.storage_handle
        else {
            anyhow::bail!("incompatible storage handle for GappsScriptWebhookEmailBackend");
        };
        let emails = payload.emails;
        let palette = payload.palette;

        let token = self
            .auth_manager
            .token()
            .await
            .with_context(|| "could not get authentication token")?;
        let emails_content = json!({
            "emails": emails,
        });
        crate::backend::storage::google_drive::upload_file(
            emails_content.to_string().into_bytes(),
            "metadata.json".to_string(),
            "application/json",
            folder_id.clone(),
            self.client.clone(),
            token.clone(),
        )
        .await?;

        // send a POST request to ENDPOINT_URL with the folderId in JSON in the body
        if !emails.is_empty() {
            let body = json!({
                "folderId": folder_id,
                "backgroundBaseColor": color_hex(palette.background.base.color),
                "backgroundBaseText": color_hex(palette.background.base.text),
                "primaryBaseColor": color_hex(palette.primary.base.color),
                "backgroundWeakColor": color_hex(palette.background.weak.color),
                "backgroundWeakText": color_hex(palette.background.weak.text),
                "eventName": payload.event_name,
                "privacyNote": payload.description,
                "contactEmail": payload.contact_email,
            });

            let res = self
                .client
                .post(self.endpoint.clone())
                .json(&body)
                .send()
                .await
                .with_context(|| "failed to send email request to Google Apps Script webhook")?;
            let email_response: PartialEmailMetadata = res.json().await.with_context(|| {
                "failed to parse email response from Google Apps Script webhook"
            })?;

            if email_response.success() {
                log::debug!("Email sent successfully");

                Ok(())
            } else {
                log::error!(
                    "Error sending email: {}",
                    email_response.error_message().unwrap_or_default()
                );
                anyhow::bail!(
                    "error sending email: {}",
                    email_response.error_message().unwrap_or_default()
                );
            }
        } else {
            log::debug!("No emails provided, skipping email sending");
            Ok(())
        }
    }
}
