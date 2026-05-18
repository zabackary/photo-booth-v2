use anyhow::Context as _;
use std::fmt::Display;
use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::Sleep;

const DESTINATION_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// A printer backend that utilizes CUPS to print photos
#[derive(Debug, Clone)]
pub struct CupsPrinterBackend {
    default_printer_name: Option<String>,
    media: Option<String>,
}

impl CupsPrinterBackend {
    pub fn new(default_printer: Option<String>, media: Option<String>) -> Self {
        Self {
            default_printer_name: default_printer,
            media,
        }
    }
}

#[async_trait::async_trait]
impl super::PrinterBackend for CupsPrinterBackend {
    async fn enumerate(&self) -> Result<Vec<Box<dyn super::PrinterBackendHandle>>, anyhow::Error> {
        let destinations = cups_rs::get_all_destinations()?;
        Ok(destinations
            .into_iter()
            .map(|d| {
                Box::new(CupsPrinterHandle {
                    destination: d,
                    media: self.media.clone(),
                }) as Box<dyn super::PrinterBackendHandle>
            })
            .collect())
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Printer>>, anyhow::Error> {
        log::info!("Opening default CUPS printer");
        // open the default printer if specified, otherwise fallback to CUPS's default
        let default_printer = if let Some(ref name) = self.default_printer_name {
            cups_rs::get_destination(name)?
        } else {
            cups_rs::get_default_destination()?
        };
        Ok(Some(Box::new(CupsPrinter {
            destination: default_printer,
            media: self.media.clone(),
        }) as Box<dyn super::Printer>))
    }
}

#[derive(Debug, Clone)]
pub struct CupsPrinterHandle {
    destination: cups_rs::Destination,
    media: Option<String>,
}

impl Display for CupsPrinterHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.destination.full_name())
    }
}

impl super::PrinterBackendHandle for CupsPrinterHandle {
    fn open(&self) -> Result<Box<dyn super::Printer>, anyhow::Error> {
        log::info!("Opening CUPS printer: {}", self);
        Ok(Box::new(CupsPrinter {
            destination: self.destination.clone(),
            media: self.media.clone(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct CupsPrinter {
    destination: cups_rs::Destination,
    media: Option<String>,
}

#[async_trait::async_trait]
impl super::Printer for CupsPrinter {
    async fn print(&mut self, photo: &image::RgbImage, copies: u32) -> Result<(), anyhow::Error> {
        log::info!(
            "Printing CUPS photo of size {}x{}",
            photo.width(),
            photo.height()
        );
        // make up a filename to report to the printer
        // it's not actually a real file, but the CUPS API requires it.
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("cups_print_{}.jpg", timestamp);

        // submit the job to CUPS
        let options = cups_rs::PrintOptions::new()
            .color_mode(cups_rs::ColorMode::Color)
            .copies(copies);
        let job = cups_rs::create_job_with_options(
            &self.destination,
            &filename,
            &if let Some(ref media) = self.media {
                options.media(media)
            } else {
                options
            },
        )
        .context("failed to create job")?;
        let mut encoded = Vec::new();
        let mut encoded_cursor = Cursor::new(&mut encoded);
        photo
            .write_to(&mut encoded_cursor, image::ImageFormat::Jpeg)
            .context("failed to encode strip image")?;
        job.submit_data(&encoded, cups_rs::job::FORMAT_JPEG, &filename)?;

        log::debug!("Print job submitted to CUPS with ID: {}", job.id);

        // wait for the job to complete by polling the printer state
        match wait_for_print_job_completion(&job).await {
            Ok(info) => {
                if info.status == cups_rs::JobStatus::Completed {
                    log::info!("CUPS print job completed with status: {}", info.status);
                } else {
                    log::error!("CUPS print job failed with status: {}", info.status);
                    anyhow::bail!("print job failed with status: {}", info.status);
                }
            }
            Err(e) => {
                log::error!("Failed to wait for CUPS print job completion: {:?}", e);
                anyhow::bail!("failed to wait for print job completion: {:?}", e);
            }
        }

        Ok(())
    }
}

fn wait_for_print_job_completion(
    job: &cups_rs::job::Job,
) -> impl std::future::Future<Output = anyhow::Result<cups_rs::JobInfo>> {
    JobStatusFuture::new(job.id)
}

struct JobStatusFuture {
    job_id: i32,
    sleep: Pin<Box<Sleep>>,
    started: bool,
}

impl JobStatusFuture {
    pub fn new(job_id: i32) -> Self {
        Self {
            job_id,
            sleep: Box::pin(tokio::time::sleep(Duration::from_secs(0))),
            started: false,
        }
    }
}

impl std::future::Future for JobStatusFuture {
    type Output = anyhow::Result<cups_rs::JobInfo>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            self.sleep = Box::pin(tokio::time::sleep(DESTINATION_POLL_INTERVAL));
        }

        if self.sleep.as_mut().poll(cx).is_ready() {
            let info = cups_rs::job::get_job_info(self.job_id);
            match info {
                Ok(info) => match info.status {
                    cups_rs::JobStatus::Pending | cups_rs::JobStatus::Processing => {
                        // job is still pending or processing, poll again later
                        self.sleep
                            .as_mut()
                            .reset(tokio::time::Instant::now() + DESTINATION_POLL_INTERVAL);
                        Poll::Pending
                    }
                    _ => Poll::Ready(Ok(info)),
                },
                Err(e) => {
                    // if we fail, return an error
                    log::error!("Failed to get CUPS job info: {:?}", e);
                    Poll::Ready(Err(anyhow::anyhow!(
                        "failed to get print job info: {:?}",
                        e
                    )))
                }
            }
        } else {
            Poll::Pending
        }
    }
}
