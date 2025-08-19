use std::fs::File;
use std::io::Write;
// use image::io::Reader as ImageReader;
// use image::ImageFormat;
use std::sync::Arc;
use tokio::sync::{broadcast, watch};

pub fn parse_screenshot(raw_data: Vec<u8>, output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&raw_data)
        .map_err(|e| format!("Failed to write image data: {}", e))?;

    println!("Screenshot saved as {}", output_path);
    Ok(())
}

pub async fn start_video_stream(
    tx: Arc<broadcast::Sender<Vec<u8>>>,
    device_id: String,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    // TODO: remove the No active WebSockte listeners loop
    // TODO: also terminate the application if the window is closed
    loop {
        // Break if shutdown signal received
        if *shutdown_rx.borrow() {
            println!("Shutting down video stream...");
            break;
        }

        match crate::adb::capture_screen(&device_id) {
            Ok(raw_data) => {
                if tx.send(raw_data).is_err() {
                    eprintln!("No active WebSocket listeners.");
                }
            }
            // Err(e) => eprintln!("Failed to capture frame: {}", e),
            Err(_) => eprintln!(""),
        }

        tokio::time::sleep(std::time::Duration::from_millis(33)).await;
    }
}
