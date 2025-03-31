use std::fs::File;
use std::io::Write;
// use image::io::Reader as ImageReader;
// use image::ImageFormat;
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn parse_screenshot(raw_data: Vec<u8>, output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&raw_data)
        .map_err(|e| format!("Failed to write image data: {}", e))?;

    println!("Screenshot saved as {}", output_path);
    Ok(())
}

pub async fn start_video_stream(tx: Arc<broadcast::Sender<Vec<u8>>>, device_id: String) {
    loop {
        match crate::adb::capture_screen(&device_id) { // Change to your actual device ID logic
            Ok(raw_data) => {
                if tx.send(raw_data).is_err() {
                    eprintln!("No active WebSocket listeners.");
                }
            }
            Err(e) => eprintln!("Failed to capture frame: {}", e),
        }

        tokio::time::sleep(std::time::Duration::from_millis(33)).await; // 30 FPS
    }
}