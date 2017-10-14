use public::*;

use std::any::Any;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::u32;

use std::ops::Index;
use std::ops::Range;
use std::ops::RangeInclusive;

// Wrapper so that it will have ByteIndex sized indexes
#[derive(Debug)]
pub enum SourceHandle<'b> {
    File(Vec<u8>),
    Memory(&'b u8),
    Error,
}

impl<'b> SourceGuard<'b> {
    pub fn open(
        compiler: &Compiler,
        source: SourceIndex,
        source_spec: &'b SourceSpec,
    ) -> Self {
        let buffer = match *source_spec {
            SourceSpec::File { ref path, .. } => Self::open_file(compiler, source, path),
            SourceSpec::Memory { ref contents, .. } => Self::Memory(contents),
        };
        if buffer.buffer.len() > (u32::MAX as usize) {
            compiler.report(SourceTooLarge.source_only(source));
        }
        buffer
    }
    fn open_file(compiler: &Compiler, source: SourceIndex, path: &PathBuf) -> Self {
        if let Some(ref path) = compiler.absolute_path(path, source) {
            match File::open(path) {
                Ok(mut read) => {
                    let mut result = SourceBuffer: Box<Vec<u8>> = Box::new(vec![]);
                    if let Err(error) = read.read_to_end(&mut guard) {
                        compiler.report(IoReadError.io_read(source, guard.len() as u32, error));
                    }
                    
                    let mut result = Self::new(&[], Some(buffer));
                }
                Err(error) => {
                    let error_type = match error.kind() {
                        io::ErrorKind::NotFound => SourceNotFound,
                        _ => IoOpenError,
                    };
                    compiler.report(error_type.io_open(source, error, path.as_path()));
                }
            }
        }
        return SourceBuffer::new(&[], None);
    }
    pub fn buffer(&self) -> &'b u8 {

    }
}

pub struct SourceBuffer<'b>(&'b u8);

impl<'b> SourceBuffer<'b> {
    pub fn len(&self) -> ByteIndex {
        self.buffer.len() as ByteIndex
    }

}
impl<'b> Index<ByteIndex> for SourceBuffer<'b> {
    type Output = u8;
    fn index(&self, index: ByteIndex) -> &u8 {
        &self.buffer[index as usize]
    }
}
impl<'b> Index<Range<ByteIndex>> for SourceBuffer<'b> {
    type Output = [u8];
    fn index(&self, range: Range<ByteIndex>) -> &[u8] {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        &self.buffer[range]
    }
}
impl<'b> Index<RangeInclusive<ByteIndex>> for SourceBuffer<'b> {
    type Output = [u8];
    fn index(&self, range: RangeInclusive<ByteIndex>) -> &[u8] {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        &self.buffer[range]
    }
}
