use compiler::source_data::ByteSlice;
use std::borrow::Cow;
use indexed_vec::IndexedVec;
use compiler::compile_errors::SourceCompileErrors;
use public::*;
use compiler::Compiler;

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

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

    pub(crate) fn open<'b>(
        &'b self,
        compiler: &Compiler,
        errors: &mut SourceCompileErrors,
    ) -> Cow<'b, ByteSlice> {
        match *self {
            SourceSpec::File { ref path, .. } => Self::open_file(compiler, errors, path),
            SourceSpec::Memory { ref contents, .. } => Cow::Borrowed(ByteSlice::from_slice(contents)),
        }
    }

    fn open_file<'b>(
        compiler: &Compiler,
        errors: &mut SourceCompileErrors,
        path: &PathBuf,
    ) -> Cow<'b, ByteSlice> {
        use CompileErrorType::*;
        if let Some(ref path) = compiler.absolute_path(path, errors) {
            match File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    if let Err(error) = file.read_to_end(&mut buffer) {
                        errors.report_io_read(ByteIndex::from(buffer.len()), &error);
                    }
                    Cow::Owned(IndexedVec::from(buffer))
                }
                Err(error) => {
                    let error_type = match error.kind() {
                        io::ErrorKind::NotFound => SourceNotFound,
                        _ => IoOpenError,
                    };
                    errors.report_io_open(error_type, &error, path.as_path());
                    Cow::Borrowed(ByteSlice::from_slice(&[]))
                }
            }
        } else {
            Cow::Borrowed(ByteSlice::from_slice(&[]))
        }
    }
}
