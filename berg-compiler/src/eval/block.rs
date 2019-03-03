use crate::eval::{ExpressionEvaluator, ScopeRef};
use crate::syntax::{
    Ast, ExpressionTreeWalker, AstIndex, AstRef, BlockIndex, ExpressionRef, FieldError, FieldIndex, IdentifierIndex,
};
use crate::value::implement::*;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::mem;
use std::rc::Rc;

///
/// A block represents the execution of an expression, including the next
/// expression to execute, as well as the scope (field values and parent block)
/// and input (a BergResult).
///
#[derive(Clone)]
pub struct BlockRef<'a>(Rc<RefCell<BlockData<'a>>>);

struct BlockData<'a> {
    expression: AstIndex,
    state: BlockState<'a>,
    index: BlockIndex,
    fields: Vec<BergResult<'a>>,
    parent: ScopeRef<'a>,
    input: BergResult<'a>,
}

#[derive(Debug)]
enum BlockState<'a> {
    Ready,
    Running,
    NextVal,
    Complete(BergResult<'a>),
}

impl<'a> BlockRef<'a> {
    ///
    /// Create a new block that will run the given expression against the
    /// given scope and input.
    ///
    pub fn new(
        expression: AstIndex,
        index: BlockIndex,
        parent: ScopeRef<'a>,
        input: BergResult<'a>,
    ) -> Self {
        BlockRef(Rc::new(RefCell::new(BlockData {
            expression,
            index,
            state: BlockState::Ready,
            fields: Default::default(),
            parent,
            input,
        })))
    }

    pub fn create_child_block(&self, expression: AstIndex, index: BlockIndex) -> Self {
        Self::new(expression, index, ScopeRef::BlockRef(self.clone()), Ok(BergVal::empty_tuple()))
    }

    pub fn apply(&self, input: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        let block = self.0.borrow();
        let input = input.into_val();
        // Evaluate immediately and take the result.
        let new_block = Self::new(
            block.expression,
            block.index,
            block.parent.clone(),
            input,
        );
        new_block.take_result(BlockState::Complete(Ok(BergVal::empty_tuple())))
    }

    fn take_result(&self, replace_with: BlockState<'a>) -> BergResult<'a> {
        self.ensure_evaluated()?;
        let mut block = self.0.borrow_mut();
        match block.state {
            BlockState::Running | BlockState::NextVal => {
                block.delocalize_error(BergError::CircularDependency)
            }
            BlockState::Complete(_) => {
                if let BlockState::Complete(result) = mem::replace(&mut block.state, replace_with) {
                    result
                } else {
                    unreachable!()
                }
            }
            BlockState::Ready => unreachable!(),
        }
    }

    fn replace_result(&self, result: BergResult<'a>) {
        let mut block = self.0.borrow_mut();
        match block.state {
            BlockState::NextVal => {
                block.state = BlockState::Complete(result);
            }
            _ => unreachable!(
                "Block didn't stay in NextVal state by itself: {:?}",
                block.state
            ),
        }
    }

    fn clone_result(&self) -> BergResult<'a> {
        self.ensure_evaluated()?;
        let block = self.0.borrow();
        match &block.state {
            BlockState::Running | BlockState::NextVal => block.delocalize_error(BergError::CircularDependency),
            BlockState::Complete(ref result) => result.clone(),
            BlockState::Ready => unreachable!(),
        }
    }

    fn ensure_evaluated(&self) -> BergResult<'a, ()> {
        // Check if the block has already been run (and don't re-run)
        let (ast, expression, index) = {
            let mut block = self.0.borrow_mut();
            match block.state {
                BlockState::Running | BlockState::NextVal => {
                    return block.delocalize_error(BergError::CircularDependency);
                }
                BlockState::Complete(_) => return Ok(()),
                BlockState::Ready => {}
            }
            block.state = BlockState::Running;
            (block.ast(), block.expression, block.index)
        };
        // Run the block and stash the result
        let scope = ScopeRef::BlockRef(self.clone());
        let expression = ExpressionEvaluator::new(&scope, &ast, expression);
        println!("Evaluating block {}", self);
        let result = expression.evaluate_inner(ast.blocks[index].boundary);
        println!("result: {}", result.display());
        let mut block = self.0.borrow_mut();
        block.state = BlockState::Complete(result);
        Ok(())
    }

    pub fn local_field(&self, index: FieldIndex, ast: &Ast) -> BergResult<'a> {
        let scope_start = ast.blocks[self.0.borrow().index].scope_start;
        let block = self.0.borrow();
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            match block.fields.get(scope_index) {
                Some(result) => result.clone(),
                None => BergError::NoSuchField(index).err(),
            }
        } else {
            block.parent.local_field(index, ast)
        }
    }

    pub fn declare_field(&self, field_index: FieldIndex, ast: &Ast) -> BergResult<'a, ()> {
        // Make sure we have enough spots to put the field
        let (input, block_field_index) = {
            let mut block = self.0.borrow_mut();
            let scope_start = ast.blocks[block.index].scope_start;
            // The index we intend to insert this at
            let block_field_index: usize = (field_index - scope_start).into();
            while block_field_index >= block.fields.len() {
                let no_such_field = BergError::NoSuchField(scope_start + block.fields.len());
                block.fields.push(no_such_field.err());
            }
            if let Err(ControlVal::ExpressionError(BergError::NoSuchField(..), ExpressionErrorPosition::Expression)) = block.fields[block_field_index] {
                // The only known way to declare a field in an object (right now) is to set it while
                // running.
                assert!(match block.state {
                    BlockState::Running => true,
                    _ => false,
                });
                // Steal the input value so we can next_val() it without fear.
                if let Err(ref error) = block.input {
                    return Err(error.clone());
                }
                let input = mem::replace(&mut block.input, BergError::CircularDependency.err());
                (input.unwrap(), block_field_index)
            } else {
                return Ok(());
            }
        };

        // Move the value forward here, outside the lock, so we don't panic
        // (we will instead see CircularReference errors).
        let next_val = input.next_val()?;

        // Put input back, and set the field to the value we got!
        let mut block = self.0.borrow_mut();
        block.fields[block_field_index] = match next_val {
            None => BergError::FieldNotSet(field_index).err(),
            Some(NextVal { head, tail }) => {
                block.input = tail;
                head
            }
        };
        Ok(())
    }

    pub fn set_local_field(
        &self,
        field_index: FieldIndex,
        value: BergResult<'a>,
        ast: &Ast,
    ) -> BergResult<'a, ()> {
        let scope_start = ast.blocks[self.0.borrow().index].scope_start;
        if field_index < scope_start {
            return self
                .0
                .borrow()
                .parent
                .set_local_field(field_index, value, ast);
        }
        println!(
            "Set {} to {:?}",
            ast.identifier_string(ast.fields[field_index].name),
            value,
        );
        {
            let mut block = self.0.borrow_mut();
            let index: usize = (field_index - scope_start).into();
            while index >= block.fields.len() {
                block.fields.push(BergError::NoSuchField(field_index).err());
            }
            block.fields[index] = value;
        }
        println!("Now we are {}", self);
        Ok(())
    }

    pub fn bring_local_field_into_scope(
        &self,
        field_index: FieldIndex,
        ast: &Ast,
    ) -> BergResult<'a, ()> {
        let scope_start = ast.blocks[self.0.borrow().index].scope_start;
        let mut block = self.0.borrow_mut();
        if field_index < scope_start {
            return Ok(());
        }

        let index: usize = (field_index - scope_start).into();
        while index >= block.fields.len() {
            block.fields.push(BergError::NoSuchField(field_index).err());
        }
        if let Err(ControlVal::ExpressionError(BergError::NoSuchField(_), ExpressionErrorPosition::Expression)) = block.fields[index] {
            block.fields[index] = BergError::FieldNotSet(field_index).err()
        }
        Ok(())
    }

    pub fn ast(&self) -> AstRef<'a> {
        self.0.borrow().ast()
    }

    pub fn field_error<T>(&self, error: FieldError) -> BergResult<'a, T> {
        use FieldError::*;
        match error {
            NoSuchPublicField(index) => BergError::NoSuchPublicField(self.clone(), index).err(),
            PrivateField(index) => BergError::PrivateField(self.clone(), index).err(),
        }
    }
}

impl<'a> From<&BlockData<'a>> for ExpressionRef<'a> {
    fn from(from: &BlockData<'a>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}
impl<'p, 'a: 'p> From<Ref<'p, BlockData<'a>>> for ExpressionRef<'a> {
    fn from(from: Ref<'p, BlockData<'a>>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}
impl<'p, 'a: 'p> From<RefMut<'p, BlockData<'a>>> for ExpressionRef<'a> {
    fn from(from: RefMut<'p, BlockData<'a>>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}

impl<'a> From<BlockRef<'a>> for BergVal<'a> {
    fn from(from: BlockRef<'a>) -> Self {
        BergVal::BlockRef(from)
    }
}

impl<'a> BergValue<'a> for BlockRef<'a> {
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        // Get the current result, and prevent anyone else from retrieving this value while we change it
        // by marking state as NextVal
        let next_val = self.take_result(BlockState::NextVal).next_val()?;
        match next_val {
            None => Ok(None),
            Some(NextVal { head, tail }) => {
                self.replace_result(tail);
                Ok(Some(NextVal { head, tail: self.into_val() }))
            }
        }
    }

    fn into_val(self) -> BergResult<'a> {
        Ok(self.into())
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.clone_result().into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.clone_result().try_into_native()
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use crate::syntax::identifiers::*;

        match operator {
            DOT => default_infix(self, operator, right),
            APPLY => self.apply(right),
            // Blocks do not evaluate when used as statements.
            SEMICOLON | NEWLINE => right.into_val(),
            _ => self.clone_result().infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        self.clone_result().infix_assign(operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        // Closures report their own internal error instead of local ones.
        self.clone_result().prefix(operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.clone_result().postfix(operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        self.clone_result().subexpression_result(boundary)
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        println!(
            "====> get {} on {}",
            self.ast().identifier_string(name),
            self
        );
        // Always try to get the field from the inner result first
        let current = self.clone_result();
        println!("got from {}", self);
        let current = current?;
        println!("current = {}", current);
        match current.field(name) {
            Err(ControlVal::ExpressionError(ref error, ExpressionErrorPosition::Expression)) if error.code() == ErrorCode::NoSuchPublicField => {
                println!("====> no such public {} on {}", self.ast().identifier_string(name), self);
                let ast = self.ast();
                // If the inner result doesn't have it, get our own local field
                let index = {
                    let block = self.0.borrow();
                    let ast_block = &ast.blocks[block.index];
                    ast_block
                        .public_field_index(block.index, name, &ast)
                        .or_else(|error| self.field_error(error))?
                };
                self.local_field(index, &ast)
            }
            Ok(value) => { println!("====> got {} for {} on {}", value, self.ast().identifier_string(name), self); Ok(value) }
            Err(error) => { println!("====> error {} for {} on {}", error, self.ast().identifier_string(name), self); Err(error) }
            // result => result,
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        // Figure out the field index from its name
        let ast = self.ast();
        let index = {
            let block = self.0.borrow();
            let ast_block = &ast.blocks[block.index];
            ast_block
                .public_field_index(block.index, name, &ast)
                .or_else(|error| self.field_error(error))?
        };

        // Set the field.
        self.set_local_field(index, value, &ast)
    }
}

impl<'a> BlockData<'a> {
    pub fn ast(&self) -> AstRef<'a> {
        self.parent.ast()
    }
    pub fn delocalize_error<T>(&self, error: BergError<'a>) -> BergResult<'a, T> {
        Err(ControlVal::Error(error.at_location(ExpressionRef::new(self.ast(), self.expression))))
    }
}

impl<'a> fmt::Display for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let block = self.0.borrow();
        let ast = self.ast();
        write!(f, "block({}", block.state)?;
        match &block.input {
            Ok(BergVal::Tuple(tuple)) if tuple.is_empty() => {},
            input => write!(f, ", input: {}", input.display())?,
        }
        if !block.fields.is_empty() {
            write!(f, ", fields: {{")?;
            let mut is_first_field = true;
            let scope_start = ast.blocks[block.index].scope_start;
            for (index, field_value) in block.fields.iter().enumerate() {
                if is_first_field {
                    is_first_field = false;
                } else {
                    write!(f, ", ")?;
                }

                let field = &ast.fields[scope_start + index];
                let name = ast.identifier_string(field.name);

                match field_value {
                    Ok(value) => write!(f, "{}: {}", name, value)?,
                    Err(ControlVal::ExpressionError(BergError::NoSuchField(..), ExpressionErrorPosition::Expression)) => write!(f, "{}: <undeclared>", name)?,
                    Err(error) => write!(f, "{}: {}", name, error)?,
                }
            }
            write!(f, "}}")?;
        }
        write!(f, ", expression: {}", ExpressionTreeWalker::basic(&ast, block.expression))?;
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BlockState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockState::Complete(result) => write!(f, "Complete({})", result.display()),
            BlockState::Ready => write!(f, "Ready",),
            BlockState::Running => write!(f, "Running"),
            BlockState::NextVal => write!(f, "NextVal"),
        }
    }
}

impl<'a> fmt::Debug for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let block = self.0.borrow();
        let ast = self.ast();
        write!(f, "BlockRef {{ state: {:?}", block.state)?;
        match &block.input {
            Ok(BergVal::Tuple(tuple)) if tuple.is_empty() => {},
            input => write!(f, ", input: {}", input.display())?,
        }
        if !block.fields.is_empty() {
            write!(f, ", fields: {{")?;
            let mut is_first_field = true;
            let scope_start = ast.blocks[block.index].scope_start;
            for (index, field_value) in block.fields.iter().enumerate() {
                if is_first_field {
                    is_first_field = false;
                } else {
                    write!(f, ", ")?;
                }

                let field = &ast.fields[scope_start + index];
                let name = ast.identifier_string(field.name);

                match field_value {
                    Ok(value) => write!(f, "{}: {}", name, value)?,
                    Err(ControlVal::ExpressionError(BergError::NoSuchField(..), ExpressionErrorPosition::Expression)) => write!(f, "{}: <undeclared>", name)?,
                    Err(error) => write!(f, "{}: {}", name, error)?,
                }
            }
            write!(f, "}}")?;
        }
        write!(
            f,
            ", expression: {}, parent: {:?}",
            ExpressionTreeWalker::basic(&ast, block.expression), block.parent
        )?;
        write!(f, "}}")
    }
}

impl<'a> PartialEq for BlockRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
