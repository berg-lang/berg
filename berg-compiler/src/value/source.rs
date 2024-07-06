use berg_util::to_indexed_cow;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;
use std::{borrow::Cow, rc::Rc};
use std::{env, io};

use berg_parser::{Ast, ByteIndex, ByteSlice};

use super::compiler_error::SourceLoadError;
use super::RootRef;

#[derive(Debug)]
pub struct SourceRoot(Result<Cow<'static, Path>, SourceLoadError>);

///
/// Specification of where we got source from.
///
#[derive(Debug, Clone)]
pub enum SourceSpec {
    File(SourceFileSpec),
    Memory(Cow<'static, str>),
}

#[derive(Debug, Clone)]
pub struct SourceFileSpec {
    user_path: Cow<'static, Path>,
    resolved_path: Result<Cow<'static, Path>, SourceLoadError>,
}

#[derive(Debug, Clone)]
pub struct AstRef(Rc<AstData>);

#[derive(Debug)]
pub struct AstData {
    pub root: RootRef,
    pub source: SourceSpec,
    pub ast: Ast,
    pub error: Option<SourceLoadError>,
}

impl SourceRoot {
    pub fn new(root_path: Cow<'static, Path>) -> Self {
        Self(Ok(root_path))
    }

    pub fn new_error(error: io::Error) -> Self {
        Self(Err(SourceLoadError::CurrentDirectoryError(Rc::new(error))))
    }

    pub fn from_env() -> Self {
        let result = match env::current_dir() {
            Ok(current_dir) => Ok(current_dir.into()),
            Err(io_error) => Err(SourceLoadError::CurrentDirectoryError(Rc::new(io_error))),
        };
        Self(result)
    }

    pub fn resolve(&self, user_path: Cow<'static, Path>) -> SourceFileSpec {
        let resolved_path = if user_path.is_relative() {
            match &self.0 {
                Ok(root_path) => Ok(root_path.join(&user_path).into()),
                Err(error) => Err(error.clone()),
            }
        } else {
            Ok(user_path.clone())
        };
        SourceFileSpec {
            user_path,
            resolved_path,
        }
    }
}

impl SourceSpec {
    pub fn name(&self) -> Cow<str> {
        match self {
            SourceSpec::File(source) => source.name(),
            SourceSpec::Memory(name) => name.clone(),
        }
    }

    pub fn parse_memory(
        root: RootRef,
        name: Cow<'static, str>,
        buffer: Cow<'static, [u8]>,
    ) -> AstRef {
        let source = SourceSpec::Memory(name);
        let ast = berg_parser::parse(to_indexed_cow(buffer));
        AstRef::new(root, source, ast)
    }
}

impl SourceFileSpec {
    pub fn name(&self) -> Cow<str> {
        self.user_path.to_string_lossy()
    }

    pub fn path(&self) -> &Result<Cow<'static, Path>, SourceLoadError> {
        &self.resolved_path
    }

    pub fn load(&self) -> Result<Cow<'static, ByteSlice>, SourceLoadError> {
        use SourceLoadError::*;

        let path = match &self.path() {
            Ok(path) => path,
            Err(error) => return Err(error.clone()),
        };

        // Open the file
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(io_error) => {
                return Err(match io_error.kind() {
                    io::ErrorKind::NotFound => SourceNotFound(Rc::new(io_error)),
                    _ => IoOpenError(Rc::new(io_error)),
                })
            }
        };

        // Read the file
        let mut buffer = Vec::new();
        if let Err(io_error) = file.read_to_end(&mut buffer) {
            return Err(IoReadError(Rc::new(io_error)));
        }

        // Check for too large source files
        if ByteIndex::MAX <= buffer.len() {
            return Err(SourceTooLarge(buffer.len()));
        }

        Ok(to_indexed_cow(buffer.into()))
    }
}

impl AstRef {
    pub fn new(root: RootRef, source: SourceSpec, ast: Ast) -> Self {
        Self(Rc::new(AstData {
            root,
            source,
            ast,
            error: None,
        }))
    }

    pub fn new_error(root: RootRef, source: SourceSpec, error: SourceLoadError) -> Self {
        Self(Rc::new(AstData {
            root,
            source,
            ast: Ast::default(),
            error: Some(error),
        }))
    }
}

impl Deref for AstRef {
    type Target = AstData;
    fn deref(&self) -> &AstData {
        &self.0
    }
}

impl Deref for AstData {
    type Target = Ast;
    fn deref(&self) -> &Ast {
        &self.ast
    }
}
