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
                    println!("First frame size: {}", raw_data.len());
                }

                match rgba_to_rgb24(&raw_data) {
                    Ok(rgb_frame) => {
                        if tx.send(rgb_frame).is_err() {
                            eprintln!("No active listeners.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Pixel conversion error: {}", e);
                        // fallback to PNG capture
                        if let Ok(png_data) = adb::capture_screen(&device_id) {
                            let _ = tx.send(png_data);
                        }
                    }
                };

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

fn rgba_to_rgb24(rgba: &[u8]) -> Result<Vec<u8>, String> {
    if rgba.len() % 4 != 0 {
        return Err(format!("Input data length {} is not multiple of 4", rgba.len()));
    }
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for chunk in rgba.chunks_exact(4) {
        rgb.push(chunk[0]); // R
        rgb.push(chunk[1]); // G
        rgb.push(chunk[2]); // B
    }
    Ok(rgb)
}
