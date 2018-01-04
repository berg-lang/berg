use util::indexed_vec::to_indexed_cow;
use std::fmt::{Debug, Formatter};
use std::fmt;
use compiler::Compiler;
use source::parse_result::{ByteIndex, ParseResult};
use source::ByteSlice;
use std::borrow::Cow;
use source::compile_errors::*;

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

pub(crate) enum SourceSpec {
    File { path: PathBuf },
    Memory { name: String, contents: Vec<u8> },
}

impl Debug for SourceSpec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            SourceSpec::File { ref path } => f.debug_struct("SourceSpec::File")
                .field("path", path)
                .finish(),
            SourceSpec::Memory {
                ref name,
                ref contents,
            } => f.debug_struct("SourceSpec::Memory")
                .field("name", name)
                .field("contents", contents)
                .finish(),
        }
    }
}

impl SourceSpec {
    pub(crate) fn file(path: PathBuf) -> Self {
        SourceSpec::File { path }
    }
    pub(crate) fn memory(name: String, contents: Vec<u8>) -> Self {
        SourceSpec::Memory { name, contents }
    }
    pub(crate) fn name(&self) -> &OsStr {
        match *self {
            SourceSpec::File { ref path, .. } => path.as_ref(),
            SourceSpec::Memory { ref name, .. } => name.as_ref(),
        }
    }

    pub(crate) fn open<'a>(
        &'a self,
        compiler: &Compiler,
        parse_result: &mut ParseResult,
    ) -> Cow<'a, ByteSlice> {
        self.open_and_report(compiler, Some(parse_result))
    }

    pub(crate) fn reopen<'a>(&'a self, compiler: &Compiler) -> Cow<ByteSlice> {
        self.open_and_report(compiler, None)
    }

    pub(crate) fn open_and_report<'a>(
        &'a self,
        compiler: &Compiler,
        mut parse_result: Option<&mut ParseResult>,
    ) -> Cow<'a, ByteSlice> {
        let result = match *self {
            SourceSpec::File { ref path, .. } => Self::open_file(compiler, path, &mut parse_result),
            SourceSpec::Memory { ref contents, .. } => Cow::Borrowed(contents.as_ref()),
        };
        let size = result.len();
        if size >= usize::from(ByteIndex::MAX) {
            parse_result.map(|parse_result| {
                if parse_result.open_error.is_none() {
                    let source = parse_result.index;
                    parse_result.report_open_error(SourceTooLarge::value(source, size));
                }
            });
        }
        to_indexed_cow(result)
    }

    fn open_file(
        compiler: &Compiler,
        path: &PathBuf,
        parse_result: &mut Option<&mut ParseResult>,
    ) -> Cow<'static, [u8]> {
        match compiler.absolute_path(path) {
            Ok(ref path) => match File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    if let Err(error) = file.read_to_end(&mut buffer) {
                        for parse_result in parse_result.iter_mut() {
                            let source = parse_result.index;
                            parse_result.report_open_error(IoReadError::value(
                                source,
                                path.clone(),
                                error.to_string(),
                            ));
                        }
                    }
                    Cow::Owned(buffer)
                }
                Err(error) => {
                    for parse_result in parse_result.iter_mut() {
                        let source = parse_result.index;
                        let path = path.clone();
                        let error_type = match error.kind() {
                            io::ErrorKind::NotFound => SourceNotFound::value,
                            _ => IoOpenError::value,
                        };
                        parse_result.report_open_error(error_type(source, path, error.to_string()))
                    }
                    Cow::Borrowed(&[])
                }
            },
            Err(io_error_string) => {
                if let Some(ref mut parse_result) = *parse_result {
                    let source = parse_result.index;
                    let error =
                        IoCurrentDirectoryError::value(source, path.clone(), io_error_string);
                    parse_result.report_open_error(error);
                }
                Cow::Borrowed(&[])
            }
        }
    }
}
