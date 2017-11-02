pub(crate) mod compile_errors;
pub(crate) mod line_column;
pub(crate) mod source_spec;
pub(crate) mod source_data;

use checker;
use compiler::compile_errors::SourceCompileErrors;
use indexed_vec::IndexedVec;
use parser;
use public::*;
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
    errors: RwLock<CompileErrors>,
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
        let sources = RwLock::new(Vec::default().into());
        let errors = Default::default();
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

    pub fn add_memory_source<Str: Into<String>, Buf: Into<Vec<u8>>>(
        &mut self,
        name: Str,
        contents: Buf,
    ) {
        let source = SourceSpec::memory(name.into(), contents.into());
        self.add_source(source)
    }

    pub fn with_sources<T, F: FnOnce(&IndexedVec<SourceData<'c>, SourceIndex>) -> T>(&self, f: F) -> T {
        let sources = self.sources.read().unwrap();
        f(&sources)
    }

    pub fn with_errors<T, F: FnOnce(&[CompileError]) -> T>(&self, f: F) -> T {
        let errors = self.errors.read().unwrap();
        f(errors.as_slice())
    }

    pub(crate) fn with_error_reporter<T, F: FnOnce(&mut SourceCompileErrors)->T>(
        &self,
        source: SourceIndex,
        f: F
    ) -> T {
        let mut source_errors = SourceCompileErrors::new(source);
        let result = f(&mut source_errors);
        self.with_errors_mut(|errors| errors.extend(source_errors.close()));
        result
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

    pub(crate) fn with_errors_mut<T, F: FnOnce(&mut CompileErrors) -> T>(
        &self,
        f: F,
    ) -> T {
        let mut errors = self.errors.write().unwrap();
        f(&mut errors)
    }

    fn add_source(&self, source_spec: SourceSpec) {
        let index = {
            let mut sources = self.sources.write().unwrap();
            if sources.len() + 1 > SourceIndex::MAX {
                self.with_errors_mut(|errors| errors.report_generic(CompileErrorType::TooManySources));
            }
            sources.push(SourceData::new(source_spec));
            sources.len() - 1
        };
        self.with_error_reporter(index, |errors| {
            let parse_data = self.with_source(index, |source| {
                println!("{}", source.name().to_string_lossy());
                parser::parse(self, errors, source.source_spec())
            });

            println!("--------------------");
            println!("PARSE RESULT:");
            print!("{}", parse_data);

            if !errors.is_empty() {
                println!();
                println!("CHECK ERRORS:");
                for error in errors.iter() {
                    println!("- {:?}", error);
                }
            }

            let checked_type = checker::check(&parse_data, errors);

            if !errors.is_empty() {
                println!();
                println!("CHECK ERRORS:");
                for error in errors.iter() {
                    println!("- {:?}", error);
                }
            }

            self.with_source_mut(index, |source| {
                source.parse_data = Some(parse_data);
                source.checked_type = Some(checked_type);
            });
        });
    }

    fn maybe_report_path_error(&self, errors: &mut SourceCompileErrors) {
        // Only report the "bad root directory" error once.
        let mut root_error = self.root_error.write().unwrap();
        if let Some(ref error) = *root_error {
            errors.report_io_source(CompileErrorType::IoCurrentDirectoryError, error);
        } else {
            return;
        }
        *root_error = None;
    }

    pub(crate) fn absolute_path(&self, path: &PathBuf, errors: &mut SourceCompileErrors) -> Option<PathBuf> {
        if path.is_relative() {
            if let Some(ref root) = self.root {
                Some(root.join(path))
            } else {
                self.maybe_report_path_error(errors);
                None
            }
        } else {
            None
        }
    }
}
