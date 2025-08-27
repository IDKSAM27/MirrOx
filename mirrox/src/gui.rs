use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use tokio::sync::broadcast::Receiver;
use tokio::sync::watch;
use std::time::Instant;

const PORTRAIT_WIDTH: u32 = 1080;
const PORTRAIT_HEIGHT: u32 = 2400;

pub async fn start_gui(mut rx: Receiver<Vec<u8>>, shutdown_tx: watch::Sender<bool>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut window = video_subsystem
        .window("MirrOx - Optimized", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    window.maximize();

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut phone_width = PORTRAIT_WIDTH;
    let mut phone_height = PORTRAIT_HEIGHT;

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, phone_width, phone_height)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut frame_count = 0;
    let start_time = Instant::now();

    'running: loop {
        if let Ok(frame) = rx.try_recv() {
            frame_count += 1;
            println!("Received frame size: {}", frame.len());

            if frame.len() == (phone_width * phone_height * 3) as usize {
                if let Err(e) = texture.update(None, &frame, (phone_width * 3) as usize) {
                    eprintln!("Texture update error: {}", e);
                } else {
                    let (win_width, win_height) = canvas.window().size();
                    let display_rect = calculate_display_rect(win_width, win_height, phone_width, phone_height);
                    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                    canvas.clear();
                    if let Err(e) = canvas.copy(&texture, None, Some(display_rect)) {
                        eprintln!("Canvas copy error: {}", e);
                    }
                    canvas.present();
                }
            } else {
                eprintln!("Received frame size mismatch: expected {}, got {}", phone_width * phone_height * 3, frame.len());
            }

            if frame_count % 30 == 0 {
                let elapsed = start_time.elapsed();
                let fps = frame_count as f32 / elapsed.as_secs_f32();
                println!("Rendered frames: {}, FPS: {:.2}", frame_count, fps);
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("Window closed, sending shutdown signal...");
                    let _ = shutdown_tx.send(true);
                    break 'running;
                }
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(w, h) = win_event {
                        let display_rect = calculate_display_rect(w as u32, h as u32, phone_width, phone_height);
                        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                        canvas.clear();
                        canvas.copy(&texture, None, Some(display_rect))?;
                        canvas.present();
                    }
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

fn calculate_display_rect(win_width: u32, win_height: u32, phone_width: u32, phone_height: u32) -> Rect {
    let phone_aspect_ratio = phone_width as f32 / phone_height as f32;
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
