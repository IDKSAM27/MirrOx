use std::fs::File;
use std::io::Write;
// use image::io::Reader as ImageReader;
// use image::ImageFormat;

pub fn parse_screenshot(raw_data: Vec<u8>, output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&raw_data)
        .map_err(|e| format!("Failed to write image data: {}", e))?;

    println!("Screenshot saved as {}", output_path);
    Ok(())
}