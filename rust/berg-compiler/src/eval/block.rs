// Evaluates a given source expression.
use util::try_from::TryFrom;
use error::{BergError, BergResult, EvalError, EvalResult, Raw, TakeError};
use eval::{Expression, Operand, ScopeRef};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use syntax::{AstRef, BlockIndex, FieldIndex, IdentifierIndex};
use value::{BergVal, BergValue};

#[derive(Clone)]
pub struct BlockRef<'a>(Rc<RefCell<BlockData<'a>>>);

pub struct BlockData<'a> {
    expression: Expression,
    state: BlockState<'a>,
    index: BlockIndex,
    fields: Vec<EvalResult<'a>>,
    parent: ScopeRef<'a>,
}

#[derive(Debug)]
enum BlockState<'a> {
    Ready,
    Running,
    Complete(Box<BergResult<'a>>),
}

impl<'a> BlockRef<'a> {
    pub fn new(expression: Expression, index: BlockIndex, parent: ScopeRef<'a>) -> Self {
        BlockRef(Rc::new(RefCell::new(BlockData {
            expression,
            index,
            state: BlockState::Ready,
            fields: Default::default(),
            parent,
        })))
    }

    pub fn create_child_block(&self, expression: Expression, index: BlockIndex) -> Self {
        Self::new(expression, index, ScopeRef::BlockRef(self.clone()))
    }

    pub fn evaluate_local(&self) -> BergResult<'a> {
        let ast = self.ast();
        let expression = {
            let mut block = self.0.borrow_mut();
            match block.state {
                BlockState::Running => return BergError::CircularDependency.take_error(&ast, block.expression),
                BlockState::Complete(ref result) => return result.as_ref().clone(),
                BlockState::Ready => {},
            }
            block.state = BlockState::Running;
            block.expression
        };
        let mut scope = ScopeRef::BlockRef(self.clone());
        let result = expression.evaluate_local(&mut scope, &ast);
        let mut block = self.0.borrow_mut();
        block.state = BlockState::Complete(Box::new(result));
        if let BlockState::Complete(ref result) = block.state {
            result.as_ref().clone()
        } else {
            unreachable!()
        }
    }

    pub fn local_field(&self, index: FieldIndex, ast: &AstRef) -> EvalResult<'a> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        let block = self.0.borrow();
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            match block.fields.get(scope_index) {
                Some(&ref result) => result.clone(),
                None => BergError::NoSuchField(index).err(),
            }
        } else {
            block.parent.local_field(index, ast)
        }
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

    pub fn set_local_field(
        &mut self,
        field_index: FieldIndex,
        value: BergResult<'a>,
        ast: &AstRef,
    ) -> EvalResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        let mut block = self.0.borrow_mut();
        if field_index < scope_start {
            return block.parent.set_local_field(field_index, value, ast);
        }

        let index: usize = (field_index - scope_start).into();
        while index >= block.fields.len() {
            block.fields.push(BergError::NoSuchField(field_index).err());
        }
        block.fields[index] = match value { Ok(value) => Ok(value), Err(error) => Err(error.into()) };
        Ok(())
    }

    pub fn bring_local_field_into_scope(
        &mut self,
        field_index: FieldIndex,
        ast: &AstRef,
    ) -> EvalResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        let mut block = self.0.borrow_mut();
        if field_index < scope_start {
            return Ok(());
        }

        let index: usize = (field_index - scope_start).into();
        while index >= block.fields.len() {
            block.fields.push(BergError::NoSuchField(field_index).err());
        }
        match block.fields[index] {
            Err(Raw(BergError::NoSuchField(_))) => block.fields[index] = BergError::FieldNotSet(field_index).err(),
            _ => {},
        }
        Ok(())
    }

    pub fn ast(&self) -> AstRef<'a> {
        self.0.borrow().parent.ast().clone()
    }
}

impl<'a> From<BlockRef<'a>> for BergVal<'a> {
    fn from(from: BlockRef<'a>) -> Self {
        BergVal::BlockRef(from)
    }
}

impl<'a> TryFrom<BergVal<'a>> for BlockRef<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::BlockRef(value) => Ok(value),
            _ => Err(from),
        }
    }
}

impl<'a> BergValue<'a> for BlockRef<'a> {
    fn evaluate(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        self.evaluate_local()?.evaluate(scope)
    }

    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        use syntax::identifiers::*;

        if operator == DOT {
            let identifier = right.evaluate_to::<IdentifierIndex>(scope, ast)?;
            self.field(identifier, scope)
        } else {
            self.evaluate_local()?.infix(operator, scope, right, ast)
        }
    }

    fn prefix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
    ) -> EvalResult<'a> {
        // Closures report their own internal error instead of local ones.
        self.evaluate_local()?.prefix(operator, scope)
    }

    fn postfix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate_local()?.prefix(operator, scope)
    }

    fn field(&self, name: IdentifierIndex, _scope: &mut ScopeRef<'a> ) -> EvalResult<'a> {
        // Figure out the field index from its name
        let ast = self.ast();
        let index = {
            let block = self.0.borrow();
            let ast_block = &ast.blocks()[block.index];
            ast_block.public_field_index(block.index, name, &ast).or_else(|error| Err(error.in_block(self)))?
        };

        // Get the field.
        match self.local_field(index, &ast) {
            // If the field isn't there yet, evaluate ourselves so that it'll get vivified, and set again.
            Err(Raw(BergError::NoSuchField(..))) => {
                self.evaluate_local()?;
                self.local_field(index, &ast)
            },
            result => result,
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>, _scope: &mut ScopeRef<'a> ) -> EvalResult<'a, ()> {
        println!("-----------------");
        println!("Setting field ...");
        // Figure out the field index from its name
        let ast = self.ast();
        let index = {
            let block = self.0.borrow();
            let ast_block = &ast.blocks()[block.index];
            ast_block.public_field_index(block.index, name, &ast).or_else(|error| Err(error.in_block(self)))?
        };

        // Set the field.
        self.set_local_field(index, value, &ast)
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

