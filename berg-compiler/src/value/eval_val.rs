use crate::eval::ScopeRef;
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::syntax::identifiers::*;
use crate::value::implement::*;
use std::fmt;
use EvalVal::*;

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

#[derive(Debug, Clone)]
pub enum EvalVal<'a> {
    /// non-syntactical results (integers, booleans, results of math expressions ...)
    Value(BergVal<'a>),
    /// if
    If,
    /// else
    Else,
    /// if, else evaluation
    ConditionalVal(ConditionalState, Option<BergVal<'a>>),
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

impl<'a> EvalVal<'a> {
    //
    // If this is a reference to something else, resolve it (it might be a syntax
    // value like Else or If).
    //
    pub fn get(self) -> EvalResult<'a> {
        use EvalVal::*;
        match self {
            Target(v) => v.get(),
            Value(_) | If | Else | ConditionalVal(..) | MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | RawIdentifier(_) => self.ok(),
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
            If => IfWithoutCondition.err(),
            ConditionalVal(IfCondition, _) => IfWithoutCondition.operand_err(ExpressionErrorPosition::RightOperand),
            Else => BergError::ElseWithoutIf.err(),
            ConditionalVal(ElseBlock, _) => ElseWithoutCode.err(),
            ConditionalVal(RunBlock, _) | ConditionalVal(IgnoreBlock, _) => IfWithoutCode.err(),
            ConditionalVal(MaybeElse, None) => BergVal::empty_tuple().ok(),
            ConditionalVal(MaybeElse, Some(v)) => v.ok(),
            MissingExpression => MissingOperand.err(),
            PartialTuple(vec) | TrailingComma(vec) => Tuple::from(vec).ok(),
            TrailingSemicolon => BergVal::empty_tuple().ok(),
            RawIdentifier(v) => v.into_val(),
            Target(v) => v.into_val(),
        }
    }
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.into_val().at_position(new_position)
    }
    fn eval_val(self) -> EvalResult<'a> {
        self.ok()
    }
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        match self {
            Value(v) => v.next_val(),
            Target(v) => v.next_val(),
            RawIdentifier(v) => v.next_val(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val()?.next_val(),
        }
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        match self {
            Value(v) => v.into_native(),
            Target(v) => v.into_native(),
            RawIdentifier(v) => v.into_native(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val()?.into_native(),
        }
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        match self {
            Value(v) => v.try_into_native(),
            Target(v) => v.try_into_native(),
            RawIdentifier(v) => v.try_into_native(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val()?.try_into_native(),
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use ExpressionErrorPosition::LeftRight;
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
            If | ConditionalVal(IfCondition, None) if operator == APPLY => {
                if right.into_native::<bool>()? {
                    ConditionalVal(RunBlock, None).ok()
                } else {
                    ConditionalVal(IgnoreBlock, None).ok()
                }
            }
            ConditionalVal(state, result) => {
                if operator == APPLY {
                    match state {
                        IfCondition => match right.get()? {
                            // if if
                            RightOperand(If, _) => IfWithoutCondition.operand_err(ExpressionErrorPosition::LeftOperand),
                            // if else
                            RightOperand(Else, _) => IfWithoutCondition.operand_err(ExpressionErrorPosition::LeftOperand),
                            // if <condition>
                            right => if result.is_none() && right.into_native::<bool>()? {
                                ConditionalVal(RunBlock, result).ok()
                            } else {
                                ConditionalVal(IgnoreBlock, result).ok()
                            }
                        }
                        // if true <block>
                        // if false {} else <block>
                        RunBlock if result.is_none() => ConditionalVal(MaybeElse, Some(right.into_val()?)).ok(),
                        RunBlock => unreachable!(),
                        IgnoreBlock => match right.get()? {
                            // if false if
                            RightOperand(If, _) => IfWithoutCode.operand_err(ExpressionErrorPosition::LeftOperand),
                            // if false else
                            RightOperand(Else, _) => IfWithoutCode.operand_err(ExpressionErrorPosition::LeftOperand),
                            // if false <block>
                            _ => ConditionalVal(MaybeElse, result).ok(),
                        }
                        // else ^
                        ElseBlock => match right.get()? {
                            // else if
                            RightOperand(If, _) => ConditionalVal(IfCondition, result).ok(),
                            // else else
                            RightOperand(Else, _) => ElseWithoutCode.operand_err(ExpressionErrorPosition::LeftOperand),
                            right => match result {
                                // else <block>
                                None => right.into_val()?.ok(),
                                // if true {} else <block>
                                Some(val) => val.ok(),
                            },
                        }
                        // if true|false {} <something>
                        MaybeElse => match right.get()? {
                            // if true {} else
                            RightOperand(Else, _) => ConditionalVal(ElseBlock, result).ok(),
                            // if true {} if
                            // if true {} 1
                            _ => IfWithoutElse.err()
                        }
                    }
                } else {
                    ConditionalVal(state, result).into_val().infix(operator, right)
                }
            }
            MissingExpression | TrailingSemicolon | TrailingComma(_) | If | Else => self.into_val().infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        match self {
            Value(v) => v.infix_assign(operator, right),
            Target(v) => v.infix_assign(operator, right),
            RawIdentifier(v) => v.infix_assign(operator, right),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val().infix_assign(operator, right),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Value(v) => v.prefix(operator),
            Target(v) => v.prefix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val().prefix(operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Value(v) => v.postfix(operator),
            Target(v) => v.postfix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val().postfix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        use ExpressionBoundary::*;
        match self {
            Value(v) => v.subexpression_result(boundary),
            Target(v) => v.subexpression_result(boundary),
            RawIdentifier(v) => v.subexpression_result(boundary),
            MissingExpression if boundary == Parentheses || boundary.is_block() => BergVal::empty_tuple().ok(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val().subexpression_result(boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Value(v) => v.field(name),
            Target(v) => v.field(name),
            RawIdentifier(v) => v.field(name),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => self.into_val().field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        match self {
            Value(v) => v.set_field(name, value),
            Target(v) => v.set_field(name, value),
            RawIdentifier(v) => v.set_field(name, value),
            MissingExpression => BergError::MissingOperand.err(),
            PartialTuple(_) | TrailingComma(_) | TrailingSemicolon | If | Else | ConditionalVal(..) => panic!("not yet implemented: can't set field {} on {:?} to {}", name, self, value),
        }
    }
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
                LocalFieldDeclaration(..) | ObjectFieldReference(..) => error.operand_err(RightOperand),
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
        use ExpressionErrorPosition::LeftOperand;
        match (operator, &self) {
            // Handle <identifier>: <value>
            (COLON, LocalFieldReference(..)) => self.set(right.into_val()?, LeftOperand),
            _ => self.get().infix(operator, right)
        }
    }
    fn infix_assign(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use ExpressionErrorPosition::LeftOperand;
        match operator {
            EMPTY_STRING => self.set(right.into_val()?, LeftOperand),
            operator => self.set(self.get().infix(operator, right).into_val()?, LeftOperand),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::RightOperand;
        match (operator, self) {
            (COLON, LocalFieldReference(scope, field)) => LocalFieldDeclaration(scope, field).ok(),
            (PLUS_PLUS, mut right) => right.set(right.get().prefix(PLUS_ONE).into_val()?, RightOperand),
            (DASH_DASH, mut right) => right.set(right.get().prefix(MINUS_ONE).into_val()?, RightOperand),
            (_, right) => right.get().prefix(operator),
        }
    }

    fn postfix(mut self, operator: IdentifierIndex) -> EvalResult<'a> {
        use ExpressionErrorPosition::LeftOperand;
        match operator {
            PLUS_PLUS => self.set(self.get().postfix(PLUS_ONE).into_val()?, LeftOperand),
            DASH_DASH => self.set(self.get().postfix(MINUS_ONE).into_val()?, LeftOperand),
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
