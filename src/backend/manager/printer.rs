use std::sync::Arc;

use crate::config::PrinterConfig;

/// A printing manager that handles printing photos
#[derive(Debug, Clone)]
pub struct PrinterManager {
    backend: Arc<tokio::sync::Mutex<Box<dyn crate::backend::printer::PrinterBackend>>>,
    reconnecting: Arc<std::sync::Mutex<bool>>,
    current_printer: Arc<tokio::sync::Mutex<Box<dyn crate::backend::printer::Printer>>>,
    config: Arc<PrinterManagerConfig>,
}

/// Configration for the printer manager
#[derive(Debug)]
pub struct PrinterManagerConfig {
    /// Whether to automatically duplicate a photo strip with a aspect ratio
    /// less than half of the width of the paper to fill the paper when printing
    pub auto_format: bool,

    /// The horizontal resolution of the image to send to the printer
    ///
    /// For the Canon Selphy CP1500 printer printing Postcard, this should be
    /// set to 300 dpi * 4 inches = 1179 pixels (using mm).
    pub horizontal_resolution: u32,

    /// The vertical resolution of the image to send to the printer
    ///
    /// For the Canon Selphy CP1500 printer printing Postcard, this should be
    /// set to 300 dpi * 6 inches = 1746 pixels (using mm).
    pub vertical_resolution: u32,

    /// How much to scale the photo strip when printing, as a percentage
    /// of the original size. This can be used to fit the photo strip better on
    /// the paper
    ///
    /// The output resolution sent to the printer will be the same, but the
    /// actual print will be scaled by this factor
    pub scale: f32,
}

impl From<PrinterConfig> for PrinterManagerConfig {
    fn from(config: PrinterConfig) -> Self {
        PrinterManagerConfig {
            auto_format: config.auto_format,
            horizontal_resolution: config.horizontal_resolution,
            vertical_resolution: config.vertical_resolution,
            scale: config.scale,
        }
    }
}

impl PrinterManager {
    /// Create a new [`PrinterManager`] with the given printer backend
    pub async fn new(
        printer_backend: Box<dyn crate::backend::printer::PrinterBackend>,
        config: PrinterManagerConfig,
    ) -> Result<Self, anyhow::Error> {
        let initial_printer = match printer_backend.open_default().await {
            Ok(Some(printer)) => printer,
            Ok(None) => {
                anyhow::bail!("Printer backend does not have a default printer");
            }
            Err(e) => {
                log::error!("Failed to open default printer from backend: {:?}", e);
                anyhow::bail!("Failed to open default printer from backend: {:?}", e);
            }
        };
        Ok(Self::with_printer(printer_backend, config, initial_printer))
    }

    fn with_printer(
        printer_backend: Box<dyn crate::backend::printer::PrinterBackend>,
        config: PrinterManagerConfig,
        initial_printer: Box<dyn crate::backend::printer::Printer>,
    ) -> Self {
        PrinterManager {
            backend: Arc::new(tokio::sync::Mutex::new(printer_backend)),
            current_printer: Arc::new(tokio::sync::Mutex::new(initial_printer)),
            reconnecting: Arc::new(std::sync::Mutex::new(false)),
            config: Arc::new(config),
        }
    }

    /// Whether the manager is busy printing a photo
    ///
    /// Essentially, whether the mutex is currently locked.
    pub async fn busy(&self) -> bool {
        self.backend.try_lock().is_err()
    }

    /// Wait for the manager to finish printing the current photo, if any.
    pub async fn wait(&self) {
        let _ = self.backend.lock().await;
    }

    /// Get whether the printer backend is currently trying to reconnect
    pub fn is_reconnecting(&self) -> bool {
        *self.reconnecting.lock().unwrap()
    }

    /// Print a photo
    pub async fn print(&self, photo: image::RgbaImage) -> Result<(), anyhow::Error> {
        let mut printer = self.current_printer.lock().await;
        let processed_photo = self.preprocess_photo(photo);
        loop {
            match printer.print(&processed_photo).await {
                Ok(()) => break,
                Err(e) => {
                    log::error!("Failed to print photo: {:?}", e);
                    *self.reconnecting.lock().unwrap() = true;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    match self.backend.lock().await.open_default().await {
                        Ok(Some(new_printer)) => {
                            *printer = new_printer;
                            *self.reconnecting.lock().unwrap() = false;
                        }
                        Ok(None) => {
                            log::error!("Printer backend does not have a default printer");
                        }
                        Err(e) => {
                            log::error!("Failed to open default printer from backend: {:?}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Preprocess a photo for printing according to the [`PrinterConfig`]
    pub fn preprocess_photo(&self, photo: image::RgbaImage) -> image::RgbaImage {
        let config = self.config.clone();

        let image = if config.auto_format {
            // if the photo's aspect ratio is skinnier than half of the smaller
            // aspect ratio of the canvas, then we'll use a "strip" format where
            // we duplicate the photo twice and put them side by side to fill
            // the canvas in order to print out two copies.
            let photo_aspect_ratio = photo.width() as f32 / photo.height() as f32;
            let canvas_aspect_ratio =
                config.horizontal_resolution as f32 / config.vertical_resolution as f32;
            if photo_aspect_ratio < canvas_aspect_ratio / 2.0 {
                let mut strip = image::RgbaImage::new(photo.width() * 2, photo.height());
                image::imageops::overlay(&mut strip, &photo, 0, 0);
                image::imageops::overlay(&mut strip, &photo, photo.width() as i64, 0);
                strip
            } else if f32::recip(photo_aspect_ratio) < f32::recip(canvas_aspect_ratio) / 2.0 {
                // Similarly, if the photo's aspect ratio is wider than half of the smaller
                // aspect ratio of the canvas, then we'll use a "strip" format where
                // we duplicate the photo twice and put them on top of each other to fill
                // the canvas in order to print out two copies.
                let mut strip = image::RgbaImage::new(photo.width(), photo.height() * 2);
                image::imageops::overlay(&mut strip, &photo, 0, 0);
                image::imageops::overlay(&mut strip, &photo, 0, photo.height() as i64);
                strip
            } else {
                photo
            }
        } else {
            photo
        };

        // Flip the image if necessary and enlarge it to fit without cropping
        let (new_width, new_height) = if image.width() * config.vertical_resolution
            > config.horizontal_resolution * image.height()
        {
            // Image is wider than canvas, fit to height
            let new_width = image.width() * config.vertical_resolution / image.height();
            (new_width, config.vertical_resolution)
        } else {
            // Image is taller than canvas, fit to width
            let new_height = image.height() * config.horizontal_resolution / image.width();
            (config.horizontal_resolution, new_height)
        };
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        // Center the resized image on the canvas
        let mut canvas = image::RgbaImage::from_pixel(
            config.horizontal_resolution,
            config.vertical_resolution,
            image::Rgba([255, 255, 255, 255]),
        );
        let offset_x = (canvas.width() - resized.width()) / 2;
        let offset_y = (canvas.height() - resized.height()) / 2;
        image::imageops::overlay(&mut canvas, &resized, offset_x as i64, offset_y as i64);

        // Final step: resize according to scale factor
        let scaled_width = (config.horizontal_resolution as f32 * config.scale / 100.0) as u32;
        let scaled_height = (config.vertical_resolution as f32 * config.scale / 100.0) as u32;
        let resized = image::imageops::resize(
            &canvas,
            scaled_width,
            scaled_height,
            image::imageops::FilterType::Lanczos3,
        );
        // Center the resized image on the canvas
        let mut final_image = image::RgbaImage::from_pixel(
            config.horizontal_resolution,
            config.vertical_resolution,
            image::Rgba([255, 255, 255, 255]),
        );
        let offset_x = (config.horizontal_resolution - scaled_width) / 2;
        let offset_y = (config.vertical_resolution - scaled_height) / 2;
        image::imageops::overlay(&mut final_image, &resized, offset_x as i64, offset_y as i64);
        final_image
    }
}
