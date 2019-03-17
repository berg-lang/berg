use std::fmt;

/// Convenience so we can do Display on BergResult.
pub trait DisplayResult {
    fn display(&self) -> DisplayResultVal<Self> {
        DisplayResultVal(self)
    }
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
pub struct DisplayResultVal<'p, T: DisplayResult+?Sized>(&'p T);

///
/// Convenience to say X.err(), which chains better than Err(X)
/// 
pub trait ErrShorthand<E>: Into<E> {
    ///
    /// Return `Err(self.into())`
    /// 
    fn err<T>(self) -> Result<T, E> {
        Err(self.into())        
    }
}

///
/// Convenience to say X.ok(), which chains better than Ok(X)
/// 
pub trait OkShorthand<T>: Into<T> {
    ///
    /// Return `Ok(value.into())`
    /// 
    fn ok<E>(self) -> Result<T, E> {
        Ok(self.into())
    }
}

impl<E, T: Into<E>> ErrShorthand<E> for T {
}

impl<T, S: Into<T>> OkShorthand<T> for S {
}

impl<'p, T: fmt::Display, E: fmt::Display> DisplayResult for Result<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ok(value) => write!(f, "{}", value),
            Err(error) => write!(f, "{}", error),
        }
    }
}
impl<'p, T: DisplayResult> fmt::Display for DisplayResultVal<'p, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        DisplayResult::fmt(self.0, f)
    }
}
