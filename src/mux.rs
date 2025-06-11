use std::io::{Read, Result};
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use crossbeam_channel::{bounded, Receiver};

/// Buffer-based reader that implements FFmpeg-compatible callback reading.
pub struct FifoIO {
    receiver: Receiver<Vec<u8>>,
    current_chunk: Vec<u8>,
    position: usize,
}

impl FifoIO {
    pub fn new(receiver: Receiver<Vec<u8>>) -> Self {
        FifoIO {
            receiver,
            current_chunk: Vec::new(),
            position: 0,
        }
    }

    extern "C" fn read_packet(
        opaque: *mut std::ffi::c_void,
        buf: *mut u8,
        buf_size: i32,
    ) -> i32 {
        let fifo = unsafe { &mut *(opaque as *mut FifoIO) };

        loop {
            if fifo.position < fifo.current_chunk.len() {
                let available = &fifo.current_chunk[fifo.position..];
                let to_copy = available.len().min(buf_size as usize);

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        available.as_ptr(),
                        buf,
                        to_copy,
                    );
                }

                fifo.position += to_copy;
                return to_copy as i32;
            }

            // Load next chunk
            match fifo.receiver.recv() {
                Ok(chunk) => {
                    fifo.current_chunk = chunk;
                    fifo.position = 0;
                }
                Err(_) => return 0, // EOF
            }
        }
    }
}

/// Reads and demuxes the scrcpy server stream, sending only video packets via channel.
pub fn spawn_mux_channel(mut stream: TcpStream) -> Result<Receiver<Vec<u8>>> {
    let (sender, receiver) = bounded::<Vec<u8>>(100);
    std::thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let _ = sender.send(buffer[..n].to_vec());
                }
                Err(_) => break,
            }
        }
    });
    Ok(receiver)
}
