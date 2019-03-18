use crate::eval::{BlockRef, ScopeRef};
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::syntax::identifiers::*;
use crate::value::implement::*;
use std::fmt;
use EvalVal::*;

#[derive(Debug, Clone)]
pub enum EvalVal<'a> {
    /// non-syntactical results (integers, booleans, results of math expressions ...)
    Value(BergVal<'a>),
    /// if
    If,
    /// else
    Else,
    /// if, else evaluation
    ConditionalVal(ConditionalState, Option<BlockRef<'a>>),
    /// while
    While,
    /// while <condition>
    WhileCondition(BlockRef<'a>),
    /// foreach
    Foreach,
    /// foreach <input>
    ForeachInput(BergVal<'a>),
    /// 1 + <here>
    MissingExpression,
    /// 1,2
    PartialTuple(Vec<BergVal<'a>>),
    /// a.b (refers to the b)
    RawIdentifier(IdentifierIndex),
    /// 1,2,
    TrailingComma(Vec<BergVal<'a>>),
    /// 1;2;
    TrailingSemicolon,
    /// Things that can be assigned to: a, :a, a.b
    Target(AssignmentTarget<'a>),
}

///
/// The result returned from most BergValue operations
/// 
pub type EvalResult<'a> = Result<EvalVal<'a>, ErrorVal<'a>>;

#[derive(Debug, Clone)]
pub enum AssignmentTarget<'a> {
    LocalFieldReference(ScopeRef<'a>, FieldIndex),
    LocalFieldDeclaration(ScopeRef<'a>, FieldIndex),
    ObjectFieldReference(BergVal<'a>, IdentifierIndex),
}

#[derive(Debug, Clone)]
pub enum ConditionalState {
    /// if ^
    IfCondition,
    /// if true ^
    RunBlock,
    /// if false ^
    IgnoreBlock,
    /// else ^
    ElseBlock,
    /// (if true | if false | else) {} ^
    MaybeElse,
}

impl<'a> EvalVal<'a> {
    //
    // If this is a reference to something else, resolve it (it might be a syntax
    // value like Else or If).
    //
    pub fn get(self) -> EvalResult<'a> {
        use EvalVal::*;
        match self {
            Target(v) => v.get(),
            Value(_) | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | RawIdentifier(_) => self.ok(),
        }
    }
}

impl<'a> BergValue<'a> for EvalVal<'a> {
    fn into_val(self) -> BergResult<'a> {
        use EvalVal::*;
        use ConditionalState::*;
        use BergError::*;
        match self {
            Value(v) => v.into_val(),
            Target(v) => v.into_val(),
            RawIdentifier(v) => v.into_val(),

            If => IfWithoutCondition.err(),
            ConditionalVal(IfCondition, _) => IfWithoutCondition.operand_err(Right),
            Else => ElseWithoutIf.err(),
            ConditionalVal(ElseBlock, _) => ElseWithoutBlock.operand_err(Right),
            ConditionalVal(RunBlock, _) | ConditionalVal(IgnoreBlock, _) => IfWithoutBlock.err(),
            ConditionalVal(MaybeElse, None) => BergVal::empty_tuple().ok(),
            ConditionalVal(MaybeElse, Some(v)) => v.ok(),
            While => WhileWithoutCondition.err(),
            WhileCondition(_) => WhileWithoutBlock.operand_err(Left),
            Foreach => ForeachWithoutInput.err(),
            ForeachInput(_) => ForeachWithoutBlock.operand_err(Left),
            MissingExpression => MissingOperand.err(),
            PartialTuple(vec) | TrailingComma(vec) => Tuple::from(vec).ok(),
            TrailingSemicolon => BergVal::empty_tuple().ok(),
        }
    }
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.into_val().at_position(new_position)
    }
    fn eval_val(self) -> EvalResult<'a> {
        match self {
            Value(v) => v.eval_val(),
            Target(v) => v.eval_val(),
            RawIdentifier(v) => v.eval_val(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.ok(),
        }
    }
    fn evaluate(self) -> BergResult<'a> {
        match self {
            Value(v) => v.evaluate(),
            Target(v) => v.evaluate(),
            RawIdentifier(v) => v.evaluate(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val()?.evaluate(),
        }
    }
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        match self {
            Value(v) => v.next_val(),
            Target(v) => v.next_val(),
            RawIdentifier(v) => v.next_val(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val()?.next_val(),
        }
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        match self {
            Value(v) => v.into_native(),
            Target(v) => v.into_native(),
            RawIdentifier(v) => v.into_native(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val()?.into_native(),
        }
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        match self {
            Value(v) => v.try_into_native(),
            Target(v) => v.try_into_native(),
            RawIdentifier(v) => v.try_into_native(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val()?.try_into_native(),
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use LeftRight;
        use BergError::*;
        use ConditionalState::*;
        match self {
            Value(v) => v.infix(operator, right),
            Target(v) => v.infix(operator, right),
            RawIdentifier(v) => v.infix(operator, right),
            TrailingSemicolon if operator == SEMICOLON => BergError::MissingOperand.operand_err(LeftRight),
            TrailingComma(_) if operator == COMMA => BergError::MissingOperand.operand_err(LeftRight),
            PartialTuple(mut vec) => match operator {
                COMMA => { vec.push(right.into_val()?); PartialTuple(vec).ok() }
                _ => PartialTuple(vec).into_val().infix(operator, right)
            }
            // if <condition>, if false else if <condition>
            If if operator == APPLY => match right.get()? {
                RightOperand(If, _) | RightOperand(Else, _) => IfWithoutCondition.operand_err(Left),
                right => if right.into_native::<bool>()? {
                    ConditionalVal(RunBlock, None).ok()
                } else {
                    ConditionalVal(IgnoreBlock, None).ok()
                }
            }
            ConditionalVal(state, result) => {
                if operator == APPLY {
                    match state {
                        IfCondition => match right.get()? {
                            // if if, if else
                            RightOperand(If, _) | RightOperand(Else, _) => IfWithoutCondition.operand_err(LeftRight),
                            // if <condition>
                            right => if result.is_none() && right.into_native::<bool>()? {
                                ConditionalVal(RunBlock, result).ok()
                            } else {
                                ConditionalVal(IgnoreBlock, result).ok()
                            }
                        }
                        RunBlock if result.is_none() => match right.into_val() {
                            // if true {}
                            Ok(BergVal::BlockRef(block)) => ConditionalVal(MaybeElse, Some(block)).ok(),
                            // if true 1
                            _ => IfBlockMustBeBlock.operand_err(Right),
                        }
                        RunBlock => unreachable!(),
                        IgnoreBlock => match right.into_val() {
                            // if false {}
                            Ok(BergVal::BlockRef(_)) => ConditionalVal(MaybeElse, result).ok(),
                            // if false 1
                            _ => IfBlockMustBeBlock.operand_err(Right),
                        }
                        // else ^
                        ElseBlock => match right.get()? {
                            // else if
                            RightOperand(If, _) => ConditionalVal(IfCondition, result).ok(),
                            right => match right.into_val() {
                                Ok(BergVal::BlockRef(block)) => match result {
                                    // if false {} else {}
                                    None => block.ok(),
                                    // if true {} else {}
                                    Some(val) => val.ok(),
                                }
                                // if true|false else else
                                // if true|false else 1
                                _ => ElseBlockMustBeBlock.operand_err(Right),
                            }
                        }
                        // if true|false {} <something>
                        MaybeElse => match right.get()? {
                            // if true|false {} else
                            RightOperand(Else, _) => ConditionalVal(ElseBlock, result).ok(),
                            // if true|false {} if
                            // if true|false {} 1
                            _ => IfFollowedByNonElse.err()
                        }
                    }
                } else {
                    ConditionalVal(state, result).into_val().infix(operator, right)
                }
            }
            // while <condition>
            While if operator == APPLY => match right.into_val()? {
                BergVal::BlockRef(block) => WhileCondition(block).ok(),
                _ => WhileConditionMustBeBlock.operand_err(Right),
            }
            WhileCondition(condition) => if operator == APPLY {
                match right.into_val()? {
                    BergVal::BlockRef(block) => run_while_loop(condition, block),
                    _ => WhileBlockMustBeBlock.operand_err(Right),
                }
            } else {
                WhileCondition(condition).into_val().infix(operator, right)
            }
            // while <condition>
            Foreach if operator == APPLY => ForeachInput(right.into_val()?).ok(),
            ForeachInput(input) => if operator == APPLY {
                match right.into_val()? {
                    BergVal::BlockRef(block) => run_foreach(input, block),
                    _ => ForeachBlockMustBeBlock.operand_err(Right),
                }
            } else {
                ForeachInput(input).into_val().infix(operator, right)
            }
            MissingExpression | TrailingSemicolon | TrailingComma(_) | If | Else | While | Foreach => self.into_val().infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        match self {
            Value(v) => v.infix_assign(operator, right),
            Target(v) => v.infix_assign(operator, right),
            RawIdentifier(v) => v.infix_assign(operator, right),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val().infix_assign(operator, right),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Value(v) => v.prefix(operator),
            Target(v) => v.prefix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val().prefix(operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Value(v) => v.postfix(operator),
            Target(v) => v.postfix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val().postfix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        use ExpressionBoundary::*;
        match self {
            Value(v) => v.subexpression_result(boundary),
            Target(v) => v.subexpression_result(boundary),
            RawIdentifier(v) => v.subexpression_result(boundary),
            MissingExpression if boundary == Parentheses || boundary.is_block() => BergVal::empty_tuple().ok(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val().subexpression_result(boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Value(v) => v.field(name),
            Target(v) => v.field(name),
            RawIdentifier(v) => v.field(name),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => self.into_val().field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        match self {
            Value(v) => v.set_field(name, value),
            Target(v) => v.set_field(name, value),
            RawIdentifier(v) => v.set_field(name, value),
            MissingExpression => BergError::MissingOperand.err(),
            PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_)  => panic!("not yet implemented: can't set field {} on {:?} to {}", name, self, value),
        }
    }
}

fn run_while_loop<'a>(condition: BlockRef<'a>, block: BlockRef<'a>) -> EvalResult<'a> {
    use ErrorCode::*;
    loop {
        // Check the condition, and break if false (or error).
        match condition.apply(BergVal::empty_tuple().into()).into_native::<bool>() {
            Ok(true) => {},
            Ok(false) => break,
            // (while APPLY { condition }) APPLY { block } means condition is the left operand's right operand
            Err(error) => return error.reposition(LeftRight).err(),
        };

        // Run the block.
        let result = block.apply(BergVal::empty_tuple().into());
        match result.into_val() {
            Ok(_) => {},
            Err(error) => match error.code() {
                BreakOutsideLoop => break,
                ContinueOutsideLoop => continue,
                // (while APPLY { condition }) APPLY { block } means block is right operand
                _ => return error.reposition(LeftRight).err()
            }
        }
    }
    BergVal::empty_tuple().ok()
}

fn run_foreach<'a>(input: BergVal<'a>, block: BlockRef<'a>) -> EvalResult<'a> {
    use ErrorCode::*;
    let mut remaining = input.ok();
    loop {
        // Grab the input.
        let value = match remaining.next_val() {
            Ok(Some(NextVal { head, tail })) => {
                remaining = tail;
                head
            },
            Ok(None) => break,
            Err(error) => return error.reposition(Right).err(),
        };

        // Run the block.
        let result = block.apply(value.into());
        match result.into_val() {
            Ok(_) => {},
            Err(error) => match error.code() {
                BreakOutsideLoop => break,
                ContinueOutsideLoop => continue,
                // (while APPLY { condition }) APPLY { block } means block is right operand
                _ => return error.reposition(LeftRight).err()
            }
        }
    }
    BergVal::empty_tuple().ok()
}

impl<'a> fmt::Display for EvalVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ConditionalState::*;
        match self {
            Value(v) => write!(f, "{}", v),
            Target(v) => write!(f, "{}", v),
            RawIdentifier(v) => write!(f, "{}", v),
            MissingExpression => write!(f, "<missing>"),
            PartialTuple(vec) => write!(f, "<partial tuple> {:?}", vec),
            TrailingComma(vec) => write!(f, "<trailing ,> {:?}", vec),
            TrailingSemicolon => write!(f, "<trailing ;>"),
            If => write!(f, "if"),
            Else => write!(f, "else"),
            ConditionalVal(ElseBlock, None) => write!(f, "else <run block>"),
            ConditionalVal(ElseBlock, Some(v)) => write!(f, "else <ignore block> -> {}", v),
            ConditionalVal(IfCondition, None) => write!(f, "if <run condition>"),
            ConditionalVal(IfCondition, Some(v)) => write!(f, "if <ignore condition> -> {}", v),
            ConditionalVal(IgnoreBlock, None) => write!(f, "if false <ignore block>"),
            ConditionalVal(IgnoreBlock, Some(v)) => write!(f, "if <ignore block> -> {}", v),
            ConditionalVal(RunBlock, None) => write!(f, "if true <run block>"),
            ConditionalVal(RunBlock, Some(_)) => unreachable!(),
            ConditionalVal(MaybeElse, None) => write!(f, "complete if -> ()"),
            ConditionalVal(MaybeElse, Some(v)) => write!(f, "complete if -> {}", v),
            While => write!(f, "while"),
            WhileCondition(condition) => write!(f, "while {}", condition),
            Foreach => write!(f, "foreach"),
            ForeachInput(input) => write!(f, "foreach {}", input),
         }
    }
}

impl<'a> AssignmentTarget<'a> {
    pub fn get(&self) -> EvalResult<'a> {
        // If it's a declaration, declare it and get its initial value, if any.
        self.declare()?;
        self.get_internal()
    }

    pub fn set(&mut self, value: BergVal<'a>, operand_position: ExpressionErrorPosition) -> EvalResult<'a> {
        match self.set_internal(value).and_then(|_| self.declare()) {
            Ok(()) => BergVal::empty_tuple().ok(),
            Err(error) => error.reposition(operand_position).err(),
        }
    }

    fn declare(&self) -> Result<(), ErrorVal<'a>> {
        use AssignmentTarget::*;
        match self {
            LocalFieldDeclaration(scope, field) => scope.declare_field(*field, &scope.ast())?,
            LocalFieldReference(..) | ObjectFieldReference(..) => {}
        }
        Ok(())
    }

    fn get_internal(&self) -> EvalResult<'a> {
        use AssignmentTarget::*;
        let result = match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => 
                scope.local_field(*field, &scope.ast()),
            ObjectFieldReference(object, name) => object.clone().field(*name)
        };
        self.point_errors_at_identifier(result)
    }

    fn set_internal(&mut self, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        use AssignmentTarget::*;
        let result = match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => 
                scope.set_local_field(*field, value, &scope.ast()),
            ObjectFieldReference(object, name) => {
                object.set_field(*name, value)
            }
        };
        self.point_errors_at_identifier(result)?;
        Ok(())
    }

    fn point_errors_at_identifier<T: fmt::Debug>(&self, result: Result<T, ErrorVal<'a>>) -> Result<T, ErrorVal<'a>> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::*;
        match result {
            Err(ErrorVal::ExpressionError(error, Expression)) => match self {
                LocalFieldDeclaration(..) | ObjectFieldReference(..) => error.operand_err(Right),
                LocalFieldReference(..) => error.err(),
            },
            Err(error) => Err(error),
            Ok(value) => Ok(value),
        }
    }
}

impl<'a> From<AssignmentTarget<'a>> for EvalVal<'a> {
    fn from(from: AssignmentTarget<'a>) -> Self {
        EvalVal::Target(from)
    }
}

impl<'a> BergValue<'a> for AssignmentTarget<'a> {
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        self.get().next_val()
    }
    fn into_val(self) -> BergResult<'a> {
        self.get().into_val()
    }
    fn evaluate(self) -> BergResult<'a> {
        self.get().evaluate()
    }
    fn eval_val(self) -> EvalResult<'a> {
        self.ok()
    }
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.get().at_position(new_position)
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        self.get().into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        self.get().try_into_native()
    }
    fn infix(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use AssignmentTarget::*;
        use Left;
        match (operator, &self) {
            // Handle <identifier>: <value>
            (COLON, LocalFieldReference(..)) => self.set(right.into_val()?, Left),
            _ => self.get().infix(operator, right)
        }
    }
    fn infix_assign(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use Left;
        match operator {
            EMPTY_STRING => self.set(right.into_val()?, Left),
            operator => self.set(self.get().infix(operator, right).into_val()?, Left),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        use AssignmentTarget::*;
        use Right;
        match (operator, self) {
            (COLON, LocalFieldReference(scope, field)) => LocalFieldDeclaration(scope, field).ok(),
            (PLUS_PLUS, mut right) => right.set(right.get().prefix(PLUS_ONE).into_val()?, Right),
            (DASH_DASH, mut right) => right.set(right.get().prefix(MINUS_ONE).into_val()?, Right),
            (_, right) => right.get().prefix(operator),
        }
    }

    fn postfix(mut self, operator: IdentifierIndex) -> EvalResult<'a> {
        use Left;
        match operator {
            PLUS_PLUS => self.set(self.get().postfix(PLUS_ONE).into_val()?, Left),
            DASH_DASH => self.set(self.get().postfix(MINUS_ONE).into_val()?, Left),
            _ => self.get().postfix(operator)
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        self.get().subexpression_result(boundary)
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        self.get().field(name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        use ExpressionErrorPosition::Expression;
        let mut obj = self.get().into_val()?;
        obj.set_field(name, value)?;
        self.set(obj, Expression).and(Ok(()))
    }
}

impl<'a> fmt::Display for AssignmentTarget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AssignmentTarget::*;
        match self {
            LocalFieldReference(scope, field) => write!(f, "{}", scope.ast().fields[*field].name),
            LocalFieldDeclaration(scope, field) => write!(f, "{}", scope.ast().fields[*field].name),
            ObjectFieldReference(object, name) => write!(f, "{}.{}", object, name),
        }
    }
}
