use crate::eval::AssignmentTarget;
use crate::syntax::IdentifierIndex;
use crate::syntax::identifiers::{SEMICOLON, COMMA};
use crate::value::implement::*;
use AmbiguousSyntax::*;
use ExpressionErrorPosition::*;
use crate::value::RightOperand;

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
    fn into_val(self) -> BergResult<'a> {
        self.err()
    }
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        self.disambiguate().next_val()
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.disambiguate().into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.disambiguate().try_into_native()
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        match self {
            Target(v) => v.infix(operator, right),
            RawIdentifier(v) => v.infix(operator, right),
            TrailingSemicolon if operator == SEMICOLON => BergError::MissingOperand.operand_err(LeftRight),
            TrailingComma(_) if operator == COMMA => BergError::MissingOperand.operand_err(LeftRight),
            PartialTuple(mut vec) => match operator {
                COMMA => { vec.push(right.into_val()?); PartialTuple(vec).disambiguate() },
                _ => PartialTuple(vec).disambiguate().infix(operator, right),
            }
            MissingExpression | TrailingSemicolon | TrailingComma(_) => self.disambiguate().infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        match self {
            Target(v) => v.infix_assign(operator, right),
            RawIdentifier(v) => v.infix_assign(operator, right),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.disambiguate().infix_assign(operator, right),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        match self {
            Target(v) => v.prefix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.disambiguate().prefix(operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        match self {
            Target(v) => v.postfix(operator),
            RawIdentifier(v) => v.prefix(operator),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.disambiguate().postfix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        use ExpressionBoundary::*;
        match self {
            Target(v) => v.subexpression_result(boundary),
            RawIdentifier(v) => v.subexpression_result(boundary),
            MissingExpression if boundary == Parentheses || boundary == CurlyBraces => BergVal::empty_tuple().ok(),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.disambiguate().subexpression_result(boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        match self {
            Target(v) => v.field(name),
            RawIdentifier(v) => v.field(name),
            MissingExpression | PartialTuple(_) | TrailingComma(_) | TrailingSemicolon => self.disambiguate().field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
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
    pub fn disambiguate(self) -> BergResult<'a> {
        use AmbiguousSyntax::*;
        match self {
            MissingExpression => BergError::MissingOperand.err(),
            PartialTuple(vec) | TrailingComma(vec) => Tuple::from(vec).ok(),
            TrailingSemicolon => BergVal::empty_tuple().ok(),
            RawIdentifier(v) => v.into_val(),
            Target(v) => v.get(),
        }
    }
}
