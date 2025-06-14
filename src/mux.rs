use crossbeam_channel::{bounded, Receiver};
use std::collections::VecDeque;
use std::io::{Read, Result as IoResult};
use std::net::TcpStream;
use std::os::raw::{c_int, c_void};

pub fn start_muxed_stream() -> IoResult<Receiver<Vec<u8>>> {
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    let (sender, receiver) = bounded::<Vec<u8>>(32);

    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(len) = stream.read(&mut buffer) {
            if len == 0 {
                break;
            }
            if sender.send(buffer[..len].to_vec()).is_err() {
                break;
            }
        }
    });

    Ok(receiver)
}

pub struct FifoIO {
    queue: VecDeque<u8>,
    pub receiver: Receiver<Vec<u8>>,
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
                Ok(data) => fifo.queue.extend(data),
                Err(_) => break,
            }
        }

        let len = buf_size.min(fifo.queue.len() as i32);
        for i in 0..len as usize {
            unsafe {
                *buf.add(i) = fifo.queue.pop_front().unwrap();
            }
        }

        println!("[mux] read_packet: delivered {} bytes", len);
        len
    }
}
