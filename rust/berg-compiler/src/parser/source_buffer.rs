use public::*;

use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::Read;
use std::u32;

pub enum SourceBuffer<'b> {
    Empty,
    Vec { buffer: Vec<u8> },
    Ref { buffer: &'b [u8] },
}

impl<'b> SourceBuffer<'b> {
    pub fn with_buffer<'c: 'b, T, F: FnOnce(&[u8]) -> T>(compiler: &Compiler<'c>, source: SourceIndex, s: &Source, f: F) -> T {
        let guard = match *s {
            Source::File { ref path, .. } => Self::open_file(compiler, source, path),
            Source::Memory { ref contents, .. } => SourceBuffer::Ref { buffer: contents.as_slice() },
        };
        let buffer = guard.buffer();
        if buffer.len() > (u32::MAX as usize) {
            panic!("Files larger than 4GB are not supported.")
        }
        f(buffer)
    }
    fn open_file(compiler: &Compiler, source: SourceIndex, path: &PathBuf) -> SourceBuffer<'b> {
        if let Some(ref path) = compiler.absolute_path(path, source) {
            match File::open(path) {
                Ok(mut read) => {
                    let mut buffer = Vec::new();
                    if let Err(error) = read.read_to_end(&mut buffer) {
                        compiler.report(IoReadError.io_read(source, buffer.len() as u32, error));
                    }
                    return SourceBuffer::Vec { buffer }
                },
                Err(error) => {
                    let error_type = match error.kind() {
                        io::ErrorKind::NotFound => SourceNotFound,
                        _ => IoOpenError,
                    };
                    compiler.report(error_type.io_open(source, error, path.as_path()));
                }
            }
        }
        SourceBuffer::Empty
    }
    pub fn buffer(&self) -> &[u8] {
        static EMPTY_BUFFER: [u8; 0] = [];
        match *self {
            SourceBuffer::Empty => &EMPTY_BUFFER,
            SourceBuffer::Vec { ref buffer } => buffer,
            SourceBuffer::Ref { buffer } => buffer,
        }
    }
}
