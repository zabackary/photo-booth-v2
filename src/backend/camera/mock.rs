use std::fmt::Display;

// 4:3 aspect ratio size
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

/// A camera backend returning placeholder images for testing purposes
#[derive(Debug, Clone, Copy)]
pub struct MockCameraBackend {}

#[async_trait::async_trait]
impl super::CameraBackend for MockCameraBackend {
    async fn enumerate(&self) -> Result<Vec<Box<dyn super::CameraBackendHandle>>, anyhow::Error> {
        Ok(vec![
            Box::new(MockCameraHandle { integrated: true }) as Box<dyn super::CameraBackendHandle>,
            Box::new(MockCameraHandle { integrated: false }) as Box<dyn super::CameraBackendHandle>,
        ])
    }

    async fn open_default(&self) -> Result<Option<Box<dyn super::Camera>>, anyhow::Error> {
        log::info!("Opening default mock camera");
        Ok(Some(Box::new(MockCamera {}) as Box<dyn super::Camera>))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockCameraHandle {
    integrated: bool,
}

impl Display for MockCameraHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.integrated {
            write!(f, "Mock Integrated Camera")
        } else {
            write!(f, "Mock External Camera")
        }
    }
}

impl super::CameraBackendHandle for MockCameraHandle {
    fn open(&self) -> Result<Box<dyn super::Camera>, anyhow::Error> {
        log::info!("Opening mock camera: {}", self);
        Ok(Box::new(MockCamera {}))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockCamera {}

const DIGITS: [[bool; 7]; 10] = [
    [true, true, true, true, true, true, false],     // 0
    [false, true, true, false, false, false, false], // 1
    [true, true, false, true, true, false, true],    // 2
    [true, true, true, true, false, false, true],    // 3
    [false, true, true, false, false, true, true],   // 4
    [true, false, true, true, false, true, true],    // 5
    [true, false, true, true, true, true, true],     // 6
    [true, true, true, false, false, false, false],  // 7
    [true, true, true, true, true, true, true],      // 8
    [true, true, true, true, false, true, true],     // 9
];

impl super::Camera for MockCamera {
    fn frame_still(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        // render a placeholder image with the timestamp as a primitive 7-segment
        // display to avoid pulling in any deps
        let mut img = image::RgbaImage::new(WIDTH, HEIGHT);
        img.fill(255);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let timestamp_str = format!("{}", timestamp);
        let digit_width = 6;
        let digit_height = 8;
        for (i, c) in timestamp_str
            .chars()
            .enumerate()
            .skip(timestamp_str.len().saturating_sub(4))
        {
            let x = i as u32 * (digit_width + 2);
            let y = HEIGHT / 4;
            if let Some(digit) = c.to_digit(10) {
                let segments = DIGITS[digit as usize];
                // top
                if segments[0] {
                    for dx in 0..digit_width {
                        img.put_pixel(x + dx, y, image::Rgba([255, 0, 0, 255]));
                    }
                }
                // top-right
                if segments[1] {
                    for dy in 0..digit_height {
                        img.put_pixel(x + digit_width - 1, y + dy, image::Rgba([255, 0, 0, 255]));
                    }
                }
                // bottom-right
                if segments[2] {
                    for dy in 0..digit_height {
                        img.put_pixel(
                            x + digit_width - 1,
                            y + digit_height + dy,
                            image::Rgba([255, 0, 0, 255]),
                        );
                    }
                }
                // bottom
                if segments[3] {
                    for dx in 0..digit_width {
                        img.put_pixel(
                            x + dx,
                            y + digit_height * 2 - 1,
                            image::Rgba([255, 0, 0, 255]),
                        );
                    }
                }
                // bottom-left
                if segments[4] {
                    for dy in 0..digit_height {
                        img.put_pixel(x, y + digit_height + dy, image::Rgba([255, 0, 0, 255]));
                    }
                }
                // top-left
                if segments[5] {
                    for dy in 0..digit_height {
                        img.put_pixel(x, y + dy, image::Rgba([255, 0, 0, 255]));
                    }
                }
                // middle
                if segments[6] {
                    for dx in 0..digit_width {
                        img.put_pixel(x + dx, y + digit_height - 1, image::Rgba([255, 0, 0, 255]));
                    }
                }
            }
        }
        Ok(img)
    }

    fn frame_preview(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        self.frame_still()
    }
}
