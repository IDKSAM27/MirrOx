use std::fs::File;
use std::io::Write;
// use image::io::Reader as ImageReader;
// use image::ImageFormat;
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn parse_screenshot(raw_data: Vec<u8>, output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&raw_data)
        .map_err(|e| format!("Failed to write image data: {}", e))?;

    println!("Screenshot saved as {}", output_path);
    Ok(())
}

#[allow(dead_code)]
pub async fn start_video_stream(tx: Arc<mpsc::UnboundedSender<Vec<u8>>>) {
    loop {
        match crate::adb::capture_screen("screenshot.png") {
            Ok(img_data) => {
                if let Err(e) = tx.send(img_data) {
                    eprintln!("Failed to send image: {}", e);
                }
            }
            Err(e) => eprintln!("Error capturing screen: {}", e),
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}