use std::fmt::{Debug, Display};

#[cfg(feature = "mock")]
pub mod mock;

/// A printer backend
#[async_trait::async_trait]
pub trait PrinterBackend: Debug + Send + Sync + 'static {
    /// Initialize this backend
    async fn initialize(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Enumerate available printers attached to this backend
    async fn enumerate(&self) -> Result<Vec<Box<dyn PrinterBackendHandle>>, anyhow::Error>;

    /// Opens the default printer provided by this backend, if any
    ///
    /// It is up to the backend to determine what the "default" printer is
    async fn open_default(&self) -> Result<Option<Box<dyn Printer>>, anyhow::Error> {
        Ok(None)
    }
}

/// A handle to open a printer
///
/// Its `Display` implementation should provide a user-friendly name for the printer.
pub trait PrinterBackendHandle: Debug + Display {
    fn open(&self) -> Result<Box<dyn Printer>, anyhow::Error>;
}

/// A printer that can print photos
#[async_trait::async_trait]
pub trait Printer: Debug + Send + Sync {
    /// Print a photo
    async fn print(&mut self, photo: &image::RgbImage) -> Result<(), anyhow::Error>;
}
