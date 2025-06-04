use std::io::{Read, Result};
use crossbeam_channel::{Sender};

#[derive(Debug)]
pub enum ScrcpyFrame {
    Video(Vec<u8>),
    Control(Vec<u8>),
    Clipboard(Vec<u8>),
    DeviceMessage(Vec<u8>),
}

pub fn demux<R: Read + Send + 'static>(
    mut reader: R,
    video_tx: Sender<Vec<u8>>,
    control_tx: Sender<Vec<u8>>,
    clipboard_tx: Sender<Vec<u8>>,
    device_tx: Sender<Vec<u8>>,
) {
    std::thread::spawn(move || {
        loop {
            let mut header = [0u8; 1];
            if reader.read_exact(&mut header).is_err() {
                break;
            }

            let frame_type = header[0];

            match frame_type {
                0x00 => {
                    // Video frame has no length prefix, it's a raw stream (we forward entire reader to video module)
                    let mut buffer = [0u8; 4096];
                    loop {
                        match reader.read(&mut buffer) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                if video_tx.send(buffer[..n].to_vec()).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
                0x01 | 0x02 | 0x03 => {
                    let mut len_buf = [0u8; 4];
                    if reader.read_exact(&mut len_buf).is_err() {
                        break;
                    }
                    let len = u32::from_be_bytes(len_buf) as usize;

                    let mut payload = vec![0u8; len];
                    if reader.read_exact(&mut payload).is_err() {
                        break;
                    }

                    let target = match frame_type {
                        0x01 => &control_tx,
                        0x02 => &clipboard_tx,
                        0x03 => &device_tx,
                        _ => continue,
                    };

                    if target.send(payload).is_err() {
                        break;
                    }
                }
                _ => {
                    // Unknown frame type
                    break;
                }
            }
        }
    });
}
