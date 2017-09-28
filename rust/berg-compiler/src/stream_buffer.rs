use std::io;
use std::ops::Index;
use std::ops::Range;

pub trait StreamBuffer<'a> : Index<usize>, Index<Range<usize>> {
    pub fn offset(&self) -> usize;
    pub fn consume(&mut self, size: usize) -> Box<[u8]>;
    pub fn discard(&mut self, size: usize);
    pub fn current_buffer(&self) -> &'a [u8];
    pub fn fill_buffer(&mut self, min: usize=0, max: usize=READ_SIZE) -> io::Result<&'a [u8]>;
    pub fn eof(&'a mut self) -> bool {
        if let Err(error) = self.slice(1) {
            error.kind == io::ErrorKind::UnexpectedEof
        } else {
            false
        }
    }
}

fn new_eof_error<T>() -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::UnexpectedEof));
}

///
/// An in-memory buffer "stream"
///
pub struct MemoryStreamBuffer<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> MemoryStreamBuffer<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, offset: 0 }
    }
}

impl<'a> StreamBuffer<'a> for MemoryStreamBuffer<'a> {
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn current_buffer(&self) -> &'a [u8] {
        self.fill_buffer[self.offset..]
    }
    pub fn fill_buffer(&mut self, min: usize = 0, max: usize = MIN_READ_SIZE) -> io::Result<&'a [u8]> {
        let end = self.offset + min;
        if end <= self.fill_buffer.len() {
            Ok(self.fill_buffer[self.offset..])
        } else {
            BufferResult::Eof
        }
    }
    pub fn consume(&mut self, size: usize) -> Box<[u8]> {
        let end = self.offset + size;
        let result = self.fill_buffer[self.offset..end];
        self.offset = end;
        result
    }
    pub fn discard(&mut self, size: usize) {
        self.offset += size;
    }
}

impl<'a> Index<usize,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, index: usize) -> io::Result<u8> { self.fill_buffer(index+1)?[index] }
}
impl<'a, R: RangeFull<usize>> Index<R,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, range: R) -> u8 { self.current_buffer() }
}
impl<'a, R: Range<usize>> Index<R,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end)?[range] }
}
impl<'a, R: RangeFrom<usize>> Index<R,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.start)?[range] }
}
impl<'a, R: RangeTo<usize>> Index<R,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end)?[range] }
}
impl<'a, R: RangeInclusive<usize>> Index<R,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end+1)?[range] }
}
impl<'a, R: RangeToInclusive<usize>> Index<R,Output=io::Result<u8>> for MemoryStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end+1)?[range] }
}

pub struct IoStreamBuffer<R: io::Read> {
    reader: R,
    reader_offset: usize,
    buffer: Vec<u8>,
    offset: usize,
}

const MIN_READ_SIZE = 4096*1024;

impl<R: io::Read> IoStreamBuffer<R> {
    pub fn new(reader: R, reader_offset: usize = 0) -> Self {
        let buffer = Vec::with_capacity(MIN_READ_SIZE*2);
        let offset = 0;
        Self { reader, reader_offset, buffer, offset }
    }
    fn read_more(&mut self, max: usize = MIN_READ_SIZE) -> io::Result<usize> {
        self.ensure_unused_capacity(max);
        let old_len = self.fill_buffer.len();
        // Grab the buffer, let it be written to, and then record what we read.
        unsafe {
            self.fill_buffer.set_len(self.fill_buffer.capacity());
            let read_buffer = self.fill_buffer.get_mut_slice()[old_len..];
            let result = reader.read(read_buffer);
            if let Ok(read_size) = result {
                self.fill_buffer.set_len(old_len + read_size);
                if (read_size == 0) {
                    new_eof_error();
                }
            } else {
                self.fill_buffer.set_len(old_len)
            }
            result
        }
    }
    fn ensure_unused_capacity(&mut self, bytes: usize) {
        let unused_capacity = self.fill_buffer.capacity() - self.fill_buffer.len();
        if unused_capacity < bytes {
            unused_capacity += self.trim_consumed();
            if unused_capacity < bytes {
                self.fill_buffer.reserve(unused_capacity);
            }
        }
    }
    fn trim_consumed(&mut self) -> usize {
        // Move the actually-used space to the beginning of the buffer.
        let unconsumed = self.fill_buffer.len() - self.offset;
        let consumed = self.offset;
        if consumed > 0 {
            unsafe {
                let p = self.fill_buffer.as_mut_ptr();
                ptr::copy(p.offset(consumed), p, unconsumed);
                self.fill_buffer.set_len(unconsumed);
            }
            self.offset = 0;
        }
        consumed
    }
}

impl<'a, R: io::Read> StreamBuffer<'a> for IoStreamBuffer<R> {
    pub fn offset(&self) -> usize {
        self.reader_offset + self.offset
    }
    pub fn current_buffer(&self) -> &'a [u8] {
        self.fill_buffer.as_slice()[self.offset..]
    }
    pub fn fill_buffer(&mut self, min: usize=0, max: usize=MIN_READ_SIZE) -> io::Result<&'a u8> {
        while self.fill_buffer.len() < self.offset+min {
            if self.read_more(max)? == 0 {
                return new_eof_error();
            }
        }
        Ok(self.current_buffer())
    }
    pub fn consume(&mut self, size: usize) -> Box<[u8]> {
        let end = self.offset + size;
        let result = self.fill_buffer[self.offset..end];
        self.offset = end;
        self.reader_offset += size;
        Box::new(result.clone())
    }
    pub fn discard(&mut self, size: usize) {
        self.offset = self.offset + size;
        self.reader_offset += size;
    }
}

impl<'a> Index<usize,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, index: usize) -> io::Result<u8> { self.fill_buffer(index+1)?[index] }
}
impl<'a, R: RangeFull<usize>> Index<R,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> u8 { self.current_buffer() }
}
impl<'a, R: Range<usize>> Index<R,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end)?[range] }
}
impl<'a, R: RangeFrom<usize>> Index<R,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.start)?[range] }
}
impl<'a, R: RangeTo<usize>> Index<R,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end)?[range] }
}
impl<'a, R: RangeInclusive<usize>> Index<R,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end+1)?[range] }
}
impl<'a, R: RangeToInclusive<usize>> Index<R,Output=io::Result<u8>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> io::Result<u8> { self.fill_buffer(range.end+1)?[range] }
}
