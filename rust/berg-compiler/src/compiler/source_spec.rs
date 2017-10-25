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

pub(crate) enum SourceSpecBuffer<'b> {
    Owned(Vec<u8>),
    Ref(&'b [u8]),
}

impl<'b> SourceSpecBuffer<'b> {
    pub(crate) fn buffer(&self) -> &[u8] {
        match *self {
            SourceSpecBuffer::Owned(ref vec) => vec.as_slice(),
            SourceSpecBuffer::Ref(vec) => vec,
        }
    }
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

    pub(crate) fn open(
        &self,
        compiler: &Compiler,
        errors: &mut SourceCompileErrors,
    ) -> SourceSpecBuffer {
        match *self {
            SourceSpec::File { ref path, .. } => Self::open_file(compiler, errors, path),
            SourceSpec::Memory { ref contents, .. } => SourceSpecBuffer::Ref(contents),
        }
    }

    fn open_file<'b>(
        compiler: &Compiler,
        errors: &mut SourceCompileErrors,
        path: &PathBuf,
    ) -> SourceSpecBuffer<'b> {
        use CompileErrorType::*;
        if let Some(ref path) = compiler.absolute_path(path, errors) {
            match File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    if let Err(error) = file.read_to_end(&mut buffer) {
                        errors.report_io_read(ByteIndex::from(buffer.len()), &error);
                    }
                    SourceSpecBuffer::Owned(buffer)
                }
                Err(error) => {
                    let error_type = match error.kind() {
                        io::ErrorKind::NotFound => SourceNotFound,
                        _ => IoOpenError,
                    };
                    errors.report_io_open(error_type, &error, path.as_path());
                    SourceSpecBuffer::Ref(&[])
                }
            }
        } else {
            SourceSpecBuffer::Ref(&[])
        }
    }
}
