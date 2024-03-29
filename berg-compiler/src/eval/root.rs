use string_interner::backend::StringBackend;
use string_interner::StringInterner;

use crate::syntax::{identifiers, FieldIndex, IdentifierIndex};
use crate::value::*;

use std::env;
use std::fmt;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub struct RootRef(Rc<RootData>);

struct RootData {
    root_path: io::Result<PathBuf>,
    #[allow(dead_code)]
    out: Box<dyn Write>,
    #[allow(dead_code)]
    err: Box<dyn Write>,
}

///
/// Keywords are fields in the root. When the identifier `true` is in the code,
/// it's treated as a normal variable reference and looked up in scope (which
/// includes the root scope).
///
#[allow(clippy::upper_case_acronyms)]
pub mod keywords {
    fields! { TRUE, FALSE, IF, ELSE, WHILE, FOREACH, BREAK, CONTINUE, TRY, CATCH, FINALLY, THROW, }
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
    pub fn new(root_path: io::Result<PathBuf>, out: Box<dyn Write>, err: Box<dyn Write>) -> Self {
        RootRef(Rc::new(RootData {
            root_path,
            out,
            err,
        }))
    }

    pub fn root_path(&self) -> &io::Result<PathBuf> {
        &self.0.root_path
    }

    pub fn from_env() -> Self {
        let root_path = env::current_dir();
        let out = Box::new(io::stdout());
        let err = Box::new(io::stderr());
        RootRef::new(root_path, out, err)
    }

    pub fn identifiers(&self) -> StringInterner<StringBackend<IdentifierIndex>> {
        identifiers::intern_all()
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
        use crate::eval::keywords::*;
        use CompilerError::*;
        use EvalVal::*;
        match index {
            TRUE => true.ok(),
            FALSE => false.ok(),
            IF => If.ok(),
            ELSE => Else.ok(),
            WHILE => While.ok(),
            FOREACH => Foreach.ok(),
            BREAK => BreakOutsideLoop.err(),
            CONTINUE => ContinueOutsideLoop.err(),
            TRY => Try.ok(),
            CATCH => Catch.ok(),
            FINALLY => Finally.ok(),
            THROW => Throw.ok(),
            _ => unreachable!(),
        }
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

impl fmt::Debug for RootRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Root")
            .field("root_path", &self.root_path())
            .finish()
    }
}
