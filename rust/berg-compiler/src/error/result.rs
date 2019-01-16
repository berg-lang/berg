use crate::error::{BergError, Error, EvalError};
use crate::eval::Expression;
use crate::syntax::AstRef;
use crate::value::BergVal;

pub type BergResult<'a, T = BergVal<'a>> = Result<T, Error<'a>>;
pub type EvalResult<'a, T = BergVal<'a>> = Result<T, EvalError<'a>>;

pub trait TakeError<'a, T, Expr>: Sized {
    fn take_error(self, ast: &AstRef<'a>, expression: Expr) -> BergResult<'a, T>;
}

pub trait UnwindFrame<'a, T>: Sized {
    fn unwind_frame(self, ast: &AstRef<'a>, expression: Expression) -> BergResult<'a, T>;
}

impl<'a, T> UnwindFrame<'a, T> for BergResult<'a, T> {
    fn unwind_frame(self, ast: &AstRef<'a>, expression: Expression) -> BergResult<'a, T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.push_frame(ast, expression)),
        }
    }
}
impl<'a, T> UnwindFrame<'a, T> for Error<'a> {
    fn unwind_frame(self, ast: &AstRef<'a>, expression: Expression) -> BergResult<'a, T> {
        Err(self.push_frame(ast, expression))
    }
}

impl<'a, T> TakeError<'a, T, Expression> for EvalResult<'a, T> {
    fn take_error(self, ast: &AstRef<'a>, expression: Expression) -> BergResult<'a, T> {
        match self {
            Ok(value) => Ok(value),
            Err(EvalError::Raw(raw)) => raw.take_error(ast, expression),
            Err(EvalError::Error(error)) => error.unwind_frame(ast, expression),
        }
    }
}
impl<'a, T> TakeError<'a, T, Expression> for BergError<'a> {
    fn take_error(self, ast: &AstRef<'a>, expression: Expression) -> BergResult<'a, T> {
        Err(Error::new(self, ast, expression))
    }
}
