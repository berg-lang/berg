use crate::eval::ExpressionEvaluator;
use crate::value::implement::*;
use berg_parser::{
    Ast, AstIndex, BlockIndex, ExpressionPosition, ExpressionToken, ExpressionTreeWalker,
    FieldError, FieldIndex, IdentifierIndex,
};
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
pub struct BlockRef(Rc<RefCell<BlockData>>);

#[derive(Clone)]
enum BlockParentRef {
    BlockRef(BlockRef),
    AstRef(AstRef),
}

#[derive(Debug)]
struct BlockData {
    expression: AstIndex,
    state: BlockState,
    index: BlockIndex,
    fields: Vec<BlockFieldValue>,
    parent: BlockParentRef,
    ///
    /// The input to this block.
    ///
    /// If it's None, the input is currently being used, and trying to use it
    /// again will cause a circular dependency error.
    ///
    input: Option<BergResult>,
}

#[derive(Debug)]
enum BlockState {
    Ready,
    Running,
    InNextVal,
    Complete(BergResult),
}

#[derive(Debug, Clone)]
enum BlockFieldValue {
    NotDeclared,
    NotSet,
    Val(BergVal),
}

impl BlockRef {
    ///
    /// Create a new block from the given AST.
    ///
    pub fn from_ast(ast: AstRef) -> Result<Self, Exception> {
        let open = ast.root_expression();
        match ast.expression_token(open) {
            ExpressionToken::Open(None, ExpressionBoundary::Source, delta) => {
                let index = ast.close_block_index(open + delta);
                Self::new(open, index, BlockParentRef::AstRef(ast), empty_tuple().ok()).ok()
            }
            _ => unreachable!("AST root must be a Source block"),
        }
    }

    ///
    /// Create a new block that will run the given expression against the
    /// given scope and input.
    ///
    fn new(
        expression: AstIndex,
        index: BlockIndex,
        parent: BlockParentRef,
        input: BergResult,
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
            BlockParentRef::BlockRef(self.clone()),
            empty_tuple().ok(),
        )
    }

    pub fn apply(&self, input: BergVal) -> BergResult {
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

    fn take_result(&self, replace_with: BlockState) -> BergResult {
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

    fn replace_result(&self, result: BergResult) {
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

    fn clone_result(&self) -> BergResult {
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

    fn ensure_evaluated(&self) -> Result<(), Exception> {
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
        let expression = ExpressionEvaluator::new(self, &ast, expression);
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
        let ast = block.ast();
        let expression = ExpressionEvaluator::new(self, &ast, block.expression);
        let mut result = "  ".repeat(expression.depth());
        result.push('|');
        result
    }

    pub fn local_field(&self, index: FieldIndex, ast: &Ast) -> EvalResult {
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
    fn take_input(&self) -> Result<BergVal, EvalException> {
        let mut block = self.0.borrow_mut();
        match block.input {
            Some(Ok(_)) => block.input.take().unwrap().unwrap().ok(),
            Some(Err(ref error)) => error.clone().err(),
            None => CompilerError::CircularDependency.err(),
        }
    }

    fn replace_input(&self, replace_with: BergResult) {
        let mut block = self.0.borrow_mut();
        assert!(block.input.is_none());
        block.input = Some(replace_with);
    }

    fn next_input(&self) -> Result<Option<BergVal>, EvalException> {
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
    ) -> Result<(), EvalException> {
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
        value: BergVal,
        ast: &Ast,
    ) -> Result<(), EvalException> {
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

    pub fn ast(&self) -> AstRef {
        self.0.borrow().ast()
    }

    pub fn field_error<T>(
        &self,
        error: FieldError,
        name: IdentifierIndex,
    ) -> Result<T, EvalException> {
        use FieldError::*;
        match error {
            NoSuchPublicField => CompilerError::NoSuchPublicField(self.clone(), name).err(),
            PrivateField => CompilerError::PrivateField(self.clone(), name).err(),
        }
    }
}

impl From<&BlockData> for ExpressionRef {
    fn from(from: &BlockData) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}
impl<'p> From<&Ref<'p, BlockData>> for ExpressionRef {
    fn from(from: &Ref<'p, BlockData>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}
impl<'p> From<&RefMut<'p, BlockData>> for ExpressionRef {
    fn from(from: &RefMut<'p, BlockData>) -> Self {
        ExpressionRef::new(from.ast(), from.expression)
    }
}

impl From<BlockRef> for BergVal {
    fn from(from: BlockRef) -> Self {
        BergVal::BlockRef(from)
    }
}

impl From<BlockRef> for EvalVal {
    fn from(from: BlockRef) -> Self {
        BergVal::from(from).into()
    }
}

impl BergValue for BlockRef {}

impl EvaluatableValue for BlockRef {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.clone_result()
    }
}

impl Value for BlockRef {
    fn lazy_val(self) -> Result<BergVal, EvalException> {
        self.ok()
    }
    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        self.ok()
    }
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        self.clone_result().into_native()
    }
    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        self.clone_result().try_into_native()
    }
    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl IteratorValue for BlockRef {
    fn next_val(self) -> Result<NextVal, EvalException> {
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

impl ObjectValue for BlockRef {
    fn field(self, name: IdentifierIndex) -> EvalResult
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
                ExpressionPosition::Expression,
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
        value: BergVal,
    ) -> Result<(), EvalException> {
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

impl OperableValue for BlockRef {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        use berg_parser::identifiers::*;
        use EvalVal::*;

        match operator {
            DOT => default_infix(self, operator, right),
            FOLLOWED_BY | APPLY => {
                let arguments = right.get()?;
                let input = match arguments {
                    // Any commas are treated as separate arguments, so `f 1,2,3` is
                    // 3 arguments. `f (1,2,3), (4,5,6)` however, is two arguments.
                    RightOperand(PartialTuple(_)) | RightOperand(TrailingComma(_)) => {
                        arguments.lazy_val()?
                    }
                    RightOperand(MissingExpression) if operator == APPLY => empty_tuple(),
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
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        self.clone_result().infix_assign(operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        // Closures report their own internal error instead of local ones.
        self.clone_result().prefix(operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        self.clone_result().postfix(operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}

impl BlockData {
    pub fn ast(&self) -> AstRef {
        self.parent.ast()
    }
}

// BlockRef/BlockData -> ExpressionRef makes error.at_location() work
impl From<&BlockRef> for ExpressionRef {
    fn from(from: &BlockRef) -> Self {
        ExpressionRef::from(&from.0.borrow())
    }
}

impl fmt::Display for BlockRef {
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

impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockState::Complete(result) => write!(f, "Complete({})", result.display()),
            BlockState::Ready => write!(f, "Ready",),
            BlockState::Running => write!(f, "Running"),
            BlockState::InNextVal => write!(f, "NextVal"),
        }
    }
}

impl fmt::Debug for BlockRef {
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

impl PartialEq for BlockRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl BlockParentRef {
    pub fn local_field(&self, index: FieldIndex, ast: &Ast) -> EvalResult {
        match &self {
            BlockParentRef::BlockRef(block) => block.local_field(index, ast),
            BlockParentRef::AstRef(ast) => ast.root.local_field(index),
        }
    }

    // pub fn field(self, name: IdentifierIndex) -> EvalResult
    // where
    //     Self: Sized,
    // {
    //     match self {
    //         BlockParentRef::BlockRef(block) => block.field(name),
    //         BlockParentRef::AstRef(ast) => ast.source.root.field(name),
    //     }
    // }
    pub fn declare_field(&self, index: FieldIndex, ast: &Ast) -> Result<(), EvalException> {
        match self {
            BlockParentRef::BlockRef(block) => block.declare_field(index, ast),
            BlockParentRef::AstRef(ref ast) => ast.root.declare_field(index),
        }
    }
    pub fn set_local_field(
        &self,
        index: FieldIndex,
        value: BergVal,
        ast: &Ast,
    ) -> Result<(), EvalException> {
        match self {
            BlockParentRef::BlockRef(block) => block.set_local_field(index, value, ast),
            BlockParentRef::AstRef(ast) => ast.root.set_local_field(index, value),
        }
    }
    pub fn ast(&self) -> AstRef {
        match self {
            BlockParentRef::BlockRef(block) => block.ast(),
            BlockParentRef::AstRef(ast) => ast.clone(),
        }
    }
}

impl fmt::Debug for BlockParentRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockParentRef::BlockRef(ref block) => block.fmt(f),
            BlockParentRef::AstRef(ref ast) => f
                .debug_struct("AstRef")
                .field("fields", &ast.root.field_names())
                .finish(),
        }
    }
}
