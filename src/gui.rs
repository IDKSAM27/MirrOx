use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use tokio::sync::broadcast::Receiver;
use image::io::Reader as ImageReader;
use std::io::Cursor;

pub async fn start_gui(mut rx: Receiver<Vec<u8>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut window = video_subsystem
        .window("MirrOx", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    window.maximize();

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 1080, 2400)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let (mut win_width, mut win_height) = canvas.output_size()?;
    
    // This calculates the connected device dimensions
    fn calculate_display_rect(win_width: u32, win_height: u32) -> Rect {
        let phone_aspect_ratio = 1080.0 / 2400.0;
        let win_aspect_ratio = win_width as f32 / win_height as f32;

        if win_aspect_ratio > phone_aspect_ratio {
            let new_width = (win_height as f32 * phone_aspect_ratio) as u32;
            let x_offset = (win_width - new_width) / 2;
            Rect::new(x_offset as i32, 0, new_width, win_height)
        } else {
            let new_height = (win_width as f32 / phone_aspect_ratio) as u32;
            let y_offset = (win_height - new_height) / 2;
            Rect::new(0, y_offset as i32, win_width, new_height)
        }
    }
    
    let mut display_rect = calculate_display_rect(win_width, win_height);

    'running: loop {
        if let Ok(frame) = rx.try_recv() {
            // below 4 frames are supposed to be PNG's initial 4 frames
            if &frame[..4] == &[137, 80, 78, 71] {
                match ImageReader::new(Cursor::new(&frame))
                    .with_guessed_format()
                    .map_err(|e| e.to_string())?
                    .decode()
                {
                    Ok(img) => {
                        let rgb_img = img.into_rgb8();
                        texture.update(None, &rgb_img, 1080 * 3).unwrap();
                        canvas.copy(&texture, None, Some(display_rect))?;
                        canvas.present();
                    }
                    Err(e) => eprintln!("Failed to decode PNG: {}", e),
                }
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(w, h) = win_event {
                        win_width = w as u32;
                        win_height = h as u32;
                        display_rect = calculate_display_rect(win_width, win_height);
                    }
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
