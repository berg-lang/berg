use public::*;
use compiler::Compiler;

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Source {
    File { path: PathBuf },
    Memory { name: String, contents: Vec<u8> },
}

impl Source {
    pub fn file(path: PathBuf) -> Self {
        Source::File { path }
    }
    pub fn memory(name: String, contents: Vec<u8>) -> Self {
        let contents = contents;
        Source::Memory { name, contents }
    }
    pub fn name(&self) -> &OsStr {
        match *self {
            Source::File { ref path, .. } => path.as_ref(),
            Source::Memory { ref name, .. } => name.as_ref(),
        }
    }

    pub(crate) fn with_buffer<T, F: FnOnce(&[u8]) -> T>(
        &self,
        compiler: &Compiler,
        source: SourceIndex,
        f: F,
    ) -> T {
        match *self {
            Source::File { ref path, .. } => Self::open_file(compiler, source, path, f),
            Source::Memory { ref contents, .. } => f(contents)
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

#[derive(Debug)]
pub struct SourceData<'c> {
    source: Source,
    parse_data: Option<(CharData, Vec<SyntaxExpression>)>,
    phantom: PhantomData<&'c Compiler<'c>>,
}

impl<'c> SourceData<'c> {
    pub fn new(source: Source) -> Self {
        SourceData {
            source,
            parse_data: None,
            phantom: PhantomData,
        }
    }

    pub fn source(&self) -> &Source { &self.source }
    pub fn name(&self) -> &OsStr { self.source.name() }
    pub fn parsed(&self) -> bool { self.parse_data.is_some() }
    pub fn char_data(&self) -> &CharData {
        match self.parse_data {
            Some((ref char_data, _)) => char_data,
            None => unreachable!(),
        }
    }
    pub fn expressions(&self) -> &[SyntaxExpression] {
        match self.parse_data {
            Some((_, ref expressions)) => expressions,
            None => unreachable!(),
        }
    }

    pub(crate) fn parse_complete(&mut self, char_data: CharData, expressions: Vec<SyntaxExpression>) {
        self.parse_data = Some((char_data, expressions))
    }
}
