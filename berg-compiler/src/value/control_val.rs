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
            ExpressionError(..) | Error(_) => self.result(),
        }
    }
    fn into_result(self) -> BergResult<'a> {
        Err(self)
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.into_native(),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.try_into_native(),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.infix(operator, right),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.infix_assign(operator, right),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.prefix(operator),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.postfix(operator),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.subexpression_result(boundary),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }

    fn into_right_operand(self) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.into_right_operand(),
            ExpressionError(error, ExpressionErrorPosition::Expression) => ExpressionError(error, ExpressionErrorPosition::RightOperand).err(),
            ExpressionError(error, position) => panic!("Expression error {:?} with position {:?} passed to into_right_operand", error, position),
            Error(_) => self.result(),
        }
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.field(name),
            ExpressionError(..) | Error(_) => self.result(),
        }
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        use ControlVal::*;
        match self {
            AmbiguousSyntax(val) => val.set_field(name, value),
            ExpressionError(..) | Error(_) => self.clone().result(),
        }
    }
}

impl<'a> ControlVal<'a> {
    pub fn result<T>(self) -> BergResult<'a, T> {
        Err(self)
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

