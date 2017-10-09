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
            None => panic!("Parsing is not finished, cannot get char_data"),
        }
    }
    pub fn expressions(&self) -> &[SyntaxExpression] {
        match self.parse_data {
            Some((_, ref expressions)) => expressions,
            None => panic!("Parsing is not finished, cannot get expressions"),
        }
    }

    pub(crate) fn parse_complete(&mut self, char_data: CharData, expressions: Vec<SyntaxExpression>) {
        self.parse_data = Some((char_data, expressions))
    }
}
