use image::buffer::ConvertBuffer as _;

/// A filter to apply to a photo when rendering
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "filter", rename_all = "lowercase")]
pub enum Filter {
    /// A filter that applies a hue rotation to the photo
    HueRotate { degrees: i32 },
    /// A filter that applies a grayscale effect to the photo, with the given intensity
    Grayscale {
        /// The intensity of the grayscale effect, from 0.0 to 1.0
        intensity: f32,
    },
    /// A filter that applies a brightness adjustment to the photo
    Brightness {
        /// The amount to adjust the brightness, where 0.0 is no change, negative values are darker, and positive values are brighter
        amount: f32,
    },
    /// A filter that applies a contrast adjustment to the photo
    Contrast {
        /// The amount to adjust the contrast, where 0.0 is no change, negative values have less contrast, and positive values have more contrast
        amount: f32,
    },
    /// A filter that applies a bilateral blur to the photo to soften skin
    #[cfg(feature = "filter_skin_softening")]
    #[serde(rename_all = "camelCase")]
    SkinSoftening {
        /// The radius of the filter (optimally between 1 to 3)
        radius: u8,
        spatial_sigma: f32,
        color_sigma: f32,
    },
}

impl Filter {
    /// Applies the filter to the given photo and returns the result
    pub fn apply(&self, photo: image::RgbImage) -> image::RgbImage {
        match self {
            Filter::HueRotate { degrees } => image::imageops::huerotate(&photo, *degrees),
            Filter::Grayscale { intensity } => {
                let grayscale_photo = image::imageops::grayscale(&photo);
                // Blend by adding an alpha channel to the grayscale photo
                let mut grayscale_photo: image::RgbaImage = grayscale_photo.convert();
                grayscale_photo
                    .pixels_mut()
                    .for_each(|pixel| pixel[3] = (intensity * 255.0) as u8);
                let mut photo: image::RgbaImage = photo.convert();
                image::imageops::overlay(&mut photo, &grayscale_photo, 0, 0);
                photo.convert()
            }
            Filter::Brightness { amount } => {
                image::imageops::brighten(&photo, (*amount * 255.0) as i32)
            }
            Filter::Contrast { amount } => image::imageops::contrast(&photo, *amount),
            #[cfg(feature = "filter_skin_softening")]
            Filter::SkinSoftening {
                radius,
                spatial_sigma,
                color_sigma,
            } => {
                use imageproc::filter::{self, bilateral::GaussianEuclideanColorDistance};
                filter::bilateral_filter(
                    &photo,
                    *radius,
                    *spatial_sigma,
                    GaussianEuclideanColorDistance::new(*color_sigma),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hue_rotate() {
        let mut photo = image::RgbImage::new(1, 1);
        photo.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        let filter = Filter::HueRotate { degrees: 90 };
        let result = filter.apply(photo);
        assert_eq!(result.get_pixel(0, 0), &image::Rgb([0, 90, 0]));
    }

    #[test]
    fn test_grayscale() {
        let mut photo = image::RgbImage::new(1, 1);
        photo.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        let filter = Filter::Grayscale { intensity: 1.0 };
        let result = filter.apply(photo);
        assert_eq!(result.get_pixel(0, 0), &image::Rgb([54, 54, 54]));
    }

    #[test]
    fn test_brightness() {
        let mut photo = image::RgbImage::new(1, 1);
        photo.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        let filter = Filter::Brightness { amount: -1.0 };
        let result = filter.apply(photo);
        assert_eq!(result.get_pixel(0, 0), &image::Rgb([0, 0, 0]));
    }
}
