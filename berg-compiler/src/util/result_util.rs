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
/// Convenience convert a Result into the return Result type.
/// 
pub trait ResShorthand: is_result::IsResult {
    ///
    /// Convert the Result into the return Result type.
    /// 
    fn res<T: From<Self::Ok>, E: From<Self::Err>>(self) -> Result<T, E> {
        match self.into_result() {
            Ok(ok) => Ok(ok.into()),
            Err(err) => Err(err.into()),
        }
    }
}

pub mod is_result {
    ///
    /// Trait implemented on all results so that we can implement anything we want
    /// on all results.
    /// 
    pub trait IsResult: Sized {
        type Ok;
        type Err;
        fn into_result(self) -> Result<Self::Ok, Self::Err>;
    }
    impl<Ok, Err> IsResult for Result<Ok, Err> {
        type Ok = Ok;
        type Err = Err;
        fn into_result(self) -> Result<Ok, Err> { self }
    }
}

impl<T, E> ResShorthand for Result<T, E> {}
impl<E, T: Into<E>> ErrShorthand<E> for T {}
impl<T, S: Into<T>> OkShorthand<T> for S {}
