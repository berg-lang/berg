pub mod compile_error;
pub mod source;

pub use compiler::source::Source;
pub use compiler::compile_error::*;
pub use compiler::compile_error::ErrorType::*;

use parser;
use parser::results::*;
use std::env;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct Compiler<'c> {
    root: io::Result<PathBuf>,
    out: Box<Write>,
    err: Box<Write>,

    sources: Vec<Source>,
    errors: RwLock<Vec<CompileError<'c>>>
}

impl<'c> Compiler<'c> {
    pub fn from_env() -> Compiler<'c> {
        let root = env::current_dir();
        let out = Box::new(io::stdout());
        let err = Box::new(io::stderr());
        let errors = RwLock::new(vec![]);
        Compiler { root, out, err, errors, sources: vec![] }
    }
    pub fn new<P: Into<PathBuf>, Out: Into<Box<Write>>, Err: Into<Box<Write>>>(root: P, out: Out, err: Err) -> Self {
        let errors = RwLock::new(vec![]);
        Compiler { root: Ok(root.into()), out: out.into(), err: err.into(), errors, sources: vec![] }
    }

    pub fn root(&self) -> &io::Result<PathBuf> {
        &self.root
    }

    pub fn report(&self, error: CompileError<'c>) {
        let mut guard = self.errors.write().unwrap();
        guard.push(error)
    }

    pub fn add_file_source<P: Into<PathBuf>>(&mut self, path: P) {
        self.sources.push(Source::file(path))
    }

    pub fn add_memory_source<S: Into<String>, B: Into<Vec<u8>>>(&mut self, name: S, contents: B) {
        self.sources.push(Source::memory(name, contents))
    }

    pub fn parse(&'c self) -> Vec<(&'c Source, Option<ParseResult>)> {
        self.sources.iter().map(|source| (source, parser::parse(&self, &source)) ).collect()
    }
}
