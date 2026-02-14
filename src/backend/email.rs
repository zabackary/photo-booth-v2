use std::fmt::Debug;

#[cfg(feature = "email_gapps_script_webhook")]
pub mod gapps_script_webhook;
#[cfg(feature = "mock")]
pub mod mock;

/// A email backend for sending photos via email
#[async_trait::async_trait]
pub trait EmailBackend {
    /// Send an email with a link to the photo strip and/or the individual photos attached
    ///
    /// Some backends may not support all storage backends and may return errors
    /// if the storage handle is incompatible with the email backend.
    async fn send_email(self, payload: EmailPayload) -> Result<(), anyhow::Error>;
}

/// Information needed to send an email with a link to the photo strip
#[derive(Debug)]
#[non_exhaustive]
pub struct EmailPayload {
    /// The storage handle for the uploaded photo strip and photos
    ///
    /// Some backends may not support all storage backends and may return an
    /// error if the storage handle is incompatible with the email backend.
    pub storage_handle: super::storage::StorageHandle,

    /// The email addresses to send the photos to
    pub emails: Vec<String>,

    /// The color palette used for this photo strip, for use in email templates
    pub palette: iced::theme::palette::Extended,

    /// The name of this event, for use in email templates
    pub event_name: String,

    /// A description or private message to include in the email, for use in email templates
    pub description: String,

    /// Contact information to include in the email, for use in email templates
    pub contact_email: String,
}
