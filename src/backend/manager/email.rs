use std::sync::Arc;

/// An email manager that handles sending photos via email
#[derive(Debug, Clone)]
pub struct EmailManager(Arc<tokio::sync::Mutex<EmailManagerInner>>);

#[derive(Debug)]
struct EmailManagerInner {
    backend: Box<dyn crate::backend::email::EmailBackend>,
    palette: iced::theme::palette::Extended,
    event_name: String,
    description: String,
    contact_email: String,
}

impl EmailManager {
    /// Create a new [`EmailManager`] with the given email backend
    pub fn new(
        email_backend: Box<dyn crate::backend::email::EmailBackend>,
        palette: iced::theme::palette::Extended,
        event_name: String,
        description: String,
        contact_email: String,
    ) -> Self {
        EmailManager(Arc::new(tokio::sync::Mutex::new(EmailManagerInner {
            backend: email_backend,
            palette,
            event_name,
            description,
            contact_email,
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
            palette: guard.palette,
            event_name: guard.event_name.clone(),
            description: guard.description.clone(),
            contact_email: guard.contact_email.clone(),
        };
        guard.backend.send_email(payload).await?;
        Ok(())
    }
}
