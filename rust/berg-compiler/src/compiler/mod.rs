pub mod compile_errors;
pub(crate) mod line_column;
pub(crate) mod source_spec;
pub(crate) mod source_data;

use checker;
use compiler::compile_errors::{CompileError,CompileErrorLocation};
use compiler::source_data::{SourceData,SourceIndex};
use compiler::source_spec::SourceSpec;
use indexed_vec::IndexedVec;
use parser;
use std::default::Default;
use std::env;
use std::fmt::*;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct Compiler<'c> {
    root: Option<PathBuf>,
    root_error: RwLock<Option<io::Error>>,
    out: Box<Write>,
    err: Box<Write>,
    sources: RwLock<IndexedVec<SourceData<'c>, SourceIndex>>,
    pub(crate) errors: RwLock<Vec<Box<CompileError+'c>>>,
}

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
        let sources = RwLock::default();
        let errors = RwLock::default();
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
        let source = SourceSpec::file(path.into());
        self.add_source(source)
    }

    pub fn add_memory_source<'s: 'c>(
        &mut self,
        name: &'s str,
        contents: &'s [u8],
    ) {
        let source = SourceSpec::memory(name, contents);
        self.add_source(source)
    }

    pub fn with_sources<T, F: FnOnce(&IndexedVec<SourceData<'c>, SourceIndex>) -> T>(&self, f: F) -> T {
        let sources = self.sources.read().unwrap();
        f(&sources)
    }

    pub fn with_errors<T, F: FnOnce(&Vec<Box<CompileError+'c>>) -> T>(&self, f: F) -> T {
        let errors = self.errors.read().unwrap();
        f(&errors)
    }

    pub(crate) fn report<T: CompileError+'c>(&self, error: T) {
        self.with_errors_mut(|errors| errors.push(Box::new(error)))
    }

    pub fn with_source<T, F: FnOnce(&SourceData<'c>) -> T>(&self, index: SourceIndex, f: F) -> T {
        self.with_sources(|sources| f(&sources[index]))
    }

    pub(crate) fn with_source_mut<T, F: FnOnce(&mut SourceData<'c>) -> T>(
        &self,
        index: SourceIndex,
        f: F,
    ) -> T {
        let mut sources = self.sources.write().unwrap();
        f(&mut sources[index])
    }

    pub(crate) fn with_errors_mut<T, F: FnOnce(&mut Vec<Box<CompileError+'c>>) -> T>(
        &self,
        f: F,
    ) -> T {
        let mut errors = self.errors.write().unwrap();
        f(&mut errors)
    }

    fn add_source<'s: 'c>(&self, source_spec: SourceSpec<'s>) {
        let index = {
            let mut sources = self.sources.write().unwrap();
            if sources.len() + 1 > SourceIndex::MAX {
                self.report(compile_errors::TooManySources { num_sources: usize::from(sources.len()) + 1 });
            }
            sources.push(SourceData::new(source_spec));
            sources.len() - 1
        };

        self.with_source(index, |source| println!("{}", source.name().to_string_lossy()));

        let parse_data = parser::parse(self, index);

        println!("--------------------");
        println!("PARSE RESULT:");
        print!("{}", parse_data);

        let checked_type = checker::check(&parse_data, self, index);

        println!("--------------------");
        println!("CHECK RESULT:");
        println!("{:?}", checked_type);

        let errors = self.errors.read().unwrap();
        if !errors.is_empty() {
            println!();
            println!("ERRORS:");
            for error in errors.iter() {
                let message = error.message(self);
                match message.location {
                    CompileErrorLocation::Generic|CompileErrorLocation::SourceOnly{..} => println!("{}", message.message),
                    CompileErrorLocation::SourceRange{range,..} => {
                        let range = parse_data.char_data().range(range);
                        println!("{}: {}", range, message.message)
                    },
                }
            }
        }

        self.with_source_mut(index, |source| {
            source.parse_data = Some(parse_data);
            source.checked_type = Some(checked_type);
        });
    }

    fn maybe_report_path_error(&self, path: &PathBuf, source: SourceIndex) {
        // Only report the "bad root directory" error once.
        let mut root_error = self.root_error.write().unwrap();
        if let Some(ref io_error) = *root_error {
            self.report(compile_errors::IoCurrentDirectoryError { source, path: path.clone(), io_error_string: io_error.to_string() });
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
                self.maybe_report_path_error(path, source);
                None
            }
        } else {
            None
        }
    }
}
