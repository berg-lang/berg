use crate::eval::{ExpressionEvaluator, ScopeRef};
use crate::syntax::{
    Ast, AstIndex, AstRef, BlockIndex, ExpressionRef, ExpressionTreeWalker, FieldError, FieldIndex,
    IdentifierIndex,
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

#[derive(Debug)]
struct BlockData<'a> {
    expression: AstIndex,
    state: BlockState<'a>,
    index: BlockIndex,
    fields: Vec<BlockFieldValue<'a>>,
    parent: ScopeRef<'a>,
    /// 
    /// The input to this block.
    ///
    /// If it's None, the input is currently being used, and trying to use it
    /// again will cause a circular dependency error.
    ///
    input: Option<BergResult<'a>>,
}

#[derive(Debug)]
enum BlockState<'a> {
    Ready,
    Running,
    InNextVal,
    Complete(BergResult<'a>),
}

#[derive(Debug, Clone)]
enum BlockFieldValue<'a> {
    NotDeclared,
    NotSet,
    Val(BergVal<'a>),
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
            input: Some(input),
        })))
    }

    pub fn create_child_block(&self, expression: AstIndex, index: BlockIndex) -> Self {
        Self::new(
            expression,
            index,
            ScopeRef::BlockRef(self.clone()),
            empty_tuple().ok(),
        )
    }

    pub fn apply(&self, input: BergVal<'a>) -> BergResult<'a> {
        let block = self.0.borrow();
        // Evaluate immediately and take the result.
        let new_block = Self::new(
            block.expression,
            block.index,
            block.parent.clone(),
            input.ok(),
        );
        new_block
            .take_result(BlockState::Complete(empty_tuple().ok()))
            .evaluate()
    }

    fn take_result(&self, replace_with: BlockState<'a>) -> BergResult<'a> {
        self.ensure_evaluated()?;
        let mut block = self.0.borrow_mut();
        match block.state {
            BlockState::Running | BlockState::InNextVal => {
                CircularDependency.at_location(&block).err()
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
            BlockState::InNextVal => {
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
            BlockState::Running | BlockState::InNextVal => {
                CircularDependency.at_location(&block).err()
            }
            BlockState::Complete(ref result) => result.clone(),
            BlockState::Ready => unreachable!(),
        }
    }

    fn ensure_evaluated(&self) -> Result<(), Exception<'a>> {
        // Check if the block has already been run (and don't re-run)
        let (ast, expression, index) = {
            let mut block = self.0.borrow_mut();
            match block.state {
                BlockState::Running | BlockState::InNextVal => {
                    return CircularDependency.at_location(&block).err();
                }
                BlockState::Complete(_) => return Ok(()),
                BlockState::Ready => {}
            }
            block.state = BlockState::Running;
            let ast = block.ast();
            let index = block.index;
            block.fields.resize(
                ast.blocks[index].scope_count.into(),
                BlockFieldValue::NotDeclared,
            );
            (ast, block.expression, index)
        };

        // Run the block
        let scope = ScopeRef::BlockRef(self.clone());
        let expression = ExpressionEvaluator::new(&scope, &ast, expression);
        println!("{}----------------------------------------", self.indent());
        println!("{} Block evaluating {:?}", self.indent(), expression);
        if let Some(input) = &self.0.borrow().input {
            println!("{} Input: {}", self.indent(), input.display());
        }
        let result = expression.evaluate_block(ast.blocks[index].boundary);

        // Stash the result and return
        self.0.borrow_mut().state = BlockState::Complete(result);
        println!("{} Block state after evaluation: {}", self.indent(), self);
        println!("{}________________________________________", self.indent());
        println!();
        Ok(())
    }

    fn indent(&self) -> String {
        let block = self.0.borrow();
        let scope = ScopeRef::BlockRef(self.clone());
        let ast = block.ast();
        let expression = ExpressionEvaluator::new(&scope, &ast, block.expression);
        let mut result = "  ".repeat(expression.depth());
        result.push('|');
        result
    }

    pub fn local_field(&self, index: FieldIndex, ast: &Ast) -> EvalResult<'a> {
        use BlockFieldValue::*;
        let block = self.0.borrow();
        let scope_start = ast.blocks[block.index].scope_start;
        if index >= scope_start {
            let scope_index: usize = (index - scope_start).into();
            match block.fields.get(scope_index) {
                Some(Val(value)) => value.clone().eval_val(),
                Some(NotSet) => CompilerError::FieldNotSet(index).err(),
                Some(NotDeclared) => CompilerError::NoSuchField(index).err(),
                None => CompilerError::NoSuchField(index).err(),
            }
        } else {
            block.parent.local_field(index, ast)
        }
    }

    ///
    /// Take the block input, replacing it with None to ensure only one person
    /// can write to it at a time.
    ///
    /// If this returns an error, the input does not need to be replaced.
    /// Otherwise, it *must* be replaced.
    ///
    fn take_input(&self) -> Result<BergVal<'a>, EvalException<'a>> {
        let mut block = self.0.borrow_mut();
        match block.input {
            Some(Ok(_)) => block.input.take().unwrap().unwrap().ok(),
            Some(Err(ref error)) => error.clone().err(),
            None => CompilerError::CircularDependency.err(),
        }
    }

    fn replace_input(&self, replace_with: BergResult<'a>) {
        let mut block = self.0.borrow_mut();
        assert!(block.input.is_none());
        block.input = Some(replace_with);
    }

    fn next_input(&self) -> Result<Option<BergVal<'a>>, EvalException<'a>> {
        let next_val = self
            .take_input()?
            .next_val()
            .map_err(|e| e.at_location(self));

        match next_val {
            Ok(NextVal { head, tail }) => {
                self.replace_input(tail.ok());
                Ok(head)
            }
            Err(error) => {
                self.replace_input(error.clone().err());
                error.err()
            }
        }
    }

    pub fn declare_field(
        &self,
        field_index: FieldIndex,
        ast: &Ast,
    ) -> Result<(), EvalException<'a>> {
        // Make sure we have enough spots to put the field
        use BlockFieldValue::*;
        let block_field_index = {
            let block = self.0.borrow();
            let scope_start = ast.blocks[block.index].scope_start;
            if field_index < scope_start {
                return block.parent.declare_field(field_index, ast);
            }

            // The index we intend to insert this at
            let block_field_index: usize = (field_index - scope_start).into();
            if let NotDeclared = block.fields[block_field_index] {
                // The only way to declare a field in an object is to set it while the block is running.
                assert_matches!(block.state, BlockState::Running);
            } else {
                return Ok(());
            };
            block_field_index
        };

        // Get the value *before* we mutably borrow, so that next_val can read
        // current field values if it needs to.
        let next = self.next_input();
        let name = ast.identifier_string(ast.fields[field_index].name);
        println!("{} Received argument {} = {:?}", self.indent(), name, next);
        let value = next?.map(Val).unwrap_or(NotSet);
        self.0.borrow_mut().fields[block_field_index] = value;

        Ok(())
    }

    pub fn set_local_field(
        &self,
        field_index: FieldIndex,
        value: BergVal<'a>,
        ast: &Ast,
    ) -> Result<(), EvalException<'a>> {
        let scope_start = ast.blocks[self.0.borrow().index].scope_start;
        if field_index < scope_start {
            return self
                .0
                .borrow()
                .parent
                .set_local_field(field_index, value, ast);
        }
        println!(
            "{}Set {} to {:?}: {}",
            self.indent(),
            ast.identifier_string(ast.fields[field_index].name),
            value,
            self
        );
        {
            let mut block = self.0.borrow_mut();
            let index: usize = (field_index - scope_start).into();
            block.fields[index] = BlockFieldValue::Val(value);
        }
        Ok(())
    }

    pub fn ast(&self) -> AstRef<'a> {
        self.0.borrow().ast()
    }

    pub fn field_error<T>(
        &self,
        error: FieldError,
        name: IdentifierIndex,
    ) -> Result<T, EvalException<'a>> {
        use FieldError::*;
        match error {
            NoSuchPublicField => CompilerError::NoSuchPublicField(self.clone(), name).err(),
            PrivateField => CompilerError::PrivateField(self.clone(), name).err(),
        }
    }
}

impl<'a> From<&BlockData<'a>> for ExpressionRef<'a> {
    fn from(from: &BlockData<'a>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}
impl<'p, 'a: 'p> From<&Ref<'p, BlockData<'a>>> for ExpressionRef<'a> {
    fn from(from: &Ref<'p, BlockData<'a>>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}
impl<'p, 'a: 'p> From<&RefMut<'p, BlockData<'a>>> for ExpressionRef<'a> {
    fn from(from: &RefMut<'p, BlockData<'a>>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}

impl<'a> From<BlockRef<'a>> for BergVal<'a> {
    fn from(from: BlockRef<'a>) -> Self {
        BergVal::BlockRef(from)
    }
}

impl<'a> From<BlockRef<'a>> for EvalVal<'a> {
    fn from(from: BlockRef<'a>) -> Self {
        BergVal::from(from).into()
    }
}

impl<'a> BergValue<'a> for BlockRef<'a> {}

impl<'a> EvaluatableValue<'a> for BlockRef<'a> {
    fn evaluate(self) -> BergResult<'a>
    where
        Self: Sized,
    {
        self.clone_result()
    }
}

impl<'a> Value<'a> for BlockRef<'a> {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> {
        self.ok()
    }
    fn eval_val(self) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.ok()
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        self.clone_result().into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        self.clone_result().try_into_native()
    }
    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for BlockRef<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        // Get the current result, and prevent anyone else from retrieving this value while we change it
        // by marking state as NextVal
        let next_val = self.take_result(BlockState::InNextVal).next_val();
        match next_val {
            Ok(NextVal { head, tail }) => {
                self.replace_result(tail.ok());
                Ok(NextVal {
                    head,
                    tail: self.into(),
                })
            }
            Err(error) => {
                let error = error.at_location(&self);
                self.replace_result(error.clone().err());
                error.err()
            }
        }
    }
}

impl<'a> ObjectValue<'a> for BlockRef<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        println!(
            "{}====> get {} on {}",
            self.indent(),
            self.ast().identifier_string(name),
            self
        );
        // Always try to get the field from the inner result first
        let current = self.clone_result();
        println!("{}got from {:?}", self.indent(), self);
        let current = current?;
        println!("{}current = {}", self.indent(), current);
        match current.field(name) {
            // If we couldn't find the field on the inner value, see if our block has the field
            Err(EvalException::Thrown(
                BergVal::CompilerError(ref error),
                ExpressionErrorPosition::Expression,
            )) if error.code() == CompilerErrorCode::NoSuchPublicField => {
                println!(
                    "{}====> no such public {} on current",
                    self.indent(),
                    self.ast().identifier_string(name)
                );
                let ast = self.ast();
                let index = {
                    let block = self.0.borrow();
                    let ast_block = &ast.blocks[block.index];
                    let index = ast_block.public_field_index(block.index, name, &ast);
                    index.or_else(|error| self.field_error(error, name))?
                };
                self.local_field(index, &ast)
            }
            Ok(value) => {
                println!(
                    "{}====> got {:?} for {} on {}",
                    self.indent(),
                    value,
                    self.ast().identifier_string(name),
                    self
                );
                Ok(value)
            }
            Err(error) => {
                println!(
                    "{}====> error {} for {} on {}",
                    self.indent(),
                    error,
                    self.ast().identifier_string(name),
                    self
                );
                Err(error)
            } // result => result,
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>> {
        self.ensure_evaluated()?;

        // Figure out the field index from its name
        let ast = self.ast();
        let index = {
            let block = self.0.borrow();
            let ast_block = &ast.blocks[block.index];
            ast_block
                .public_field_index(block.index, name, &ast)
                .or_else(|error| self.field_error(error, name))?
        };

        // Set the field.
        self.set_local_field(index, value, &ast)
    }
}

impl<'a> OperableValue<'a> for BlockRef<'a> {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use crate::syntax::identifiers::*;
        use EvalVal::*;

        match operator {
            DOT => default_infix(self, operator, right),
            FOLLOWED_BY | APPLY => {
                let arguments = right.get()?;
                let input = match arguments {
                    // Any commas are treated as separate arguments, so `f 1,2,3` is
                    // 3 arguments. `f (1,2,3), (4,5,6)` however, is two arguments.
                    RightOperand(PartialTuple(_), _) | RightOperand(TrailingComma(_), _) => {
                        arguments.lazy_val()?
                    }
                    RightOperand(MissingExpression, _) if operator == APPLY => empty_tuple(),
                    // f (1,2,3) is a single argument which is itself a tuple.
                    _ => BergVal::from(vec![arguments.lazy_val()?]),
                };
                self.apply(input)?.ok()
            }
            _ => self.clone_result().infix(operator, right),
        }
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.clone_result().infix_assign(operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        // Closures report their own internal error instead of local ones.
        self.clone_result().prefix(operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.clone_result().postfix(operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a>
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}

impl<'a> BlockData<'a> {
    pub fn ast(&self) -> AstRef<'a> {
        self.parent.ast()
    }
}

// BlockRef/BlockData -> ExpressionRef makes error.at_location() work
impl<'a> From<&BlockRef<'a>> for ExpressionRef<'a> {
    fn from(from: &BlockRef<'a>) -> Self {
        ExpressionRef::from(&from.0.borrow())
    }
}

impl<'a> fmt::Display for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BlockFieldValue::*;
        let block = self.0.borrow();
        let ast = self.ast();
        write!(f, "block({}", block.state)?;
        match &block.input {
            Some(Ok(BergVal::Tuple(tuple))) if tuple.is_empty() => {}
            Some(input) => write!(f, ", input: {}", input.display())?,
            None => write!(f, "input: <input currently being used>")?,
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
                    Val(value) => write!(f, "{}: {}", name, value)?,
                    NotDeclared => write!(f, "{}: <undeclared>", name)?,
                    NotSet => write!(f, "{}: <not set>", name)?,
                }
            }
            write!(f, "}}")?;
        }
        write!(
            f,
            ", expression: {}",
            ExpressionTreeWalker::basic(&ast, block.expression)
        )?;
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BlockState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockState::Complete(result) => write!(f, "Complete({})", result.display()),
            BlockState::Ready => write!(f, "Ready",),
            BlockState::Running => write!(f, "Running"),
            BlockState::InNextVal => write!(f, "NextVal"),
        }
    }
}

impl<'a> fmt::Debug for BlockRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BlockFieldValue::*;
        let block = self.0.borrow();
        let ast = self.ast();
        write!(f, "BlockRef {{ state: {:?}", block.state)?;
        match &block.input {
            Some(Ok(BergVal::Tuple(tuple))) if tuple.is_empty() => {}
            input => write!(f, ", input: {:?}", input)?,
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
                    Val(value) => write!(f, "{}: {:?}", name, value)?,
                    NotDeclared => write!(f, "{}: <undeclared>", name)?,
                    NotSet => write!(f, "{}: <not set>", name)?,
                }
            }
            write!(f, "}}")?;
        }
        write!(
            f,
            ", expression: {:?}, parent: {:?}",
            ExpressionTreeWalker::basic(&ast, block.expression),
            block.parent
        )?;
        write!(f, "}}")
    }
}

impl<'a> PartialEq for BlockRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
