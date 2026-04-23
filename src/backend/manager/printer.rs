use std::sync::Arc;

use image::buffer::ConvertBuffer as _;

use crate::config::PrinterConfig;

/// A printing manager that handles printing photos
#[derive(Debug, Clone)]
pub struct PrinterManager {
    backend: Arc<tokio::sync::Mutex<Box<dyn crate::backend::printer::PrinterBackend>>>,
    reconnecting: Arc<std::sync::Mutex<bool>>,
    current_printer: Arc<tokio::sync::Mutex<Box<dyn crate::backend::printer::Printer>>>,
    config: Arc<PrinterManagerConfig>,
}

/// Configuration for the printer manager
#[derive(Debug, Clone)]
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

    /// A file to log print quantities to for billing or analytics purposes.
    pub print_log_file: Option<std::path::PathBuf>,
}

impl From<&PrinterConfig> for PrinterManagerConfig {
    fn from(config: &PrinterConfig) -> Self {
        PrinterManagerConfig {
            auto_format: config.auto_format,
            horizontal_resolution: config.horizontal_resolution,
            vertical_resolution: config.vertical_resolution,
            scale: config.scale,
            print_log_file: config.print_log_file.clone(),
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
    pub fn busy(&self) -> bool {
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

    pub async fn print(&self, photo: image::RgbaImage, quantity: u32) -> Result<(), anyhow::Error> {
        // Log the print job to the print log file, if configured
        if let Some(log_file) = &self.config.print_log_file
            && let Err(e) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .and_then(|mut file| {
                    use std::io::Write as _;
                    writeln!(file, "{},{}", chrono::Utc::now().to_rfc3339(), quantity)
                })
        {
            log::error!("Failed to log print job to file {:?}: {:?}", log_file, e);
        }
        if quantity == 0 {
            log::trace!("Print quantity is 0, skipping printing");
            return Ok(());
        }
        let photo: image::RgbImage = photo.convert();
        let (processed_photo, copies_per_output) = preprocess_photo(&self.config, photo);
        let true_quantity = (quantity as f32 / copies_per_output as f32).ceil() as u32;
        if true_quantity != quantity {
            log::info!(
                "Auto-formatting resulted in {} copies per output, so printing {} copies instead of requested {}",
                copies_per_output,
                true_quantity,
                quantity
            );
        }
        let mut backend = self.backend.lock().await;
        for i in 0..true_quantity {
            log::debug!("Printing copy {}/{}", i + 1, true_quantity);
            self.print_single(&mut backend, &processed_photo).await?;
        }
        std::mem::drop(backend); // hold on to it until here to print all copies before allowing any other print jobs to start
        Ok(())
    }

    /// Print a single photo
    async fn print_single(
        &self,
        backend: &mut Box<dyn crate::backend::printer::PrinterBackend>,
        processed_photo: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    ) -> Result<(), anyhow::Error> {
        let mut printer = self.current_printer.lock().await;
        loop {
            match printer.print(processed_photo).await {
                Ok(()) => break,
                Err(e) => {
                    log::error!("Failed to print photo: {:?}", e);
                    *self.reconnecting.lock().unwrap() = true;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    match backend.open_default().await {
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
}

/// Preprocess a photo for printing according to the [`PrinterManagerConfig`]
pub fn preprocess_photo(
    config: &PrinterManagerConfig,
    photo: image::RgbImage,
) -> (image::RgbImage, u32) {
    let mut copies_per_output = 1u32;

    // Step 1: Optionally convert into a "strip" by duplicating the image
    // horizontally or vertically when the photo is very skinny/wide.
    let mut work = if config.auto_format {
        let photo_ar = photo.width() as f32 / photo.height() as f32;
        let canvas_ar = config.horizontal_resolution as f32 / config.vertical_resolution as f32;
        if photo_ar < canvas_ar / 2.0 {
            // very skinny: make a vertical strip duplicated side-by-side
            let mut strip = image::RgbImage::new(photo.width() * 2, photo.height());
            image::imageops::overlay(&mut strip, &photo, 0, 0);
            image::imageops::overlay(&mut strip, &photo, photo.width() as i64, 0);
            copies_per_output = 2;
            strip
        } else if (1.0 / photo_ar) < (1.0 / canvas_ar) / 2.0 {
            // very wide: duplicate top/bottom
            let mut strip = image::RgbImage::new(photo.width(), photo.height() * 2);
            image::imageops::overlay(&mut strip, &photo, 0, 0);
            image::imageops::overlay(&mut strip, &photo, 0, photo.height() as i64);
            copies_per_output = 2;
            strip
        } else {
            photo
        }
    } else {
        photo
    };

    // Canvas target size
    let canvas_w = config.horizontal_resolution;
    let canvas_h = config.vertical_resolution;

    // Step 2: Choose orientation (rotated or not) that gives the largest
    // scale-to-fit factor so the image fills the canvas as much as possible.
    let (w, h) = (work.width() as f32, work.height() as f32);
    let scale_no_rot = (canvas_w as f32 / w).min(canvas_h as f32 / h);
    let scale_rot = (canvas_w as f32 / h).min(canvas_h as f32 / w);
    let rotate = scale_rot > scale_no_rot;
    if rotate {
        work = image::imageops::rotate90(&work);
    }

    // Step 3: Resize the work image to be as large as possible within the
    // canvas while preserving aspect ratio.
    let img_w = work.width() as f32;
    let img_h = work.height() as f32;
    let scale = (canvas_w as f32 / img_w).min(canvas_h as f32 / img_h);
    let target_w = (img_w * scale).round().max(1.0) as u32;
    let target_h = (img_h * scale).round().max(1.0) as u32;
    let resized = image::imageops::resize(
        &work,
        target_w,
        target_h,
        image::imageops::FilterType::Lanczos3,
    );

    // Center the resized image on a white canvas of the configured size
    let mut canvas = image::RgbImage::from_pixel(canvas_w, canvas_h, image::Rgb([255, 255, 255]));
    let offset_x = (canvas_w as i64 - resized.width() as i64) / 2;
    let offset_y = (canvas_h as i64 - resized.height() as i64) / 2;
    image::imageops::overlay(&mut canvas, &resized, offset_x, offset_y);

    // Step 4: Apply final scale factor (percent) and center that result on
    // a canvas of the configured size. This preserves the final output
    // resolution while simulating a physical scale change.
    let scaled_w = ((canvas_w as f32) * config.scale / 100.0).round().max(1.0) as u32;
    let scaled_h = ((canvas_h as f32) * config.scale / 100.0).round().max(1.0) as u32;
    let scaled = image::imageops::resize(
        &canvas,
        scaled_w,
        scaled_h,
        image::imageops::FilterType::Lanczos3,
    );
    let mut final_image =
        image::RgbImage::from_pixel(canvas_w, canvas_h, image::Rgb([255, 255, 255]));
    let off_x = (canvas_w as i64 - scaled.width() as i64) / 2;
    let off_y = (canvas_h as i64 - scaled.height() as i64) / 2;
    image::imageops::overlay(&mut final_image, &scaled, off_x, off_y);

    (final_image, copies_per_output)
}
