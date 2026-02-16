use std::fmt::Display;

/// A printer backend
#[derive(Debug, Clone, Copy)]
pub struct MockPrinterBackend {}

#[async_trait::async_trait]
impl super::PrinterBackend for MockPrinterBackend {
    async fn enumerate(&self) -> Result<Vec<Box<dyn super::PrinterBackendHandle>>, anyhow::Error> {
        Ok(vec![
            Box::new(MockPrinterHandle {}) as Box<dyn super::PrinterBackendHandle>
        ])
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Printer>>, anyhow::Error> {
        log::info!("Opening default mock printer");
        Ok(Some(Box::new(MockPrinter {}) as Box<dyn super::Printer>))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockPrinterHandle {}

impl Display for MockPrinterHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mock Printer")
    }
}

impl super::PrinterBackendHandle for MockPrinterHandle {
    fn open(&self) -> Result<Box<dyn super::Printer>, anyhow::Error> {
        log::info!("Opening mock printer: {}", self);
        Ok(Box::new(MockPrinter {}))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockPrinter {}

#[async_trait::async_trait]
impl super::Printer for MockPrinter {
    async fn print(&mut self, photo: &image::RgbImage) -> Result<(), anyhow::Error> {
        log::info!(
            "Printing mock photo of size {}x{}",
            photo.width(),
            photo.height()
        );
        // FIXME: remove
        tokio::time::sleep(std::time::Duration::from_secs(25)).await;
        Ok(())
    }
}
