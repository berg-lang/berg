use crate::eval::AssignmentTarget;
use crate::syntax::IdentifierIndex;
use crate::syntax::identifiers::{SEMICOLON, COMMA};
use crate::value::implement::*;

#[derive(Debug, Clone)]
pub enum AmbiguousSyntax<'a> {
    MissingExpression,
    PartialTuple(Vec<BergVal<'a>>),
    TrailingComma(Vec<BergVal<'a>>),
    TrailingSemicolon,
    RawIdentifier(IdentifierIndex),
    Target(AssignmentTarget<'a>),
}

impl<'a> BergValue<'a> for AmbiguousSyntax<'a> {
    fn into_result(self) -> BergResult<'a> {
        self.err()
    }
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        self.delocalize(ExpressionErrorPosition::Expression).next_val()
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.delocalize(ExpressionErrorPosition::Expression).into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.delocalize(ExpressionErrorPosition::Expression).try_into_native()
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.infix(operator, right),
            RawIdentifier(v) => v.infix(operator, right),
            TrailingSemicolon if operator == SEMICOLON => BergError::MissingOperand.operand_err(ExpressionErrorPosition::ImmediateLeftOperand),
            TrailingComma(_) if operator == COMMA => BergError::MissingOperand.operand_err(ExpressionErrorPosition::ImmediateLeftOperand),
            PartialTuple(mut vec) => match operator {
                COMMA => { vec.push(right.into_result()?); PartialTuple(vec).into_result() },
                _ => PartialTuple(vec).delocalize(ExpressionErrorPosition::LeftOperand).infix(operator, right),
            }
            MissingExpression | TrailingSemicolon | TrailingComma(_) => self.delocalize(ExpressionErrorPosition::LeftOperand).infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.infix_assign(operator, right),
            RawIdentifier(v) => v.infix_assign(operator, right),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.delocalize(ExpressionErrorPosition::LeftOperand).infix_assign(operator, right),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.prefix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.delocalize(ExpressionErrorPosition::RightOperand).prefix(operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.postfix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.delocalize(ExpressionErrorPosition::LeftOperand).postfix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        use ExpressionBoundary::*;
        match self {
            Target(v) => v.subexpression_result(boundary),
            RawIdentifier(v) => v.subexpression_result(boundary),
            MissingExpression if boundary == Parentheses || boundary == CurlyBraces => BergVal::empty_tuple().ok(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.delocalize(ExpressionErrorPosition::Expression).subexpression_result(boundary),
        }
    }

    fn into_right_operand(self) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.into_right_operand(),
            RawIdentifier(v) => v.into_right_operand(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.delocalize(ExpressionErrorPosition::Expression).into_right_operand(),
        }
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.field(name),
            RawIdentifier(v) => v.field(name),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.delocalize(ExpressionErrorPosition::Expression).field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        use AmbiguousSyntax::*;
        match self {
            Target(v) => v.set_field(name, value),
            RawIdentifier(v) => v.set_field(name, value),
            MissingExpression => BergError::MissingOperand.err(),
            PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => panic!("not yet implemented: can't set field {} on {:?} to {}", name, self, value.display()),
        }
    }
}

impl<'a> From<AmbiguousSyntax<'a>> for ControlVal<'a> {
    fn from(from: AmbiguousSyntax<'a>) -> Self {
        ControlVal::AmbiguousSyntax(from)
    }
}

impl<'a> AmbiguousSyntax<'a> {
    pub fn result<T>(self) -> BergResult<'a, T> {
        Err(ControlVal::AmbiguousSyntax(self))
    }
    fn delocalize(self, self_position: ExpressionErrorPosition) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            MissingExpression => BergError::MissingOperand.operand_err(self_position),
            PartialTuple(vec) | TrailingComma(vec) => Tuple::from(vec).ok(),
            TrailingSemicolon => BergVal::empty_tuple().ok(),
            RawIdentifier(v) => v.into_result(),
            Target(v) => v.get(),
        }
    }
}
