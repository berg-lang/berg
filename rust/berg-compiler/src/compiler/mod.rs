use std::cell::RefCell;
use std::rc::Rc;
use std::sync::RwLockWriteGuard;
use std::sync::RwLockReadGuard;
use source::compile_errors;
use source::compile_errors::CompileError;
use source::{Source, SourceIndex};
use source::source_spec::SourceSpec;
use util::indexed_vec::IndexedVec;
use interpreter::value::Value;
use std::default::Default;
use std::env;
use std::fmt::*;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;
use std::result;
use interpreter;

type Sources = IndexedVec<Rc<RefCell<Source>>, SourceIndex>;

pub struct Compiler {
    root: result::Result<PathBuf, String>,
    out: Box<Write>,
    err: Box<Write>,
    sources: RwLock<Sources>,
    generic_errors: RwLock<Vec<Box<CompileError>>>,
}

impl Debug for Compiler {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Foo")
            .field("root", &self.root)
            .field("sources", &self.sources)
            .finish()
    }
}

//
// Implementation
//

impl Compiler {
    pub fn from_env() -> Self {
        let root = match env::current_dir() {
            Ok(path) => Ok(path),
            Err(error) => Err(error.to_string()),
        };
        let out = Box::new(io::stdout());
        let err = Box::new(io::stderr());
        Self::new(root, out, err)
    }

    pub fn new(root: result::Result<PathBuf, String>, out: Box<Write>, err: Box<Write>) -> Self {
        let sources = RwLock::default();
        let generic_errors = RwLock::default();
        Compiler {
            root,
            out,
            err,
            sources,
            generic_errors,
        }
    }

    pub fn add_file_source<P: Into<PathBuf>>(&mut self, path: P) -> Value {
        let source = SourceSpec::file(path.into());
        self.add_source(source)
    }

    pub fn add_memory_source<T: Into<Vec<u8>>>(&mut self, name: String, contents: T) -> Value {
        let source = SourceSpec::memory(name, contents.into());
        self.add_source(source)
    }

    pub fn sources(&self) -> RwLockReadGuard<Sources> {
        self.sources.read().unwrap()
    }

    pub fn sources_mut(&self) -> RwLockWriteGuard<Sources> {
        self.sources.write().unwrap()
    }

    pub fn source(&self, index: SourceIndex) -> Rc<RefCell<Source>> {
        let sources = self.sources();
        Rc::clone(&sources[index])
    }

    pub fn generic_errors(&self) -> RwLockReadGuard<Vec<Box<CompileError>>> {
        self.generic_errors.read().unwrap()
    }

    pub fn generic_errors_mut(&self) -> RwLockWriteGuard<Vec<Box<CompileError>>> {
        self.generic_errors.write().unwrap()
    }

    fn report_generic_error<E: CompileError + 'static + Clone>(&self, error: E) -> Value {
        let mut errors = self.generic_errors.write().unwrap();
        let result = error.clone();
        errors.push(Box::new(error));
        result.into()
    }

    fn add_source(&self, source_spec: SourceSpec) -> Value {
        let index = {
            let mut sources = self.sources_mut();
            let next_index = sources.next_index();
            if usize::from(next_index) >= SourceIndex::MAX.into() {
                return self.report_generic_error(compile_errors::TooManySources {
                    num_sources: usize::from(next_index),
                });
            }
            sources.push(Rc::new(RefCell::new(Source::new(source_spec, next_index))))
        };
        let source = self.source(index);
        assert!(source.borrow().index == index);
        println!("{}", source.borrow().name().to_string_lossy());
        let value = source.borrow().value.clone();
        value.into()
    }

    pub fn run(&self, source_block: Value) -> Value {
        match source_block {
            Value::Block(source_block) => interpreter::run(self, &source_block),
            _ => unreachable!(),
        }
    }

    pub(crate) fn absolute_path(&self, path: &PathBuf) -> result::Result<PathBuf, String> {
        if path.is_relative() {
            match self.root {
                Ok(ref root) => Ok(root.join(path)),
                Err(ref error) => Err(error.clone()),
            }
        } else {
            Ok(path.clone())
        }
    }
}
