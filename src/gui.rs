use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use tokio::sync::broadcast::Receiver;
use image::io::Reader as ImageReader;
// use image::DynamicImage;
use std::io::Cursor;

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
        .create_texture_streaming(PixelFormatEnum::RGB24, 1080, 2400)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;



    'running: loop {
        if let Ok(frame) = rx.try_recv() {
            println!("{:?}", &frame[..100]);


            // Debug statement
            println!("Received frame size: {}", frame.len());

            if &frame[..4] == &[137, 80, 78, 71] {
                println!("PNG detected, decoding...");
                match ImageReader::new(Cursor::new(&frame))
                    .with_guessed_format()
                    .map_err(|e| e.to_string())?
                    .decode()
                {
                    Ok(img) => {
                        let rgb_img = img.into_rgb8(); // Convert to RGB8
                        println!("Decoded image size: {}x{}", rgb_img.width(), rgb_img.height());
            
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

            canvas.copy(&texture, None, Some(Rect::new(0, 0, 800, 600)))?;
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
