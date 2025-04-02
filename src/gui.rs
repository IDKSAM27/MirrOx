use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
// use sdl2::render::TextureAccess;
use tokio::sync::broadcast::Receiver;
use image::{save_buffer, ColorType}; // For debugging

// ADB usually sends frames in YUV420, but SDL2 does not support YUV420 directly in Rust.
fn yuv420_to_rgb(yuv: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgb = vec![0u8; width * height * 3];

    let frame_size = width * height;
    let u_offset = frame_size;
    let v_offset = frame_size + (frame_size / 4);

    for j in 0..height {
        for i in 0..width {
            let y = yuv[j * width + i] as i32;
            let u = yuv[u_offset + (j / 2) * (width / 2) + (i / 2)] as i32 - 128;
            let v = yuv[v_offset + (j / 2) * (width / 2) + (i / 2)] as i32 - 128;

            let r = (y + (1.370705 * v as f32) as i32).clamp(0, 255);
            let g = (y - (0.698001 * v as f32 + 0.337633 * u as f32) as i32).clamp(0, 255);
            let b = (y + (1.732446 * u as f32) as i32).clamp(0, 255);

            let index = (j * width + i) * 3;
            rgb[index] = r as u8;
            rgb[index + 1] = g as u8;
            rgb[index + 2] = b as u8;
        }
    }

    rgb
}

pub async fn start_gui(mut rx: Receiver<Vec<u8>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("MirrOx", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 800, 600) // ✅ Use RGB24
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        // Check for a new frame
        if let Ok(frame) = rx.try_recv() {
            // Debug: first 100 bytes
            println!("{:?}", &frame[..100]); 

            // Debug: Save the frame to check its contents
            println!("Received frame size: {}", frame.len());

            save_buffer("debug_frame.png", &frame, 800, 600, ColorType::Rgb8)
                .expect("Failed to save frame");
        
            if frame.len() == 800 * 600 * 3 {
                texture.update(None, &frame, 800 * 3).unwrap();
            } else if frame.len() > 800 * 600 { // If frame is larger than expected, try YUV conversion
                let rgb_frame = yuv420_to_rgb(&frame, 800, 600);
                texture.update(None, &rgb_frame, 800 * 3).unwrap();
            } else {
                eprintln!("Unexpected frame size: {}", frame.len());
            }
        
            canvas.copy(&texture, None, Some(Rect::new(0, 0, 800, 600)))?;
            canvas.present();
        }

        // Handle quit event
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10)); // Prevent CPU overuse
    }
    Ok(())
}
