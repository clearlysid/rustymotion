use anyhow::Error;
use image::{GenericImageView, ImageFormat, Pixel};

// Trim double quotes and unescape string
pub fn unescape_json_string(escaped_json: &str) -> Result<String, serde_json::Error> {
    let trimmed = escaped_json.trim_matches('"');
    let unescaped = serde_json::from_str::<String>(&format!("\"{}\"", trimmed))?;
    Ok(unescaped)
}

// This function is quite slow
pub fn get_bgra_from_png(png_data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let img = image::load_from_memory_with_format(&png_data, ImageFormat::Png)?;
    let (width, height) = img.dimensions();
    let mut bgra_data = vec![0u8; (width * height * 4) as usize];
    for (x, y, pixel) in img.pixels() {
        let rgba = pixel.to_rgba();
        let offset = ((y * width + x) * 4) as usize;
        bgra_data[offset] = rgba[2];
        bgra_data[offset + 1] = rgba[1];
        bgra_data[offset + 2] = rgba[0];
        bgra_data[offset + 3] = rgba[3];
    }

    Ok(bgra_data)
}

pub fn create_display_time(frame_index: u32, fps: u32) -> u64 {
    let frame_duration_nanos = 1_000_000_000 / fps as u64;
    frame_index as u64 * frame_duration_nanos as u64
}
