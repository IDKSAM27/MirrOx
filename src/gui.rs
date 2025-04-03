use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use tokio::sync::broadcast::Receiver;
use image::io::Reader as ImageReader;
use std::io::Cursor;

pub async fn start_gui(mut rx: Receiver<Vec<u8>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut window = video_subsystem
        .window("MirrOx", 800, 600)
        .position_centered()
        .resizable() // allows resizing of the window, quite imp
        .build()
        .map_err(|e| e.to_string())?;

    window.maximize();

    // Get window size BEFORE moving window into canvas
    let (win_width, win_height) = window.size();
    println!("Window Size: {}x{}", win_width, win_height);
    
    // Now move window into canvas
    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 1080, 2400)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        if let Ok(frame) = rx.try_recv() {
            // Debug statement
            // println!("{:?}", &frame[..100]);


            // Debug statement
            // println!("Received frame size: {}", frame.len());

            if &frame[..4] == &[137, 80, 78, 71] {
                // println!("PNG detected, decoding...");
                match ImageReader::new(Cursor::new(&frame))
                    .with_guessed_format()
                    .map_err(|e| e.to_string())?
                    .decode()
                {
                    Ok(img) => {
                        let rgb_img = img.into_rgb8(); // Convert to RGB8
                        // println!("Decoded image size: {}x{}", rgb_img.width(), rgb_img.height());
            
                        if rgb_img.width() != 1080 || rgb_img.height() != 2400 {
                            println!("Warning: Image size mismatch! Expected 1080x2400.");
                        }
            
                        // Update texture (Stride = width * bytes_per_pixel)
                        texture.update(None, &rgb_img, 1080 * 3).unwrap();
                        canvas.copy(&texture, None, Some(Rect::new(0, 0, 1080, 2400)))?;
                        canvas.present();
                    }
                    Err(e) => eprintln!("Failed to decode PNG: {}", e),
                }
            }

            // // Original phone resolution
            // let phone_width = 1080;
            // let phone_height = 2400;

            // // Calculate scaling factor to fit within window while keeping aspect ratio
            // let scale_x = win_width as f32 / phone_width as f32;
            // let scale_y = win_height as f32 / phone_height as f32;
            // let scale = scale_x.min(scale_y); // Use the smaller scale to fit

            // // Calculate new size
            // let new_width = (phone_width as f32 * scale) as u32;
            // let new_height = (phone_height as f32 * scale) as u32;

            // // Centering
            // let x_offset = (win_width - new_width) / 2;
            // let y_offset = (win_height - new_height) / 2;

            // // Render image
            // let dst_rect = Rect::new(x_offset as i32, y_offset as i32, new_width, new_height);
            // canvas.clear();
            // canvas.copy(&texture, None, dst_rect)?; // Scale & center image
            // canvas.present();

            // Set clear color to black (prevents ghosting effect)
            canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas.clear();

            // Maintain correct aspect ratio for phone screen (1080x2400)
            let phone_aspect_ratio = 1080.0 / 2400.0;
            let win_aspect_ratio = win_width as f32 / win_height as f32;

            let display_rect = if win_aspect_ratio > phone_aspect_ratio {
                let new_width = (win_height as f32 * phone_aspect_ratio) as u32;
                let x_offset = (win_width - new_width) / 2;
                Rect::new(x_offset as i32, 0, new_width, win_height)
            } else {
                let new_height = (win_width as f32 / phone_aspect_ratio) as u32;
                let y_offset = (win_height - new_height) / 2;
                Rect::new(0, y_offset as i32, win_width, new_height)
            };

            // Clear screen before rendering new frame
            canvas.clear();

            // Render the phone screen inside the display_rect
            canvas.copy(&texture, None, Some(display_rect))?;
            canvas.present();
            

        }

        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
