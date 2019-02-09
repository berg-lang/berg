use crate::syntax::ExpressionRef;
use crate::value::{BergError, BergVal, Error, EvalError};

pub type BergResult<'a, T = BergVal<'a>> = Result<T, Error<'a>>;
pub type EvalResult<'a, T = BergVal<'a>> = Result<T, EvalError<'a>>;

pub trait TakeError<'a, T>: Sized {
    fn take_error(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T>;
}

pub trait UnwindFrame<'a, T>: Sized {
    fn unwind_frame(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T>;
}

impl<'a, T> UnwindFrame<'a, T> for BergResult<'a, T> {
    fn unwind_frame(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.push_frame(expression.into())),
        }
    }
}
impl<'a, T> UnwindFrame<'a, T> for Error<'a> {
    fn unwind_frame(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        Err(self.push_frame(expression.into()))
    }
}

impl<'a, T> TakeError<'a, T> for EvalResult<'a, T> {
    fn take_error(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        match self {
            Ok(value) => Ok(value),
            Err(EvalError::Raw(raw)) => raw.take_error(expression),
            Err(EvalError::Error(error)) => error.unwind_frame(expression),
        }
    }
}
impl<'a, T> TakeError<'a, T> for BergError<'a> {
    fn take_error(self, location: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        Err(Error::new(self, location.into()))
    }
}
