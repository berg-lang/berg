use public::*;
use compiler::Compiler;

use std::ffi::OsStr;
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
}

#[derive(Debug)]
pub struct SourceData<'c> {
    pub source: Source,
    pub char_data: Option<CharData>,
    pub expressions: Option<Vec<SyntaxExpression>>,
    phantom: PhantomData<&'c Compiler<'c>>,
}

impl<'c> SourceData<'c> {
    pub fn new(source: Source) -> Self {
        SourceData {
            source,
            char_data: None,
            expressions: None,
            phantom: PhantomData,
        }
    }
}
