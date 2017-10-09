pub mod compile_error;
pub mod source;

use public::*;
use parser::Parser;

use std::env;
use std::fmt::*;
use std::io;
use std::io::Write;
use std::ops::Index;
use std::ops::IndexMut;
use std::path::PathBuf;
use std::sync::RwLock;
use std::u32;

pub struct Compiler<'c> {
    root: Option<PathBuf>,
    root_error: RwLock<Option<io::Error>>,
    out: Box<Write>,
    err: Box<Write>,

    sources: RwLock<Vec<SourceData<'c>>>,
    errors: RwLock<Vec<CompileError>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceIndex(pub u32);

impl<'c> Debug for Compiler<'c> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Foo")
            .field("root", &self.root)
            .field("root_error", &self.root_error)
            .field("sources", &self.sources)
            .field("errors", &self.errors)
            .finish()
    }
}

impl<'c> Index<SourceIndex> for Vec<SourceData<'c>> {
    type Output = SourceData<'c>;
    fn index(&self, index: SourceIndex) -> &SourceData<'c> {
        &self[index.0 as usize]
    }
}
impl<'c> IndexMut<SourceIndex> for Vec<SourceData<'c>> {
    fn index_mut(&mut self, index: SourceIndex) -> &mut SourceData<'c> {
        &mut self[index.0 as usize]
    }
}

//
// Implementation
//

impl<'c> Compiler<'c> {
    pub fn from_env() -> Self {
        let mut root = None;
        let mut root_error = None;
        match env::current_dir() {
            Ok(path) => root = Some(path),
            Err(error) => root_error = Some(error),
        }
        let out = Box::new(io::stdout());
        let err = Box::new(io::stderr());
        Self::new(root, root_error, out, err)
    }

    pub fn new(
        root: Option<PathBuf>,
        root_error: Option<io::Error>,
        out: Box<Write>,
        err: Box<Write>,
    ) -> Self {
        let root_error = RwLock::new(root_error);
        let out = out.into();
        let err = err.into();
        let sources = RwLock::new(vec![]);
        let errors = RwLock::new(vec![]);
        Compiler {
            root,
            root_error,
            out,
            err,
            sources,
            errors,
        }
    }

    pub fn add_file_source<P: Into<PathBuf>>(&mut self, path: P) {
        let source = Source::file(path.into());
        self.add_source(source)
    }

    pub fn add_memory_source<Str: Into<String>, Buf: Into<Vec<u8>>>(
        &mut self,
        name: Str,
        contents: Buf,
    ) {
        let source = Source::memory(name.into(), contents.into());
        self.add_source(source)
    }

    pub fn with_sources<T, F: FnOnce(&[SourceData]) -> T>(
        &self,
        f: F,
    ) -> T {
        let sources = self.sources.read().unwrap();
        f(sources.as_slice())
    }

    pub fn with_errors<T, F: FnOnce(&[CompileError]) -> T>(
        &self,
        f: F,
    ) -> T {
        let errors = self.errors.read().unwrap();
        f(errors.as_slice())
    }

    pub fn with_source<T, F: FnOnce(&SourceData) -> T>(
        &self,
        index: SourceIndex,
        f: F,
    ) -> T {
        self.with_sources(|sources| f(&sources[index.0 as usize]))
    }

    fn add_source(&self, source: Source) {
        let index = {
            let mut sources = self.sources.write().unwrap();
            if sources.len() + 1 > (u32::MAX as usize) {
                panic!("Too many source files opened! Max is {}", u32::MAX)
            }
            sources.push(SourceData::new(source));
            SourceIndex((sources.len() - 1) as u32)
        };
        Parser::parse(self, index);
        self.with_source(index, |source| {
            println!("{}", source.name().to_string_lossy());
            println!("--------------------");
            println!("Result:");
            for expression in source.expressions() {
                println!(
                    "- {}: {:?} \"{}\"",
                    source.char_data().range(expression.range()),
                    expression.expression_type,
                    expression.string
                );
            }
        });
        let errors = self.errors.read().unwrap();
        if errors.len() > 0 {
            println!("");
            println!("ERRORS:");
            for error in errors.iter() {
                println!("- {:?}", error);
            }
        }
    }

    pub(crate) fn with_source_mut<T, F: FnOnce(&mut SourceData) -> T>(
        &self,
        index: SourceIndex,
        f: F,
    ) -> T {
        let mut sources = self.sources.write().unwrap();
        let source_data = &mut sources[index.0 as usize];
        f(source_data)
    }

    fn maybe_report_path_error(&self, source: SourceIndex) {
        // Only report the "bad root directory" error once.
        let mut root_error = self.root_error.write().unwrap();
        if let Some(ref error) = *root_error {
            self.report(IoCurrentDirectoryError.io_generic(source, error));
        } else {
            return;
        }
        *root_error = None;
    }

    pub(crate) fn absolute_path(&self, path: &PathBuf, source: SourceIndex) -> Option<PathBuf> {
        if path.is_relative() {
            if let Some(ref root) = self.root {
                Some(root.join(path))
            } else {
                self.maybe_report_path_error(source);
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn report(&self, error: CompileError) {
        let mut errors = self.errors.write().unwrap();
        errors.push(error)
    }
}
