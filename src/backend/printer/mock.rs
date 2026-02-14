use std::fmt::Display;

/// A printer backend
#[derive(Debug, Clone, Copy)]
pub struct MockPrinterBackend {}

#[async_trait::async_trait]
impl super::PrinterBackend for MockPrinterBackend {
    type Error = anyhow::Error;

    async fn enumerate(&self) -> Result<Vec<dyn super::PrinterBackendHandle>, Self::Error> {
        let mut vec = Vec::<dyn super::PrinterBackendHandle>::with_capacity(1);

        vec.push(MockPrinterHandle {});

        Ok(vec)
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Printer>>, Self::Error> {
        log::info!("Opening default mock printer");
        Ok(Some(Box::new(MockPrinter {}) as Box<dyn super::Printer>))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockPrinterHandle {}

impl Display for MockPrinterHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.integrated {
            write!(f, "Mock Integrated Printer")
        } else {
            write!(f, "Mock External Printer")
        }
    }
}

impl super::PrinterBackendHandle for MockPrinterHandle {
    fn open(&self) -> Result<Box<dyn super::Printer>, anyhow::Error> {
        log::info!("Opening mock printer: {}", self);
        Ok(Box::new(MockPrinter {}))
    }
}

pub struct MockPrinter {}

#[async_trait::async_trait]
impl super::Printer for MockPrinter {
    async fn print(&mut self, photo: &image::RgbaImage) -> Result<(), anyhow::Error> {
        log::info!(
            "Printing mock photo of size {}x{}",
            photo.width(),
            photo.height()
        );
        Ok(())
    }
}
