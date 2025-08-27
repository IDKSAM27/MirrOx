use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};
use crate::adb;

pub async fn start_video_stream(
    tx: Arc<broadcast::Sender<Vec<u8>>>,
    device_id: String,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    println!("Starting optimized video stream for device: {}", device_id);

    let target_frame_time = Duration::from_millis(33);
    let mut frame_count = 0;
    let start_time = Instant::now();

    loop {
        if *shutdown_rx.borrow() {
            println!("Shutting down optimized video stream...");
            break;
        }

        let frame_start = Instant::now();

        match adb::capture_screen_raw_instrumented(&device_id) {
            Ok(raw_data) => {
                frame_count += 1;

                if frame_count == 1 {
                    println!("First raw framebuffer frame size: {}", raw_data.len());
                }

                match extract_rgb_from_raw_frame(&raw_data) {
                    Ok(rgb_frame) => {
                        if tx.send(rgb_frame).is_err() {
                            eprintln!("No active listeners to receive frames.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error processing framebuffer: {}", e);
                        // Fall back to PNG capture frame for safety
                        if let Ok(png_data) = adb::capture_screen(&device_id) {
                            let _ = tx.send(png_data);
                        }
                    }
                }

                if frame_count % 30 == 0 {
                    let elapsed = start_time.elapsed();
                    let fps = frame_count as f32 / elapsed.as_secs_f32();
                    println!("FPS: {:.2}", fps);
                }
            }
            Err(e) => {
                eprintln!("Frame capture error: {}", e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_time {
            tokio::time::sleep(target_frame_time - elapsed).await;
        }
    }
}

/// Extracts RGB24 pixel data from raw Android framebuffer with header.
/// Assumes header occupies first 16 bytes:
/// - 0..4: stride (bytes per row)
/// - 4..8: height (pixels)
/// - 8..12: pixel format (1 for RGBA_8888)
/// Skips header and converts pixel data row-wise removing alpha.
fn extract_rgb_from_raw_frame(raw_data: &[u8]) -> Result<Vec<u8>, String> {
    if raw_data.len() < 16 {
        return Err("Raw framebuffer data too short to contain header".to_string());
    }

    let stride = u32::from_le_bytes([raw_data[0], raw_data[1], raw_data[2], raw_data[3]]) as usize;
    let height = u32::from_le_bytes([raw_data[4], raw_data[5], raw_data[6], raw_data[7]]) as usize;
    let pixel_format = u32::from_le_bytes([raw_data[8], raw_data[9], raw_data[10], raw_data[11]]);

    if pixel_format != 1 {
        return Err(format!("Unsupported pixel format: {} (expecting 1 for RGBA_8888)", pixel_format));
    }

    let pixel_data = &raw_data[16..];

    let width = stride / 4;

    if pixel_data.len() < stride * height {
        return Err(format!(
            "Pixel data length {} less than stride*height {}",
            pixel_data.len(),
            stride * height
        ));
    }

    let mut rgb_data = Vec::with_capacity(width * height * 3);

    for row in 0..height {
        let start = row * stride;
        let end = start + width * 4;
        let row_data = &pixel_data[start..end];
        for pixel in row_data.chunks(4) {
            rgb_data.push(pixel[0]); // R
            rgb_data.push(pixel[1]); // G
            rgb_data.push(pixel[2]); // B
        }
    }

    Ok(rgb_data)
}
