use crate::syntax::ExpressionRef;
use crate::value::*;

pub type BergResult<'a, T = BergVal<'a>> = Result<T, ControlVal<'a>>;

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
            Err(control) => Err(control.push_frame(expression.into())),
        }
    }
}
impl<'a, T> UnwindFrame<'a, T> for Error<'a> {
    fn unwind_frame(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        Err(self.push_frame(expression.into()).into())
    }
}
impl<'a, T> UnwindFrame<'a, T> for ControlVal<'a> {
    fn unwind_frame(self, location: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        Err(self.push_frame(location.into()))
    }
}
impl<'a, T> TakeError<'a, T> for BergResult<'a, T> {
    fn take_error(self, expression: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        match self {
            Ok(value) => Ok(value),
            Err(control) => control.take_error(expression),
        }
    }
}
impl<'a, T> TakeError<'a, T> for BergError<'a> {
    fn take_error(self, location: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        Err(Error::new(self, location.into()).into())
    }
}
impl<'a, T> TakeError<'a, T> for ControlVal<'a> {
    fn take_error(self, location: impl Into<ExpressionRef<'a>>) -> BergResult<'a, T> {
        Err(self.push_frame(location.into()))
    }
}