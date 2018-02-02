use eval::BergEval;
use std;
use std::env;
use std::fmt;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use syntax::identifiers;
use syntax::{FieldIndex, IdentifierIndex};
use util::intern_pool::InternPool;
use value::{BergError, BergResult, BergVal};

pub struct RootRef(Rc<RootData>);

struct RootData {
    root_path: io::Result<PathBuf>,
    out: Box<Write>,
    err: Box<Write>,
}

pub mod root_fields {
    use syntax::{FieldIndex, IdentifierIndex};
    use syntax::identifiers;

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

    pub fn identifiers(&self) -> InternPool<IdentifierIndex> {
        identifiers::intern_all()
    }

    pub fn field_names(&self) -> std::slice::Iter<'static, IdentifierIndex> {
        root_fields::NAMES.iter()
    }

    pub fn field<'a>(&self, index: FieldIndex) -> BergResult<'a, BergEval<'a>> {
        use eval::root_fields::*;
        match index {
            // Can't figure out another way to downgrade the static lifetime :(
            TRUE => Ok(BergVal::Boolean(true).into()),
            FALSE => Ok(BergVal::Boolean(false).into()),
            _ => unreachable!(),
        }
    }

    #[cfg_attr(feature = "clippy", allow(needless_pass_by_value))]
    pub fn set_field<'a>(&self, index: FieldIndex, _value: BergResult<'a, BergEval<'a>>) -> BergResult<'a, ()> {
        BergError::ImmutableField(index).err()
    }

    pub fn declare_field<'a>(&self, _index: FieldIndex) -> BergResult<'a, ()> {
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
