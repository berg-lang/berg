use crate::value::implement::*;
use std::fmt;

pub type BergResult<'a, T = BergVal<'a>> = Result<T, ControlVal<'a>>;

// Convenience so we can do Display on BergResult. Grr.
pub trait DisplayAnyway {
    fn display(&self) -> DisplayAnywayVal<Self> {
        DisplayAnywayVal(self)
    }
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
pub struct DisplayAnywayVal<'p, T: DisplayAnyway+?Sized>(&'p T);
impl<'p, T: fmt::Display, E: fmt::Display> DisplayAnyway for Result<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ok(value) => write!(f, "{}", value),
            Err(error) => write!(f, "{}", error),
        }
    }
}
impl<'p, T: DisplayAnyway> fmt::Display for DisplayAnywayVal<'p, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        DisplayAnyway::fmt(self.0, f)
    }
}

impl<'a, V: BergValue<'a>+Clone, E: BergValue<'a>+Clone> BergValue<'a> for Result<V, E> {
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        match self {
            Ok(value) => value.next_val(),
            Err(control) => control.next_val(),
        }
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        match self {
            Ok(value) => value.into_native(),
            Err(control) => control.into_native()
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        match self {
            Ok(value) => value.try_into_native(),
            Err(control) => control.try_into_native()
        }
    }

    fn into_val(self) -> BergResult<'a> {
        match self {
            Ok(value) => value.into_val(),
            Err(control) => control.into_val()
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        match self {
            Ok(value) => value.infix(operator, right),
            Err(control) => control.infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        match self {
            Ok(value) => value.infix_assign(operator, right),
            Err(control) => control.infix_assign(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        match self {
            Ok(value) => value.postfix(operator),
            Err(control) => control.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        match self {
            Ok(value) => value.prefix(operator),
            Err(control) => control.prefix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        match self {
            Ok(value) => value.subexpression_result(boundary),
            Err(control) => control.subexpression_result(boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a, BergResult<'a>> {
        match self {
            Ok(v) => v.field(name),
            Err(v) => v.field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> where Self: Clone {
        match self {
            Ok(v) => v.set_field(name, value),
            Err(v) => v.set_field(name, value),
        }
    }
}
