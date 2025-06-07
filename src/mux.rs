use std::collections::VecDeque;
use std::io::{Read, Result, Seek, SeekFrom};

pub struct FifoIO {
    buffer: VecDeque<u8>,
}

impl FifoIO {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    pub fn push_data(&mut self, data: &[u8]) {
        self.buffer.extend(data);
    }
}

impl Read for FifoIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = std::cmp::min(buf.len(), self.buffer.len());
        for i in 0..n {
            if let Some(b) = self.buffer.pop_front() {
                buf[i] = b;
            }
        }
        Ok(n)
    }
}

impl Seek for FifoIO {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(_) => Ok(0),
            SeekFrom::End(_) => Ok(0),
            SeekFrom::Current(_) => Ok(0),
        }
    }
}
