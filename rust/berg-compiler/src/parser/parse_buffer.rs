use public::*;

use std::ops::Index;
use std::ops::Range;
use std::ops::RangeInclusive;

// Wrapper so that it will have ByteIndex sized indexes
#[derive(Debug)]
pub struct ParseBuffer<'p> {
    buffer: &'p [u8],
}

impl<'p> ParseBuffer<'p> {
    pub fn new(buffer: &'p [u8]) -> Self {
        ParseBuffer { buffer }
    }
    pub fn len(&self) -> ByteIndex {
        self.buffer.len() as ByteIndex
    }
}
impl<'p> Index<ByteIndex> for ParseBuffer<'p> {
    type Output = u8;
    fn index(&self, index: ByteIndex) -> &u8 {
        &self.buffer[index as usize]
    }
}
impl<'p> Index<Range<ByteIndex>> for ParseBuffer<'p> {
    type Output = [u8];
    fn index(&self, range: Range<ByteIndex>) -> &[u8] {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        &self.buffer[range]
    }
}
impl<'p> Index<RangeInclusive<ByteIndex>> for ParseBuffer<'p> {
    type Output = [u8];
    fn index(&self, range: RangeInclusive<ByteIndex>) -> &[u8] {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        &self.buffer[range]
    }
}
