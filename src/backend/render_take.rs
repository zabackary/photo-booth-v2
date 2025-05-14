use image::GenericImage;

pub fn render_take(photos: Vec<image::RgbaImage>) -> image::RgbaImage {
    let template_overlay = image::load_from_memory(include_bytes!("../../assets/template.png"))
        .expect("Failed to load strip image")
        .to_rgba8();
    let mut strip = image::RgbaImage::new(template_overlay.width(), template_overlay.height());

    // All frames are 2000x1333
    // First frame
    // 134, 134
    // 134, 1600
    // 134, 3066
    // 134, 4532

    assert!(photos.len() == 4, "Expected 4 photos");

    for (i, photo) in photos.iter().enumerate() {
        let x = 30;
        let y = 40 + (i as u32 * 390);
        let resized_photo =
            image::imageops::resize(photo, 540, 360, image::imageops::FilterType::Lanczos3);
        strip.copy_from(&resized_photo, x, y).unwrap();
    }

    image::imageops::overlay(&mut strip, &template_overlay, 0, 0);

    strip
}
