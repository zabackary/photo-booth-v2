use std::sync::Arc;

/// An email manager that handles sending photos via email
#[derive(Debug, Clone)]
pub struct EmailManager(Arc<tokio::sync::Mutex<EmailManagerInner>>);

/// Configration for the email manager
#[derive(Debug)]
pub struct EmailManagerConfig {
    /// The color palette to use for the email
    pub palette: iced::theme::palette::Extended,
    /// The name of the event to include in the email
    pub event_name: String,
    /// The description of the event to include in the email
    pub description: String,
    /// The contact email to include in the email
    pub contact_email: String,
}

#[derive(Debug)]
struct EmailManagerInner {
    backend: Box<dyn crate::backend::email::EmailBackend>,
    config: EmailManagerConfig,
}

impl EmailManager {
    /// Create a new [`EmailManager`] with the given email backend
    pub fn new(
        email_backend: Box<dyn crate::backend::email::EmailBackend>,
        config: EmailManagerConfig,
    ) -> Self {
        EmailManager(Arc::new(tokio::sync::Mutex::new(EmailManagerInner {
            backend: email_backend,
            config,
        })))
    }

    /// Whether the manager is busy sending an email
    ///
    /// Essentially, whether the mutex is currently locked.
    pub async fn busy(&self) -> bool {
        self.0.try_lock().is_err()
    }

    /// Wait for the manager to finish sending the current email, if any.
    pub async fn wait(&self) {
        let _ = self.0.lock().await;
    }

    /// Send an email with the given storage handle and recipient emails
    pub async fn send_email(
        &self,
        storage_handle: crate::backend::storage::StorageHandle,
        emails: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        let guard = self.0.lock().await;
        let payload = crate::backend::email::EmailPayload {
            storage_handle,
            emails,
            palette: guard.config.palette,
            event_name: guard.config.event_name.clone(),
            description: guard.config.description.clone(),
            contact_email: guard.config.contact_email.clone(),
        };
        guard.backend.send_email(payload).await?;
        Ok(())
    }
}
