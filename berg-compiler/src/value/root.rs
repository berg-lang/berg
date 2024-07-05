use source::SourceRoot;

use crate::syntax::identifiers::keywords;
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::util::indexed_vec::to_indexed_cow;
use crate::{parser, value::*};

use std::borrow::Cow;
use std::fmt;
use std::io;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
pub struct RootRef(Rc<RootData>);

struct RootData {
    root: SourceRoot,
    #[allow(dead_code)]
    out: Box<dyn Write>,
    #[allow(dead_code)]
    err: Box<dyn Write>,
}

impl Default for RootRef {
    fn default() -> Self {
        RootRef::from_env()
    }
}

//
// Implementation
//
impl RootRef {
    pub fn new(root: SourceRoot, out: Box<dyn Write>, err: Box<dyn Write>) -> Self {
        RootRef(Rc::new(RootData { root, out, err }))
    }

    pub fn root(&self) -> &SourceRoot {
        &self.0.root
    }

    pub fn from_env() -> Self {
        let root_path = SourceRoot::from_env();
        let out = Box::new(io::stdout());
        let err = Box::new(io::stderr());
        RootRef::new(root_path, out, err)
    }

    pub fn parse_file(&self, path: impl Into<Cow<'static, Path>>) -> AstRef {
        let source = self.root().resolve(path.into());
        match source.load() {
            Ok(buffer) => AstRef::new(
                self.clone(),
                SourceSpec::File(source),
                parser::parse(buffer),
            ),
            Err(error) => AstRef::new_error(self.clone(), SourceSpec::File(source), error),
        }
    }

    pub fn parse_bytes(
        &self,
        name: impl Into<Cow<'static, str>>,
        buffer: impl Into<Cow<'static, [u8]>>,
    ) -> AstRef {
        let source = SourceSpec::Memory(name.into());
        let ast = parser::parse(to_indexed_cow(buffer.into()));
        AstRef::new(self.clone(), source, ast)
    }

    pub fn field_names(&self) -> impl ExactSizeIterator<Item = &IdentifierIndex> + fmt::Debug {
        keywords::FIELD_NAMES.iter()
    }

    fn field_index<'a>(&self, name: IdentifierIndex) -> Result<FieldIndex, EvalException<'a>> {
        match self.field_names().enumerate().find(|&(_, n)| name == *n) {
            Some((index, _)) => Ok(FieldIndex(index as u32)),
            None => CompilerError::NoSuchPublicFieldOnRoot(name).err(),
        }
    }

    pub fn field<'a>(&self, name: IdentifierIndex) -> EvalResult<'a> {
        self.local_field(self.field_index(name)?)
    }

    pub fn local_field<'a>(&self, index: FieldIndex) -> EvalResult<'a> {
        keyword_value(index)
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn set_local_field<'a>(
        &self,
        index: FieldIndex,
        _value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>> {
        CompilerError::ImmutableFieldOnRoot(index).err()
    }

    pub fn declare_field<'a>(&self, _index: FieldIndex) -> Result<(), EvalException<'a>> {
        // This should not be possible to do. We can fill in an error here when we find a testcase that triggers it.
        unreachable!()
    }
}

fn keyword_value<'a>(index: FieldIndex) -> EvalResult<'a> {
    use CompilerError::*;
    use EvalVal::*;
    match index {
        keywords::TRUE => true.ok(),
        keywords::FALSE => false.ok(),
        keywords::IF => If.ok(),
        keywords::ELSE => Else.ok(),
        keywords::WHILE => While.ok(),
        keywords::FOREACH => Foreach.ok(),
        keywords::BREAK => BreakOutsideLoop.err(),
        keywords::CONTINUE => ContinueOutsideLoop.err(),
        keywords::TRY => Try.ok(),
        keywords::CATCH => Catch.ok(),
        keywords::FINALLY => Finally.ok(),
        keywords::THROW => Throw.ok(),
        _ => unreachable!(),
    }
}

impl fmt::Debug for RootRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Root").field("root", &self.root()).finish()
    }
}
