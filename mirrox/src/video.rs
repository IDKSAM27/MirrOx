use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};
use std::process::Command;
use crate::adb;

pub async fn start_video_stream(
    tx: Arc<broadcast::Sender<Vec<u8>>>,
    device_id: String,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    println!("Starting optimized video stream for device: {}", device_id);

    // Query device screen resolution
    let (screen_width, screen_height) = match get_screen_resolution(&device_id) {
        Ok((w, h)) => (w, h),
        Err(e) => {
            eprintln!("Failed to get screen resolution, using defaults 1080x2400: {}", e);
            (1080, 2400)
        }
    };
    println!("Screen resolution: {}x{}", screen_width, screen_height);

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
                
                match extract_rgb_from_raw_frame(&raw_data, screen_width, screen_height) {
                    Ok(rgb_frame) => {
                        if tx.send(rgb_frame).is_err() {
                            eprintln!("No active listeners to receive frames.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error processing framebuffer: {}", e);
                        // Fallback to PNG capture frame
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

/// Queries device screen resolution using `adb shell wm size`
fn get_screen_resolution(device_id: &str) -> Result<(usize, usize), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "wm", "size"])
        .output()
        .map_err(|e| format!("Failed to run adb wm size: {}", e))?;

    if !output.status.success() {
        return Err("adb wm size command failed".into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(pos) = line.find("Physical size:") {
            let size_str = &line[pos + "Physical size:".len()..].trim();
            let parts: Vec<&str> = size_str.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse(), parts[1].parse()) {
                    return Ok((w, h));
                }
            }
        }
    }
    Err("Failed to parse screen size from adb output".into())
}

/// Extracts RGB24 pixels from raw framebuffer with header and stride handling.
/// Header: first 16 bytes (stride, height, pixel_format, reserved).
fn extract_rgb_from_raw_frame(raw_data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    if raw_data.len() < 16 {
        return Err("Raw framebuffer data too short to contain header".to_string());
    }

    let stride = u32::from_le_bytes([raw_data[0], raw_data[1], raw_data[2], raw_data[3]]) as usize;
    let pixel_format = u32::from_le_bytes([raw_data[8], raw_data[9], raw_data[10], raw_data[11]]);

    if pixel_format != 1 {
        return Err(format!("Unsupported pixel format: {} (expecting 1 for RGBA_8888)", pixel_format));
    }

    let pixel_data = &raw_data[16..];

    if pixel_data.len() < stride * height {
        return Err(format!(
            "Pixel data length {} less than stride*height {}",
            pixel_data.len(),
            stride * height
        ));
    }

    let mut rgb_data = Vec::with_capacity(width * height * 3);

    for row in 0..height {
        let row_start = row * stride;
        // Copy only width * 4 bytes from each row (skip stride padding)
        let pixel_row = &pixel_data[row_start..row_start + width * 4];

        for pixel in pixel_row.chunks_exact(4) {
            rgb_data.push(pixel[0]); // R
            rgb_data.push(pixel[1]); // G
            rgb_data.push(pixel[2]); // B
        }
    }

    Ok(rgb_data)
}
