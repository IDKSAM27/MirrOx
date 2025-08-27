use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};
use crate::adb::AdbShell;

pub fn parse_screenshot(raw_data: Vec<u8>, output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&raw_data)
        .map_err(|e| format!("Failed to write image data: {}", e))?;
    println!("Screenshot saved as {}", output_path);
    Ok(())
}

pub async fn start_video_stream_optimized(
    tx: Arc<broadcast::Sender<Vec<u8>>>,
    device_id: String,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    println!("Starting optimized video stream for device: {}", device_id);
    
    // Initialize persistent ADB shell
    let mut adb_shell = match AdbShell::new(&device_id) {
        Ok(shell) => shell,
        Err(e) => {
            eprintln!("Failed to create ADB shell: {}", e);
            return;
        }
    };
    
    // Get screen dimensions
    let (screen_width, screen_height) = match adb_shell.get_screen_info() {
        Ok((w, h)) => (w, h),
        Err(e) => {
            eprintln!("Failed to get screen info: {}, using defaults", e);
            (1080, 2400)
        }
    };
    
    println!("Screen dimensions: {}x{}", screen_width, screen_height);
    
    let mut frame_count = 0;
    let mut total_capture_time = Duration::new(0, 0);
    let mut last_fps_report = Instant::now();
    let mut consecutive_errors = 0;
    let max_consecutive_errors = 10;
    
    // Target frame time for 30 FPS
    let target_frame_time = Duration::from_millis(33);
    
    loop {
        if *shutdown_rx.borrow() {
            println!("Shutting down optimized video stream...");
            break;
        }
        
        let frame_start = Instant::now();
        let capture_start = Instant::now();
        
        match adb_shell.capture_screen_raw() {
            Ok(raw_data) => {
                let capture_time = capture_start.elapsed();
                total_capture_time += capture_time;
                frame_count += 1;
                consecutive_errors = 0;
                
                // Process raw framebuffer data to create a proper frame
                if let Ok(processed_frame) = process_raw_framebuffer(&raw_data, screen_width, screen_height) {
                    if tx.send(processed_frame).is_err() {
                        eprintln!("No active listeners.");
                    }
                } else {
                    // Fallback to PNG capture if raw processing fails
                    match crate::adb::capture_screen(&device_id) {
                        Ok(png_data) => {
                            if tx.send(png_data).is_err() {
                                eprintln!("No active listeners.");
                            }
                        }
                        Err(e) => {
                            consecutive_errors += 1;
                            if consecutive_errors >= max_consecutive_errors {
                                eprintln!("Too many consecutive errors, stopping stream");
                                break;
                            }
                        }
                    }
                }
                
                // Performance reporting every 5 seconds
                if last_fps_report.elapsed() >= Duration::from_secs(5) {
                    let avg_fps = frame_count as f32 / last_fps_report.elapsed().as_secs_f32();
                    let avg_capture_time = total_capture_time.as_millis() as f32 / frame_count as f32;
                    
                    println!(
                        "Performance - FPS: {:.1}, Avg Capture Time: {:.1}ms, Frames: {}",
                        avg_fps, avg_capture_time, frame_count
                    );
                    
                    // Reset counters
                    frame_count = 0;
                    total_capture_time = Duration::new(0, 0);
                    last_fps_report = Instant::now();
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                eprintln!("Frame capture error ({}): {}", consecutive_errors, e);
                
                if consecutive_errors >= max_consecutive_errors {
                    eprintln!("Too many consecutive errors, stopping stream");
                    break;
                }
                
                // Back off on errors
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        }
        
        // Maintain target frame rate
        let frame_duration = frame_start.elapsed();
        if frame_duration < target_frame_time {
            tokio::time::sleep(target_frame_time - frame_duration).await;
        }
    }
    
    println!("Video stream ended");
}

fn process_raw_framebuffer(raw_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    // Raw framebuffer data is typically in RGBA format
    // We need to convert it to a format that can be displayed
    
    if raw_data.len() < 12 {
        return Err("Invalid raw data - too small".to_string());
    }
    
    // Check if this is actually PNG data (starts with PNG header)
    if raw_data.starts_with(&[137, 80, 78, 71]) {
        // It's PNG data, return as-is
        return Ok(raw_data.to_vec());
    }
    
    // Parse raw framebuffer header (first 12 bytes typically contain metadata)
    // Format: width(4) + height(4) + format(4) + ...
    let fb_width = u32::from_le_bytes([raw_data[0], raw_data[1], raw_data[2], raw_data[3]]);
    let fb_height = u32::from_le_bytes([raw_data[4], raw_data[5], raw_data[6], raw_data[7]]);
    let fb_format = u32::from_le_bytes([raw_data[8], raw_data[9], raw_data[10], raw_data[11]]);
    
    // Skip header and get pixel data
    let pixel_data = &raw_data[12..];
    
    // For now, we'll create a simple bitmap format that SDL2 can handle
    // This is a simplified conversion - in practice, you'd want proper format handling
    create_bitmap_from_framebuffer(pixel_data, fb_width, fb_height, fb_format)
}

fn create_bitmap_from_framebuffer(pixel_data: &[u8], width: u32, height: u32, format: u32) -> Result<Vec<u8>, String> {
    // This is a simplified implementation
    // In a real implementation, you'd handle different pixel formats properly
    
    // For now, let's create a simple RGB bitmap header + data
    let mut result = Vec::new();
    
    // Add a simple header identifying this as processed framebuffer data
    result.extend_from_slice(b"MIRR"); // Magic number for MirrOx format
    result.extend_from_slice(&width.to_le_bytes());
    result.extend_from_slice(&height.to_le_bytes());
    result.extend_from_slice(&format.to_le_bytes());
    
    // Add pixel data (this might need format conversion)
    result.extend_from_slice(pixel_data);
    
    Ok(result)
}

// Legacy function for compatibility
pub async fn start_video_stream(
    tx: Arc<broadcast::Sender<Vec<u8>>>,
    device_id: String,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    start_video_stream_optimized(tx, device_id, shutdown_rx).await;
}
