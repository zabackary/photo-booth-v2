/// A mock email backend for testing purposes
#[derive(Debug, Clone, Copy)]
pub struct MockEmailBackend {}

#[async_trait::async_trait]
impl super::EmailBackend for MockEmailBackend {
    async fn send_email(&self, payload: super::EmailPayload) -> Result<(), anyhow::Error> {
        log::info!("Sending email with mock email backend");
        log::info!("Storage handle: {:?}", payload.storage_handle);
        log::info!("Emails: {:?}", payload.emails);
        log::info!("Palette: {:?}", payload.palette);
        Ok(())
    }
}
