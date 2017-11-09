use compiler::source_data::ByteSlice;
use std::borrow::Cow;
use indexed_vec::IndexedVec;
use compiler::compile_errors;
use compiler::Compiler;
use compiler::source_data::{ByteIndex,SourceIndex};

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum SourceSpec<'s> {
    File { path: PathBuf },
    Memory { name: &'s str, contents: &'s [u8] },
}

impl<'s> SourceSpec<'s> {
    pub(crate) fn file(path: PathBuf) -> Self {
        SourceSpec::File { path }
    }
    pub(crate) fn memory(name: &'s str, contents: &'s [u8]) -> Self {
        let contents = contents;
        SourceSpec::Memory { name, contents }
    }
    pub(crate) fn name(&self) -> &OsStr {
        match *self {
            SourceSpec::File { ref path, .. } => path.as_ref(),
            SourceSpec::Memory { name, .. } => name.as_ref(),
        }
    }

    pub(crate) fn open<'p>(
        &'p self,
        compiler: &'p Compiler,
        source: SourceIndex,
    ) -> Cow<'s, ByteSlice> {
        let result = match *self {
            SourceSpec::File { ref path, .. } => Self::open_file(compiler, source, path),
            SourceSpec::Memory { contents, .. } => Cow::Borrowed(ByteSlice::from_slice(contents)),
        };
        if result.as_raw_slice().len() >= usize::from(ByteIndex::MAX) {
            compiler.report(compile_errors::SourceTooLarge { source, size: result.as_raw_slice().len() })
        }
        result
    }

    fn open_file(
        compiler: &Compiler,
        source: SourceIndex,
        path: &PathBuf,
    ) -> Cow<'s, ByteSlice> {
        if let Some(ref path) = compiler.absolute_path(path, source) {
            match File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    if let Err(error) = file.read_to_end(&mut buffer) {
                        let range = ByteIndex::from(buffer.len())..ByteIndex::from(buffer.len());
                        compiler.report(compile_errors::IoReadError { source, range, path: path.clone(), io_error_string: error.to_string() });
                    }
                    Cow::Owned(IndexedVec::from(buffer))
                }
                Err(error) => {
                    match error.kind() {
                        io::ErrorKind::NotFound => compiler.report(compile_errors::SourceNotFound { source, path: path.clone(), io_error_string: error.to_string() }),
                        _ => compiler.report(compile_errors::IoOpenError { source, path: path.clone(), io_error_string: error.to_string() }),
                    };
                    Cow::Borrowed(ByteSlice::from_slice(&[]))
                }
            }
        } else {
            Cow::Borrowed(ByteSlice::from_slice(&[]))
        }
    }
}
