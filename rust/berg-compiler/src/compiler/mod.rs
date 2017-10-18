pub(crate) mod compile_error;
pub(crate) mod source;
pub(crate) mod source_data;

use public::*;
use parser;
use checker;
use compiler::source_data::Sources;
use compiler::source_data::*;
use std::default::Default;

use std::env;
use std::fmt::*;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct Compiler<'c> {
    root: Option<PathBuf>,
    root_error: RwLock<Option<io::Error>>,
    out: Box<Write>,
    err: Box<Write>,

    sources: RwLock<Sources<'c>>,
    errors: RwLock<Vec<CompileError>>,
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
        let out = out.into();
        let err = err.into();
        let sources = RwLock::new(Default::default());
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

    pub fn with_sources<T, F: FnOnce(&Sources<'c>) -> T>(&self, f: F) -> T {
        let sources = self.sources.read().unwrap();
        f(&sources)
    }

    pub fn with_errors<T, F: FnOnce(&[CompileError]) -> T>(&self, f: F) -> T {
        let errors = self.errors.read().unwrap();
        f(errors.as_slice())
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
        let source_data = &mut sources[index];
        f(source_data)
    }

    pub(crate) fn report_at(
        &self,
        error_type: CompileErrorType,
        source: SourceIndex,
        start: ByteIndex,
        string: &str,
    ) {
        self.report(error_type.at(source, start, string))
    }

    pub(crate) fn report_source_only(&self, error_type: CompileErrorType, source: SourceIndex) {
        self.report(error_type.source_only(source))
    }

    pub(crate) fn report_generic(&self, error_type: CompileErrorType) {
        self.report(error_type.generic())
    }

    pub(crate) fn report_invalid_bytes(
        &self,
        error_type: CompileErrorType,
        source: SourceIndex,
        start: ByteIndex,
        bytes: &[u8],
    ) {
        self.report(error_type.invalid_bytes(source, start, bytes))
    }

    pub(crate) fn report_io_read(
        &self,
        error_type: CompileErrorType,
        source: SourceIndex,
        start: ByteIndex,
        error: &io::Error,
    ) {
        self.report(error_type.io_read(source, start, error))
    }

    pub(crate) fn report_io_open(
        &self,
        error_type: CompileErrorType,
        source: SourceIndex,
        error: &io::Error,
        path: &Path,
    ) {
        self.report(error_type.io_open(source, error, path))
    }

    fn add_source(&self, source_spec: SourceSpec) {
        let index = {
            let mut sources = self.sources.write().unwrap();
            if sources.len() + 1 > SourceIndex::MAX {
                self.report_generic(TooManySources);
            }
            sources.push(SourceData::new(source_spec));
            SourceIndex::from(sources.len() - 1)
        };
        self.with_source_mut(index, |source| {
            source.parse_data = Some(parser::parse(self, index, source.source_spec()));
            source.checked_type = Some(checker::check(self, index, source));
        });

        self.with_source(index, |source| {
            println!("{}", source.name().to_string_lossy());
            println!("--------------------");
            println!("Result:");
            for token in TokenIndex(0)..source.num_tokens() {
                println!(
                    "- {}: {:?} \"{}\"",
                    source.token_range(token),
                    source.token(token).token_type,
                    source.token(token).string
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

    fn report(&self, error: CompileError) {
        let mut errors = self.errors.write().unwrap();
        errors.push(error)
    }
}
