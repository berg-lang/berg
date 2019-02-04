use crate::eval::{ExpressionEvaluator, ScopeRef};
use crate::syntax::{AstRef, BlockIndex, Expression, FieldError, FieldIndex, IdentifierIndex, Token};
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::{BergError, BergResult, NextVal, BergVal, BergValue, ErrorCode, EvalResult, EvalError::Raw, TakeError};
use std::cell::RefCell;
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
    expression: Expression,
    state: BlockState<'a>,
    index: BlockIndex,
    fields: Vec<EvalResult<'a>>,
    parent: ScopeRef<'a>,
    input: EvalResult<'a, Option<BergVal<'a>>>,
}

#[derive(Debug)]
enum BlockState<'a> {
    Ready,
    Running,
    NextVal,
    Complete(BergResult<'a, Option<BergVal<'a>>>),
}

impl<'a> BlockRef<'a> {
    ///
    /// Create a new block that will run the given expression against the
    /// given scope and input.
    ///
    pub fn new(
        expression: Expression,
        index: BlockIndex,
        parent: ScopeRef<'a>,
        input: Option<BergVal<'a>>,
    ) -> Self {
        BlockRef(Rc::new(RefCell::new(BlockData {
            expression,
            index,
            state: BlockState::Ready,
            fields: Default::default(),
            parent,
            input: Ok(input)
        })))
    }

    pub fn create_child_block(&self, expression: Expression, index: BlockIndex) -> Self {
        Self::new(expression, index, ScopeRef::BlockRef(self.clone()), None)
    }

    pub fn apply(&self, input: impl BergValue<'a>) -> BergResult<'a> {
        let block = self.0.borrow();
        let input = input.into_val()?;
        Self::new(block.expression, block.index, block.parent.clone(), Some(input)).ok()
    }

    fn take_result(&self, replace_with: BlockState<'a>) -> BergResult<'a, Option<BergVal<'a>>> {
        self.ensure_evaluated()?;
        let mut block = self.0.borrow_mut();
        match block.state {
            BlockState::Running | BlockState::NextVal => BergError::CircularDependency.take_error(&self.ast(), block.expression),
            BlockState::Complete(_) => {
                if let BlockState::Complete(result) = mem::replace(&mut block.state, replace_with) {
                    result
                } else {
                    unreachable!()
                }
            },
            BlockState::Ready => unreachable!(),
        }
    }

    fn replace_result(&self, result: BergResult<'a, Option<BergVal<'a>>>) {
        let mut block = self.0.borrow_mut();
        match block.state {
            BlockState::NextVal => {
                block.state = BlockState::Complete(result);
            },
            _ => unreachable!("Block didn't stay in NextVal state by itself: {:?}", block.state),
        }
    }

    pub fn borrow_result<T, F: Fn(&BergVal<'a>) -> T>(&self, f: F) -> BergResult<'a, Option<T>> {
        self.ensure_evaluated()?;
        let block = self.0.borrow();
        match &block.state {
            BlockState::Running | BlockState::NextVal => BergError::CircularDependency.take_error(&self.ast(), block.expression),
            BlockState::Complete(Ok(Some(ref result))) => Ok(Some(f(result))),
            BlockState::Complete(Ok(None)) => Ok(None),
            BlockState::Complete(Err(error)) => Err(error.clone()),
            BlockState::Ready => unreachable!(),
        }
    }

    fn clone_result(&self) -> BergResult<'a> {
        match self.borrow_result(|value| value.clone())? {
            Some(value) => Ok(value),
            None => Ok(BergVal::Nothing),
        }
    }

    fn ensure_evaluated(&self) -> BergResult<'a, ()> {
        // Check if the block has already been run (and don't re-run)
        let ast = self.ast();
        let expression = {
            let mut block = self.0.borrow_mut();
            match block.state {
                BlockState::Running | BlockState::NextVal => return BergError::CircularDependency.take_error(&ast, block.expression),
                BlockState::Complete(_) => return Ok(()),
                BlockState::Ready => {}
            }
            block.state = BlockState::Running;
            block.expression
        };
        // Run the block and stash the result
        let scope = ScopeRef::BlockRef(self.clone());
        let expression = ExpressionEvaluator::new(&scope, &ast, expression);
        let result = match expression.token() {
            Token::MissingExpression => Ok(None),
            _ => match expression.into_val() {
                Ok(value) => Ok(Some(value)),
                Err(error) => Err(error),
            },
        };
        let mut block = self.0.borrow_mut();
        block.state = BlockState::Complete(result);
        Ok(())
    }

    pub fn local_field(&self, index: FieldIndex, ast: &AstRef) -> EvalResult<'a> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
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

    pub fn declare_field(&self, field_index: FieldIndex, ast: &AstRef) -> EvalResult<'a, ()> {
        // Make sure we have enough spots to put the field
        let (input, block_field_index) = {
            let mut block = self.0.borrow_mut();
            let scope_start = ast.blocks()[block.index].scope_start;
            // The index we intend to insert this at
            let block_field_index: usize = (field_index - scope_start).into();
            while block_field_index >= block.fields.len() {
                let no_such_field = BergError::NoSuchField(scope_start+block.fields.len());
                block.fields.push(no_such_field.err());
            }
            if let Err(Raw(BergError::NoSuchField(..))) = block.fields[block_field_index] {
                // The only known way to declare a field in an object (right now) is to set it while
                // running.
                assert!(match block.state { BlockState::Running => true, _ => false });
                // Steal the input value so we can next_val() it without fear.
                if let Err(ref error) = block.input {
                    return Err(error.clone());
                }
                let input = mem::replace(&mut block.input, Err(Raw(BergError::CircularDependency)));
                (input.unwrap(), block_field_index)
            } else {
                return Ok(())
            }
        };

        // Move the value forward here, outside the lock, so we don't panic
        // (we will instead see CircularReference errors).
        let (head, tail) = match input {
            None => (None, None),
            Some(input) => input.next_val()?.into_head_tail(),
        };

        // Put input back, and set the field to the value we got!
        {
            let mut block = self.0.borrow_mut();
            block.fields[block_field_index] = match head {
                None => BergError::FieldNotSet(field_index).err(),
                Some(value) => Ok(value),
            };
            block.input = Ok(tail);
        }
        Ok(())
    }

    pub fn set_local_field(
        &self,
        field_index: FieldIndex,
        value: BergResult<'a>,
        ast: &AstRef,
    ) -> EvalResult<'a, ()> {
        let scope_start = ast.blocks()[self.0.borrow().index].scope_start;
        if field_index < scope_start {
            return self.0.borrow().parent.set_local_field(field_index, value, ast);
        }
        println!("Set {} to {}", ast.identifier_string(ast.fields()[field_index].name), match &value { Ok(v)=>format!("{}",v),Err(v)=>format!("{}",v)});
        {
            let mut block = self.0.borrow_mut();
            let index: usize = (field_index - scope_start).into();
            while index >= block.fields.len() {
                block.fields.push(BergError::NoSuchField(field_index).err());
            }
            block.fields[index] = match value {
                Ok(value) => Ok(value),
                Err(error) => Err(error.into()),
            };
        }
        println!("Now we are {}", self);
        Ok(())
    }

    pub fn bring_local_field_into_scope(
        &self,
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
        if let Err(Raw(BergError::NoSuchField(_))) = block.fields[index] {
            block.fields[index] = BergError::FieldNotSet(field_index).err()
        }
        Ok(())
    }

    pub fn ast(&self) -> AstRef<'a> {
        self.0.borrow().parent.ast().clone()
    }

    pub fn field_error<T>(&self, error: FieldError) -> EvalResult<'a, T> {
        use FieldError::*;
        match error {
            NoSuchPublicField(index) => BergError::NoSuchPublicField(self.clone(), index).err(),
            PrivateField(index) => BergError::PrivateField(self.clone(), index).err(),
        }
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
    fn next_val(self) -> BergResult<'a, NextVal<'a>> {
        // Get the current result, and prevent anyone else from retrieving this value while we change it
        // by marking it as "running"
        match self.take_result(BlockState::NextVal)? {
            None => Ok(NextVal::none()),
            Some(value) => match value.next_val() {
                Ok(NextVal(None)) => {
                    self.replace_result(Ok(None));
                    Ok(NextVal::none())
                }
                Ok(NextVal(Some((head, None)))) => {
                    self.replace_result(Ok(None));
                    Ok(NextVal::single(head))
                }
                // If there's a tail, take the tail and return ourselves so the our properties will still show up
                Ok(NextVal(Some((head, tail)))) => {
                    self.replace_result(Ok(tail));
                    Ok(NextVal::head_tail(head, self.into()))
                }
                Err(error) => Err(error)
            }
        }
    }

    fn into_val(self) -> BergResult<'a> {
        Ok(self.into())
    }

    fn into_native<T: TypeName + TryFrom<BergVal<'a>>> (
        self
    ) -> BergResult<'a, EvalResult<'a, T>> where <T as TryFrom<BergVal<'a>>>::Error: Into<BergVal<'a>> {
        println!("into_native({})", self);
        self.clone_result()?.into_native()
    }

    fn infix<T: BergValue<'a>>(
        self,
        operator: IdentifierIndex,
        right: T
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::*;

        match operator {
            DOT => self.field(right.into_native::<IdentifierIndex>()??),
            APPLY => Ok(self.apply(right)?),
            _ => self.clone_result()?.infix(operator, right)
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        // Closures report their own internal error instead of local ones.
        self.clone_result()?.prefix(operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        self.clone_result()?.postfix(operator)
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        println!("====> get {} on {}", self.ast().identifier_string(name), self);
        // Always try to get the field from the inner result first
        let current = self.clone_result();
        println!("got from {}", self);
        let current = current?;
        println!("current = {}", current);
        match current.field(name) {
            Err(Raw(ref error)) if error.code() == ErrorCode::NoSuchPublicField => {
                println!("====> no such public {} on {}", self.ast().identifier_string(name), self);
                let ast = self.ast();
                // If the inner result doesn't have it, get our own local field
                let index = {
                    let block = self.0.borrow();
                    let ast_block = &ast.blocks()[block.index];
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

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        // Figure out the field index from its name
        let ast = self.ast();
        let index = {
            let block = self.0.borrow();
            let ast_block = &ast.blocks()[block.index];
            ast_block
                .public_field_index(block.index, name, &ast)
                .or_else(|error| self.field_error(error))?
        };

        // Set the field.
        self.set_local_field(index, value, &ast)
    }
}

impl<'a> fmt::Display for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let block = self.0.borrow();
        let ast = self.ast();
        write!(f, "block({{ {} }}, {}, fields: {{", block.expression.to_string(&ast), block.state)?;
        let mut is_first_field = true;
        let scope_start = ast.blocks()[block.index].scope_start;
        for (index, field_value) in block.fields.iter().enumerate() {
            if is_first_field {
                is_first_field = false;
            } else {
                write!(f, ", ")?;
            }

            let field = &ast.fields()[scope_start + index];
            let name = &ast.identifiers()[field.name];

            match field_value {
                Ok(value) => write!(f, "{}: {}", name, value)?,
                Err(Raw(BergError::NoSuchField(..))) => {}
                Err(error) => write!(f, "{}: {}", name, error)?,
            }
        }
        write!(f, "}})")
    }
}

impl<'a> fmt::Display for BlockState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockState::Complete(Err(error)) => write!(f, "Complete({})", error),
            BlockState::Complete(Ok(Some(value))) => write!(f, "Complete({})", value),
            BlockState::Complete(Ok(None)) => write!(f, "Complete(<nothing>)"),
            BlockState::Ready => write!(f, "Ready", ),
            BlockState::Running => write!(f, "Running"),
            BlockState::NextVal => write!(f, "NextVal"),
        }
    }
}

impl<'a> fmt::Debug for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let block = self.0.borrow();
        let ast = self.ast();
        write!(f, "BlockRef {{ state: {:?}, expression: {}, fields: {{", block.state, block.expression.to_string(&ast))?;
        let mut is_first_field = true;
        let scope_start = ast.blocks()[block.index].scope_start;
        for (index, field_value) in block.fields.iter().enumerate() {
            if is_first_field {
                is_first_field = false;
            } else {
                write!(f, ", ")?;
            }

            let field = &ast.fields()[scope_start + index];
            let name = &ast.identifiers()[field.name];

            match field_value {
                Ok(value) => write!(f, "{}: {}", name, value)?,
                Err(Raw(BergError::NoSuchField(..))) => {}
                Err(error) => write!(f, "{}: {}", name, error)?,
            }
        }
        write!(f, "}}, parent: {:?}", block.parent)?;
        match &block.input {
            Ok(Some(value)) => write!(f, ", input: {} }}", value),
            Ok(None) => write!(f, ", input: <nothing> }}"),
            Err(error) => write!(f, ", input: {:?} }}", error),
        }
    }
}

impl<'a> PartialEq for BlockRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
