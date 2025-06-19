use std::io::{Read, Result as IoResult};
use crossbeam_channel::Receiver;

pub struct FifoIO {
    receiver: Receiver<u8>,
    buffer: Vec<u8>,
}

impl FifoIO {
    pub fn new(receiver: Receiver<u8>) -> Self {
        Self {
            receiver,
            buffer: Vec::new(),
        }
    }
}

impl Read for FifoIO {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        while self.buffer.len() < buf.len() {
            match self.receiver.recv() {
                Ok(byte) => self.buffer.push(byte),
                Err(_) => break,
            }
        }

        let n = std::cmp::min(buf.len(), self.buffer.len());
        buf[..n].copy_from_slice(&self.buffer[..n]);
        self.buffer.drain(..n);
        Ok(n)
    }
}
