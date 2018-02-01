// Evaluates a given source expression.
use syntax::{AstRef, BlockIndex, FieldIndex};
use eval::ScopeRef;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use value::{BergError, BergResult};

#[derive(Clone)]
pub struct BlockRef<'a>(Rc<RefCell<BlockData<'a>>>);

pub struct BlockData<'a> {
    index: BlockIndex,
    fields: Vec<Option<BergResult<'a>>>,
    parent: ScopeRef<'a>,
}

impl<'a> BlockRef<'a> {
    pub fn new(index: BlockIndex, parent: ScopeRef<'a>) -> Self {
        BlockRef(Rc::new(RefCell::new(BlockData {
            index,
            fields: Default::default(),
            parent,
        })))
    }

    pub fn create_child_block(&self, index: BlockIndex) -> Self {
        Self::new(index, ScopeRef::BlockRef(self.clone()))
    }

    pub fn field(&self, index: FieldIndex, ast: &AstRef) -> BergResult<'a, BergResult<'a>> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            match self.0.borrow().fields.get(scope_index) {
                Some(&Some(ref result)) => Ok(result.clone()),
                Some(&None) => BergError::FieldNotSet(index).err(),
                None => BergError::NoSuchField(index).err(),
            }
        } else {
            self.0.borrow().parent.field(index, ast)
        }
    }

    pub fn declare_field(&mut self, index: FieldIndex, ast: &AstRef) -> BergResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            let mut block = self.0.borrow_mut();
            while scope_index >= block.fields.len() {
                block.fields.push(None);
            }
        }
        Ok(())
    }

    pub fn set_field(
        &mut self,
        index: FieldIndex,
        value: BergResult<'a>,
        ast: &AstRef,
    ) -> BergResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            let mut block = self.0.borrow_mut();
            while scope_index >= block.fields.len() {
                block.fields.push(None);
            }
            block.fields[scope_index] = Some(value);
            Ok(())
        } else {
            self.0.borrow_mut().parent.set_field(index, value, ast)
        }
    }

    pub fn ast(&self) -> AstRef<'a> {
        self.0.borrow().parent.ast().clone()
    }
}

impl<'a> fmt::Debug for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "BlockRef {{ fields: {{ ")?;
        let block = self.0.borrow();
        let ast = self.ast();
        let scope_start = ast.blocks()[block.index].scope_start;
        for (index, field) in block.fields.iter().enumerate() {
            let name = ast.fields()[scope_start + index].name;
            write!(f, "{}: ", &ast.identifiers()[name])?;
            match *field {
                None => write!(f, "None")?,
                Some(Err(ref error)) => {
                    write!(f, "Err(")?;
                    error.fmt_debug_shallow(f)?;
                    write!(f, ")")?;
                }
                Some(Ok(ref value)) => value.fmt_debug_shallow(f)?,
            }
        }
        write!(f, "}}, parent: {:?} }}", block.parent)
    }
}

impl<'a> PartialEq for BlockRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
