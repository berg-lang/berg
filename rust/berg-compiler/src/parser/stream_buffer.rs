use std::io;
use std::ptr;

pub trait StreamBuffer {
    fn offset(&self) -> usize;
    fn consume(&mut self, size: usize) -> Vec<u8>;
    fn discard(&mut self, size: usize);
    fn current_buffer<'b>(&'b self) -> &'b [u8];
    fn fill_buffer<'b>(&'b mut self, min: usize) -> io::Result<&'b [u8]>;
    fn eof(&mut self) -> bool {
        if let Err(error) = self.fill_buffer(1) {
            error.kind() == io::ErrorKind::UnexpectedEof
        } else {
            false
        }
    }
}

fn new_eof_error<T>() -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
}

///
/// An in-memory buffer "stream"
///
pub struct MemoryStreamBuffer<'p> {
    buffer: &'p [u8],
    offset: usize,
}

impl<'p> MemoryStreamBuffer<'p> {
    pub fn new(buffer: &'p [u8]) -> Self {
        Self { buffer, offset: 0 }
    }
}

impl<'p> StreamBuffer for MemoryStreamBuffer<'p> {
    fn offset(&self) -> usize {
        self.offset
    }
    fn current_buffer<'b>(&'b self) -> &'b [u8] {
        &self.buffer[self.offset..]
    }
    fn fill_buffer<'b>(&'b mut self, min: usize) -> io::Result<&'b [u8]> {
        let end = self.offset + min;
        if end <= self.buffer.len() {
            Ok(&self.buffer[self.offset..])
        } else {
            new_eof_error()
        }
    }
    fn consume(&mut self, size: usize) -> Vec<u8> {
        let end = self.offset + size;
        let result = self.buffer[self.offset..end].to_vec();
        self.offset = end;
        result
    }
    fn discard(&mut self, size: usize) {
        self.offset += size;
    }
}

pub struct IoStreamBuffer<R: io::Read> {
    reader: R,
    reader_offset: usize,
    eof: bool,
    buffer: Vec<u8>,
    offset: usize,
}

const MIN_READ_SIZE: usize = 4096*1024;

impl<R: io::Read> IoStreamBuffer<R> {
    pub fn new(reader: R, reader_offset: usize) -> Self {
        let eof = false;
        let buffer = Vec::with_capacity(MIN_READ_SIZE*2);
        let offset = 0;
        Self { reader, reader_offset, eof, buffer, offset }
    }
    fn read_more(&mut self, max: usize) -> io::Result<usize> {
        if self.eof {
            return new_eof_error();
        }
        self.ensure_unused_capacity(max);
        let old_len = self.buffer.len();
        let capacity = self.buffer.capacity();
        // Grab the buffer, let it be written to, and then record what we read.
        unsafe {
            self.buffer.set_len(capacity);
            let result = self.reader.read(&mut self.buffer[old_len..]);
            match result {
                Ok(read_size) => {
                    self.buffer.set_len(old_len + read_size);
                    if read_size == 0 {
                        self.eof = true;
                        return new_eof_error();
                    }
                },
                Err(_) => {
                    self.eof = true;
                    self.buffer.set_len(old_len)
                }
            }
            result
        }
    }

    fn ensure_unused_capacity(&mut self, bytes: usize) {
        let mut unused_capacity = self.buffer.capacity() - self.buffer.len();
        if unused_capacity < bytes {
            unused_capacity += self.trim_consumed();
            if unused_capacity < bytes {
                self.buffer.reserve(unused_capacity);
            }
        }
    }

    fn trim_consumed(&mut self) -> usize {
        // Move the actually-used space to the beginning of the buffer.
        let unconsumed = self.buffer.len() - self.offset;
        let consumed = self.offset;
        if consumed > 0 {
            unsafe {
                let p = self.buffer.as_mut_ptr();
                ptr::copy(p.offset(consumed as isize), p, unconsumed);
                self.buffer.set_len(unconsumed);
            }
            self.offset = 0;
        }
        consumed
    }
}

impl<'p, R: io::Read> StreamBuffer for IoStreamBuffer<R> {
    fn offset(&self) -> usize {
        self.reader_offset + self.offset
    }
    fn current_buffer<'b>(&'b self) -> &'b [u8] {
        &self.buffer[self.offset..]
    }
    fn fill_buffer<'b>(&'b mut self, min: usize) -> io::Result<&'b [u8]> {
        while self.buffer.len() < self.offset+min {
            if self.read_more(MIN_READ_SIZE)? == 0 {
                return new_eof_error();
            }
        }
        Ok(self.current_buffer())
    }
    fn consume(&mut self, size: usize) -> Vec<u8> {
        let end = self.offset + size;
        let result = self.buffer[self.offset..end].to_vec();
        self.offset = end;
        self.reader_offset += size;
        result
    }
    fn discard(&mut self, size: usize) {
        self.offset = self.offset + size;
        self.reader_offset += size;
    }
}
