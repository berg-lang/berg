use crate::eval::{BlockRef, ScopeRef};
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::syntax::identifiers::*;
use crate::value::implement::*;
use std::fmt;
use EvalVal::*;

///
/// The result of an evaluation.
/// 
/// The primary distinction between this and [`BergVal`] is that an [`EvalVal`]
/// can be "contextual," meaning depending on what expressions are run on it
/// it will behave differently than a normal value.
/// 
/// It is primarily used for ambiguous syntax: for example, when a
/// MissingExpression occurs, it might be an error like in `(1 + )`, or ignored
/// like in `1,2,` and `f();`, or it might yield an empty tuple like in `()` and
/// `{}`.
/// 
#[derive(Debug, Clone)]
pub enum EvalVal<'a> {
    /// non-syntactical results (integers, booleans, results of math expressions ...)
    Val(BergVal<'a>),
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
    ForeachInput(Result<BergVal<'a>, EvalException<'a>>),
    /// try
    Try,
    /// try { <error> }
    TryResult(BergResult<'a>),
    /// catch
    Catch,
    /// <error> catch
    TryCatch(BergResult<'a>),
    /// <error> catch { block }
    CatchResult(BergResult<'a>),
    /// finally
    Finally,
    /// <anything> finally
    TryFinally(BergResult<'a>),
    /// throw
    Throw,
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

pub trait AtLocation<'a>: Sized {
    ///
    /// Give this value a location, making it global.
    /// 
    fn at_location(self, location: impl Into<ExpressionRef<'a>>) -> Result<EvalVal<'a>, Exception<'a>>;
}

///
/// The result returned from most BergValue operations
/// 
pub type EvalResult<'a> = Result<EvalVal<'a>, EvalException<'a>>;

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
            Val(_) | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw | MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | RawIdentifier(_) => self.ok(),
        }
    }
}

impl<'a> Value<'a> for EvalVal<'a> {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized {
        use EvalVal::*;
        use ConditionalState::*;
        use CompilerError::*;
        match self {
            Val(v) => v.lazy_val(),
            Target(v) => v.lazy_val(),
            RawIdentifier(v) => v.lazy_val(),

            If => IfWithoutCondition.err(),
            ConditionalVal(IfCondition, _) => IfWithoutCondition.operand_err(Right),
            Else => ElseWithoutIf.err(),
            ConditionalVal(ElseBlock, _) => ElseWithoutBlock.operand_err(Right),
            ConditionalVal(RunBlock, _) | ConditionalVal(IgnoreBlock, _) => IfWithoutBlock.operand_err(Left),
            ConditionalVal(MaybeElse, None) => empty_tuple().ok(),
            ConditionalVal(MaybeElse, Some(v)) => v.ok(),
            While => WhileWithoutCondition.err(),
            WhileCondition(_) => WhileWithoutBlock.operand_err(Left),
            Foreach => ForeachWithoutInput.err(),
            ForeachInput(_) => ForeachWithoutBlock.operand_err(Left),
            Try => TryWithoutBlock.err(),
            TryResult(_) => TryWithoutCatchOrFinally.err(),
            Catch => CatchWithoutResult.err(),
            TryCatch(_) => CatchWithoutBlock.err(),
            CatchResult(result) => result?.ok(),
            Finally => FinallyWithoutResult.err(),
            TryFinally(_) => FinallyWithoutBlock.err(),
            Throw => ThrowWithoutException.err(),

            MissingExpression => MissingOperand.err(),
            PartialTuple(vec) | TrailingComma(vec) => Tuple::from(vec).ok(),
            TrailingSemicolon => empty_tuple().ok(),
        }
    }

    fn eval_val(self) -> EvalResult<'a> {
        match self {
            Val(v) => v.eval_val(),
            Target(v) => v.eval_val(),
            RawIdentifier(v) => v.eval_val(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.ok(),
        }
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        match self {
            Val(v) => v.into_native(),
            Target(v) => v.into_native(),
            RawIdentifier(v) => v.into_native(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().into_native(),
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        match self {
            Val(v) => v.try_into_native(),
            Target(v) => v.try_into_native(),
            RawIdentifier(v) => v.try_into_native(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().try_into_native(),
        }
    }

    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for EvalVal<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        match self {
            Val(v) => v.next_val(),
            Target(v) => v.next_val(),
            RawIdentifier(v) => v.next_val(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().next_val(),
        }
    }
}

impl<'a> ObjectValue<'a> for EvalVal<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match self {
            Val(v) => v.field(name),
            Target(v) => v.field(name),
            RawIdentifier(v) => v.field(name),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), EvalException<'a>> {
        match self {
            Val(v) => v.set_field(name, value),
            Target(v) => v.set_field(name, value),
            RawIdentifier(v) => v.set_field(name, value),
            MissingExpression => CompilerError::MissingOperand.err(),
            PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => panic!("not yet implemented: can't set field {} on {:?} to {}", name, self, value),
        }
    }
}

impl<'a> OperableValue<'a> for EvalVal<'a> {
    // TODO Yes, bad, must fix.
    #[allow(clippy::cyclomatic_complexity)]
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        use CompilerError::*;
        use ConditionalState::*;
        match self {
            Val(v) => v.infix(operator, right),
            Target(v) => v.infix(operator, right),
            RawIdentifier(v) => v.infix(operator, right),
            TrailingSemicolon if operator == SEMICOLON => CompilerError::MissingOperand.operand_err(LeftRight),
            TrailingComma(_) if operator == COMMA => CompilerError::MissingOperand.operand_err(LeftRight),
            PartialTuple(mut vec) => match operator {
                COMMA => match right.get()? {
                    RightOperand(MissingExpression, _) => TrailingComma(vec).ok(),
                    value => { vec.push(value.lazy_val()?); PartialTuple(vec).ok() }
                }
                _ => PartialTuple(vec).lazy_val().infix(operator, right)
            }
            // if <condition>, if false else if <condition>
            If if operator.is_followed_by() => match right.get()? {
                RightOperand(If, _) | RightOperand(Else, _) => IfWithoutCondition.operand_err(Left),
                right => if right.into_native::<bool>()? {
                    ConditionalVal(RunBlock, None).ok()
                } else {
                    ConditionalVal(IgnoreBlock, None).ok()
                }
            }
            ConditionalVal(state, result) => {
                if operator.is_followed_by() {
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
                        RunBlock if result.is_none() => match right.lazy_val() {
                            // if true {}
                            Ok(BergVal::BlockRef(block)) => ConditionalVal(MaybeElse, Some(block)).ok(),
                            // if true 1
                            _ => IfBlockMustBeBlock.operand_err(Right),
                        }
                        RunBlock => unreachable!(),
                        IgnoreBlock => match right.lazy_val() {
                            // if false {}
                            Ok(BergVal::BlockRef(_)) => ConditionalVal(MaybeElse, result).ok(),
                            // if false 1
                            _ => IfBlockMustBeBlock.operand_err(Right),
                        }
                        // else ^
                        ElseBlock => match right.get()? {
                            // else if
                            RightOperand(If, _) => ConditionalVal(IfCondition, result).ok(),
                            right => match right.lazy_val() {
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
                            _ => IfFollowedByNonElse.operand_err(Right)
                        }
                    }
                } else {
                    ConditionalVal(state, result).lazy_val().infix(operator, right)
                }
            }
            // while <condition>
            While if operator.is_followed_by() => match right.lazy_val()? {
                BergVal::BlockRef(block) => WhileCondition(block).ok(),
                _ => WhileConditionMustBeBlock.operand_err(Right),
            }
            WhileCondition(condition) => if operator.is_followed_by() {
                match right.lazy_val()? {
                    BergVal::BlockRef(block) => run_while_loop(condition, block),
                    _ => WhileBlockMustBeBlock.operand_err(Right),
                }
            } else {
                WhileCondition(condition).lazy_val().infix(operator, right)
            }
            // while <condition>
            Foreach if operator.is_followed_by() => ForeachInput(right.lazy_val()).ok(),
            ForeachInput(input) => if operator.is_followed_by() {
                match right.lazy_val()? {
                    BergVal::BlockRef(block) => run_foreach(input, block),
                    _ => ForeachBlockMustBeBlock.operand_err(Right),
                }
            } else {
                ForeachInput(input).lazy_val().infix(operator, right)
            }
            Try if operator.is_followed_by() => match right.lazy_val()? {
                BergVal::BlockRef(block) => TryResult(block.evaluate()).ok(),
                _ => TryBlockMustBeBlock.operand_err(Right),
            }
            TryResult(result) => if operator.is_followed_by() {
                match right.get() {
                    // try { ... } catch
                    Ok(RightOperand(Catch, _)) => TryCatch(result).ok(),
                    Ok(RightOperand(Finally, _)) => TryFinally(result).ok(),
                    Ok(_) => TryWithoutCatchOrFinally.operand_err(LeftLeft),
                    Err(error) => error.err(),
                }
            } else {
                TryResult(result).lazy_val().infix(operator, right)
            }
            TryCatch(result) => if operator.is_followed_by() {
                match right.lazy_val() {
                    Ok(BergVal::BlockRef(block)) => {
                        let result = result.or_else(|exception| block.apply(exception.catch().into()));
                        CatchResult(result.evaluate()).ok()
                    }
                    Ok(_) => CatchBlockMustBeBlock.operand_err(Right),
                    Err(error) => error.err(),
                }
            } else {
                TryCatch(result).lazy_val().infix(operator, right)
            }
            CatchResult(result) => if operator.is_followed_by() {
                match right.get() {
                    // try { ... } catch { ... } finally
                    Ok(RightOperand(Finally, _)) => TryFinally(result).ok(),
                    Ok(_) => CatchWithoutFinally.operand_err(LeftLeft),
                    Err(error) => error.err(),
                }
            } else {
                CatchResult(result).lazy_val().infix(operator, right)
            }
            TryFinally(result) => if operator.is_followed_by() {
                match right.lazy_val()? {
                    BergVal::BlockRef(block) => {
                        block.evaluate()?;
                        result?.ok()
                    },
                    _ => FinallyBlockMustBeBlock.operand_err(Right),
                }
            } else {
                TryFinally(result).lazy_val().infix(operator, right)
            }
            Throw if operator.is_followed_by() => right.lazy_val()?.throw(),
            MissingExpression | TrailingSemicolon | TrailingComma(_) | If | Else | While | Foreach | Try | Catch | Finally | Throw => self.lazy_val().infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        match self {
            Val(v) => v.infix_assign(operator, right),
            Target(v) => v.infix_assign(operator, right),
            RawIdentifier(v) => v.infix_assign(operator, right),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().infix_assign(operator, right),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match self {
            Val(v) => v.prefix(operator),
            Target(v) => v.prefix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().prefix(operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match self {
            Val(v) => v.postfix(operator),
            Target(v) => v.postfix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().postfix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> where Self: Sized {
        use ExpressionBoundary::*;
        match self {
            Val(v) => v.subexpression_result(boundary),
            Target(v) => v.subexpression_result(boundary),
            RawIdentifier(v) => v.subexpression_result(boundary),
            MissingExpression if boundary == Parentheses || boundary.is_block() => empty_tuple().ok(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) | While | WhileCondition(_) | Foreach | ForeachInput(_) | Try | TryResult(_) | Catch | TryCatch(_) | CatchResult(_) | Finally | TryFinally(_) | Throw => self.lazy_val().subexpression_result(boundary),
        }
    }
}

fn run_while_loop<'a>(condition: BlockRef<'a>, block: BlockRef<'a>) -> EvalResult<'a> {
    use CompilerErrorCode::*;
    while condition.apply(empty_tuple()).into_native::<bool>().map_err(|e| e.reposition(LeftRight))? {
        // Run the block.
        match block.apply(empty_tuple()) {
            Ok(_) => {},
            Err(error) => match error.code() {
                Some(BreakOutsideLoop) => break,
                Some(ContinueOutsideLoop) => continue,
                // (while FOLLOWED_BY { condition }) FOLLOWED_BY { block } means block is right operand
                _ => return error.err()
            }
        }
    }
    empty_tuple().ok()
}

fn run_foreach<'a>(input: Result<BergVal<'a>, EvalException<'a>>, block: BlockRef<'a>) -> EvalResult<'a> {
    use CompilerErrorCode::*;
    let mut remaining = input?;
    while let NextVal { head: Some(value), tail } = remaining.next_val().map_err(|e| e.reposition(Right))? {
        remaining = tail;
        // Run the block.
        let result = block.apply(value);
        match result.lazy_val() {
            Ok(_) => {},
            Err(error) => match error.code() {
                Some(BreakOutsideLoop) => break,
                Some(ContinueOutsideLoop) => continue,
                // (while FOLLOWED_BY { condition }) FOLLOWED_BY { block } means block is right operand
                _ => return error.reposition(LeftRight).err()
            }
        }
    }
    empty_tuple().ok()
}

impl<'a> fmt::Display for EvalVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ConditionalState::*;
        match self {
            Val(v) => write!(f, "{}", v),
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
            ForeachInput(input) => write!(f, "foreach {}", input.display()),
            Try => write!(f, "try"),
            TryResult(r) => write!(f, "try -> {}", r.display()),
            Catch => write!(f, "catch"),
            TryCatch(r) => write!(f, "try catch -> {} catch", r.display()),
            CatchResult(r) => write!(f, "try catch ... -> {}", r.display()),
            Finally => write!(f, "finally"),
            TryFinally(r) => write!(f, "{} finally", r.display()),
            Throw => write!(f, "throw"),
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
            Ok(()) => empty_tuple().ok(),
            Err(error) => error.reposition(operand_position).err(),
        }
    }

    fn declare(&self) -> Result<(), EvalException<'a>> {
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

    fn set_internal(&mut self, value: BergVal<'a>) -> Result<(), EvalException<'a>> {
        use AssignmentTarget::*;
        let result = match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => 
                scope.set_local_field(*field, value, &scope.ast()),
            ObjectFieldReference(object, name) => {
                object.set_field(*name, value)
            }
        };
        self.point_errors_at_identifier(result)
    }

    fn point_errors_at_identifier<T>(&self, result: Result<T, EvalException<'a>>) -> Result<T, EvalException<'a>> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::*;
        match self {
            LocalFieldDeclaration(..) | ObjectFieldReference(..) => result.map_err(|e| e.reposition(Right)),
            LocalFieldReference(..) => result,
        }
    }
}

impl<'a> From<AssignmentTarget<'a>> for EvalVal<'a> {
    fn from(from: AssignmentTarget<'a>) -> Self {
        EvalVal::Target(from)
    }
}

impl<'a> Value<'a> for AssignmentTarget<'a> {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized {
        self.get().lazy_val()
    }
    fn eval_val(self) -> EvalResult<'a> where Self: Sized {
        self.ok()
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        self.get().into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        self.get().try_into_native()
    }
    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for AssignmentTarget<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        self.get().next_val()
    }
}

impl<'a> ObjectValue<'a> for AssignmentTarget<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        self.get().field(name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), EvalException<'a>> {
        let mut obj = self.get().lazy_val()?;
        obj.set_field(name, value)?;
        self.set(obj, Expression).and(Ok(()))
    }
}

impl<'a> OperableValue<'a> for AssignmentTarget<'a> {
    fn infix(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> {
        use AssignmentTarget::*;
        match (operator, &self) {
            // Handle <identifier>: <value>
            (COLON, LocalFieldReference(..)) => self.set(right.lazy_val()?, Left),
            _ => self.get().infix(operator, right)
        }
    }
    fn infix_assign(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> {
        match operator {
            EMPTY_STRING => self.set(right.lazy_val()?, Left),
            operator => self.set(self.get().infix(operator, right).lazy_val()?, Left),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        use AssignmentTarget::*;
        match (operator, self) {
            (COLON, LocalFieldReference(scope, field)) => LocalFieldDeclaration(scope, field).ok(),
            (PLUS_PLUS, mut right) => right.set(right.get().prefix(PLUS_ONE).lazy_val()?, Right),
            (DASH_DASH, mut right) => right.set(right.get().prefix(MINUS_ONE).lazy_val()?, Right),
            (_, right) => right.get().prefix(operator),
        }
    }

    fn postfix(mut self, operator: IdentifierIndex) -> EvalResult<'a> {
        match operator {
            PLUS_PLUS => self.set(self.get().postfix(PLUS_ONE).lazy_val()?, Left),
            DASH_DASH => self.set(self.get().postfix(MINUS_ONE).lazy_val()?, Left),
            _ => self.get().postfix(operator)
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> where Self: Sized {
        self.get().subexpression_result(boundary)
    }
}

impl<'a> fmt::Display for AssignmentTarget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AssignmentTarget::*;
        match self {
            LocalFieldReference(scope, field) => write!(f, "{}", scope.ast().identifier_string(scope.ast().fields[*field].name)),
            LocalFieldDeclaration(scope, field) => write!(f, "{}", scope.ast().identifier_string(scope.ast().fields[*field].name)),
            ObjectFieldReference(object, name) => write!(f, "{}.{}", object, name),
        }
    }
}
