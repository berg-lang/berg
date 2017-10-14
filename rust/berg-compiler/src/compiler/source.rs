use public::*;
use compiler::Compiler;

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceIndex(pub u32);

#[derive(Debug)]
pub enum SourceSpec {
    File { path: PathBuf },
    Memory { name: String, contents: Vec<u8> },
}

impl SourceSpec {
    pub fn file(path: PathBuf) -> Self {
        SourceSpec::File { path }
    }
    pub fn memory(name: String, contents: Vec<u8>) -> Self {
        let contents = contents;
        SourceSpec::Memory { name, contents }
    }
    pub fn name(&self) -> &OsStr {
        match *self {
            SourceSpec::File { ref path, .. } => path.as_ref(),
            SourceSpec::Memory { ref name, .. } => name.as_ref(),
        }
    }

    pub(crate) fn with_buffer<T, F: FnOnce(&[u8]) -> T>(
        &self,
        compiler: &Compiler,
        source: SourceIndex,
        f: F,
    ) -> T {
        match *self {
            SourceSpec::File { ref path, .. } => Self::open_file(compiler, source, path, f),
            SourceSpec::Memory { ref contents, .. } => f(contents),
        }
    }
    fn open_file<T, F: FnOnce(&[u8]) -> T>(compiler: &Compiler, source: SourceIndex, path: &PathBuf, f: F) -> T {
        if let Some(ref path) = compiler.absolute_path(path, source) {
            match File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    if let Err(error) = file.read_to_end(&mut buffer) {
                        compiler.report_io_read(IoReadError, source, buffer.len() as u32, &error);
                    }
                    f(&buffer)
                }
                Err(error) => {
                    let error_type = match error.kind() {
                        io::ErrorKind::NotFound => SourceNotFound,
                        _ => IoOpenError,
                    };
                    compiler.report_io_open(error_type, source, &error, path.as_path());
                    f(&[])
                }
            }
        } else {
            f(&[])
        }
    }
}

