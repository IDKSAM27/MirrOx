use crossbeam_channel::{bounded, Receiver};
use std::collections::VecDeque;
use std::io::{Read, Result as IoResult};
use std::net::TcpStream;
use std::os::raw::{c_int, c_void};

const VIDEO_CHANNEL_ID: u8 = 0;

pub fn start_muxed_stream() -> IoResult<Receiver<Vec<u8>>> {
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    let (sender, receiver) = bounded::<Vec<u8>>(32);

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut scratch = VecDeque::new();

        loop {
            // Read from TCP stream
            let len = match stream.read(&mut buf) {
                Ok(0) => {
                    eprintln!("[mux] TCP stream closed");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[mux] TCP read error: {e}");
                    break;
                }
            };

            scratch.extend(&buf[..len]);

            // Extract as many complete packets as possible
            while scratch.len() >= 5 {
                let channel_id = scratch[0];
                let length = u32::from_be_bytes([
                    scratch[1], scratch[2], scratch[3], scratch[4],
                ]) as usize;

                if scratch.len() < 5 + length {
                    // Wait for more bytes
                    break;
                }

                scratch.drain(0..5); // Remove header

                let packet: Vec<u8> = scratch.drain(0..length).collect();

                if channel_id == VIDEO_CHANNEL_ID {
                    if let Err(e) = sender.send(packet) {
                        eprintln!("[mux] Failed to send video packet: {e}");
                        break;
                    }
                }
            }
        }

        eprintln!("[mux] Demux thread exiting");
    });

    Ok(receiver)
}

pub struct FifoIO {
    queue: VecDeque<u8>,
    receiver: Receiver<Vec<u8>>,
}

impl FifoIO {
    pub fn new(receiver: Receiver<Vec<u8>>) -> Self {
        Self {
            queue: VecDeque::new(),
            receiver,
        }
    }

    pub extern "C" fn read_packet(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int {
        let fifo = unsafe { &mut *(opaque as *mut FifoIO) };

        while fifo.queue.len() < buf_size as usize {
            match fifo.receiver.recv() {
                Ok(data) => {
                    fifo.queue.extend(data);
                }
                Err(_) => {
                    eprintln!("[mux] channel closed, returning EOF");
                    return 0; // EOF
                }
            }
        }

        let len = buf_size.min(fifo.queue.len() as i32);
        for i in 0..len as usize {
            unsafe {
                *buf.add(i) = fifo.queue.pop_front().unwrap();
            }
        }

        eprintln!("[mux] read_packet: delivered {} bytes", len);
        len
    }
}
