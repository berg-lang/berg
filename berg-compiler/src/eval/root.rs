use crate::syntax::identifiers;
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::value::*;
use std;
use std::env;
use std::fmt;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use string_interner::StringInterner;

#[derive(Clone)]
pub struct RootRef(Rc<RootData>);

struct RootData {
    root_path: io::Result<PathBuf>,
    #[allow(dead_code)]
    out: Box<Write>,
    #[allow(dead_code)]
    err: Box<Write>,
}

pub mod root_fields {
    use crate::syntax::identifiers;
    use crate::syntax::{FieldIndex, IdentifierIndex};

    pub const TRUE: FieldIndex = FieldIndex(0);
    pub const FALSE: FieldIndex = FieldIndex(1);
    pub const NAMES: [IdentifierIndex; 2] = [identifiers::TRUE, identifiers::FALSE];
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
    pub fn new(root_path: io::Result<PathBuf>, out: Box<Write>, err: Box<Write>) -> Self {
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

    pub fn identifiers(&self) -> StringInterner<IdentifierIndex> {
        identifiers::intern_all()
    }

    pub fn field_names(&self) -> impl ExactSizeIterator<Item = &IdentifierIndex> + fmt::Debug {
        root_fields::NAMES.iter()
    }

    fn field_index<'a>(&self, name: IdentifierIndex) -> Result<FieldIndex, ErrorVal<'a>> {
        match self.field_names().enumerate().find(|&(_, n)| name == *n) {
            Some((index, _)) => Ok(FieldIndex(index as u32)),
            None => BergError::NoSuchPublicFieldOnRoot(name).err(),
        }
    }

    pub fn field<'a>(&self, name: IdentifierIndex) -> EvalResult<'a> {
        self.local_field(self.field_index(name)?)
    }

    pub fn local_field<'a>(&self, index: FieldIndex) -> EvalResult<'a> {
        use crate::eval::root_fields::*;
        match index {
            TRUE => true.ok(),
            FALSE => false.ok(),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn set_local_field<'a>(
        &self,
        index: FieldIndex,
        _value: BergVal<'a>,
    ) -> Result<(), ErrorVal<'a>> {
        BergError::ImmutableFieldOnRoot(index).err()
    }

    pub fn declare_field<'a>(&self, _index: FieldIndex) -> Result<(), ErrorVal<'a>> {
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
