use crate::eval::AmbiguousSyntax;
use crate::syntax::IdentifierIndex;
use crate::value::*;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ControlVal<'a> {
    AmbiguousSyntax(AmbiguousSyntax<'a>),
    ExpressionError(BergError<'a>, ExpressionErrorPosition),
    Error(Error<'a>),
}

impl<'a> BergValue<'a> for ControlVal<'a> {
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.next_val(),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::Expression),
        }
    }
    fn into_val(self) -> BergResult<'a> {
        Err(self)
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.into_native(),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::Expression),
        }
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.try_into_native(),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::Expression),
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.infix(operator, right),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::LeftOperand),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.infix_assign(operator, right),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::LeftOperand),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.prefix(operator),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::RightOperand),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.postfix(operator),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::LeftOperand),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.subexpression_result(boundary),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::RightOperand),
        }
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.field(name),
            ExpressionError(..) | Error(_) => self.at_position(ExpressionErrorPosition::LeftOperand),
        }
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> BergResult<'a, ()> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.set_field(name, value),
            ExpressionError(..) | Error(_) => self.clone().at_position(ExpressionErrorPosition::LeftOperand),
        }
    }
}

impl<'a> ControlVal<'a> {
    pub fn at_position<T>(self, new_position: ExpressionErrorPosition) -> BergResult<'a, T> {
        use ControlVal::*;
        use ExpressionErrorPosition::*;
        match self {
            ExpressionError(error, position) => match (new_position, position) {
                (new_position, Expression) => ExpressionError(error, new_position).err(),
                (Expression, position) => ExpressionError(error, position).err(),
                (LeftOperand, LeftOperand) => ExpressionError(error, LeftLeft).err(),
                (LeftOperand, RightOperand) => ExpressionError(error, LeftRight).err(),
                (RightOperand, LeftOperand) => ExpressionError(error, RightLeft).err(),
                (RightOperand, RightOperand) => ExpressionError(error, RightRight).err(),
                _ => unreachable!("{:?} {:?} at {:?}", error, position, new_position),
            }
            _ => self.err(),
        }
    }
    pub fn disambiguate(self) -> BergResult<'a> {
        match self {
            ControlVal::AmbiguousSyntax(syntax) => match syntax.disambiguate() {
                Err(ControlVal::AmbiguousSyntax(_)) => unreachable!(),
                result => result,
            }
            result => result.err(),
        }
    }
    pub fn disambiguate_operand(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        match self {
            ControlVal::AmbiguousSyntax(syntax) => syntax.disambiguate_operand(new_position),
            _ => self.at_position(new_position)
        }
    }
}

impl<'a> From<Error<'a>> for ControlVal<'a> {
    fn from(from: Error<'a>) -> Self {
        ControlVal::Error(from)
    }
}

impl<'a> fmt::Display for ControlVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(target) => write!(f, "{:?}", target),
            ExpressionError(error, position) => write!(f, "{:?} at {:?}", error, position),
            Error(error) => write!(f, "{}", error),
        }
    }
}

