use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::collections::VecDeque;
use std::io::{Read, Result};
use std::net::TcpStream;

pub struct FifoIO {
    receiver: Receiver<u8>,
    buffer: VecDeque<u8>,
}

impl FifoIO {
    pub fn new(receiver: Receiver<u8>) -> Self {
        FifoIO {
            receiver,
            buffer: VecDeque::with_capacity(8192),
        }
    }
}

impl Read for FifoIO {
    fn read(&mut self, out: &mut [u8]) -> Result<usize> {
        while self.buffer.len() < out.len() {
            match self.receiver.recv() {
                Ok(byte) => self.buffer.push_back(byte),
                Err(_) => break,
            }
        }

        let n = std::cmp::min(out.len(), self.buffer.len());
        for i in 0..n {
            out[i] = self.buffer.pop_front().unwrap();
        }
        Ok(n)
    }
}

pub fn start_mux(mut stream: TcpStream, sender: Sender<u8>) -> Result<()> {
    let mut buffer = [0u8; 4096];
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        for &byte in &buffer[..n] {
            sender.send(byte).unwrap();
        }
    }
    Ok(())
}
