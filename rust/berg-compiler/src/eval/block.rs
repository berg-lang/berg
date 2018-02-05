// Evaluates a given source expression.
use error::{BergError, EvalError, BergResult, Raw, EvalResult};
use eval::ScopeRef;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use syntax::{AstRef, BlockIndex, FieldIndex, IdentifierIndex};

#[derive(Clone)]
pub struct BlockRef<'a>(Rc<RefCell<BlockData<'a>>>);

pub struct BlockData<'a> {
    index: BlockIndex,
    fields: Vec<EvalResult<'a>>,
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

    pub fn field(&self, index: FieldIndex, ast: &AstRef) -> EvalResult<'a> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        let block = self.0.borrow();
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            match block.fields.get(scope_index) {
                Some(&ref result) => result.clone(),
                None => BergError::NoSuchField(index).err(),
            }
        } else {
            block.parent.field(index, ast)
        }
    }

    pub fn public_field_by_name(&self, name: IdentifierIndex, ast: &AstRef<'a>) -> EvalResult<'a> {
        let block = self.0.borrow();
        let ast_block = &ast.blocks()[block.index];
        let index = ast_block.public_field_index(block.index, name, ast).or_else(|error| Err(error.in_block(self)))?;
        self.field(index, ast)
    }

    pub fn declare_field(&mut self, field_index: FieldIndex, ast: &AstRef) -> EvalResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        let mut block = self.0.borrow_mut();
        let index: usize = (field_index - scope_start).into();
        while index >= block.fields.len() {
            block.fields.push(BergError::NoSuchField(field_index).err());
        }
        if let Err(Raw(BergError::NoSuchField(..))) = block.fields[index] {
            block.fields[index] = BergError::FieldNotSet(field_index).err();
        }
        Ok(())
    }

    pub fn set_field(
        &mut self,
        field_index: FieldIndex,
        value: BergResult<'a>,
        ast: &AstRef,
    ) -> EvalResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        let mut block = self.0.borrow_mut();
        if field_index < scope_start {
            return block.parent.set_field(field_index, value, ast);
        }

        let index: usize = (field_index - scope_start).into();
        while index >= block.fields.len() {
            block.fields.push(BergError::NoSuchField(field_index).err());
        }
        block.fields[index] = match value { Ok(value) => Ok(value), Err(error) => Err(error.into()) };
        Ok(())
    }

    pub fn ast(&self) -> AstRef<'a> {
        self.0.borrow().parent.ast().clone()
    }
}

impl<'a> fmt::Display for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "block{{")?;
        let block = self.0.borrow();
        let ast = self.ast();
        let scope_start = ast.blocks()[block.index].scope_start;
        for (index, field_value) in block.fields.iter().enumerate() {
            let field = &ast.fields()[scope_start + index];
            let name = &ast.identifiers()[field.name];

            match *field_value {
                Ok(ref value) => write!(f, " {}: {},", name, value)?,
                Err(Raw(BergError::NoSuchField(..))) => {},
                Err(Raw(BergError::FieldNotSet(..))) => write!(f, " {}: <uninitialized>,", name)?,
                Err(Raw(_)) => unreachable!(),
                Err(EvalError::Error(ref error)) => write!(f, " {}: {},", name, error)?,
            }
        }
        write!(f, " }}")
    }
}

impl<'a> fmt::Debug for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BlockRef {{ fields: {{ ")?;
        let block = self.0.borrow();
        let ast = self.ast();
        let scope_start = ast.blocks()[block.index].scope_start;
        for (index, field_value) in block.fields.iter().enumerate() {
            let field = &ast.fields()[scope_start + index];
            let name = &ast.identifiers()[field.name];

            match *field_value {
                Ok(ref value) => write!(f, "{}: {}", name, value)?,
                Err(Raw(BergError::NoSuchField(..))) => {},
                Err(Raw(BergError::FieldNotSet(..))) => write!(f, " {}: <uninitialized>,", name)?,
                Err(Raw(_)) => unreachable!(),
                Err(EvalError::Error(ref error)) => write!(f, " {}: {},", name, error)?,
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

