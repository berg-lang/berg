use value::{BergResult, BergVal};
use eval::BlockClosure;
use std::fmt;

#[derive(Clone)]
pub enum BergEval<'a> {
    BergVal(BergVal),
    BlockClosure(BlockClosure<'a>),
}

impl<'a, T: Into<BergVal>> From<T> for BergEval<'a> {
    fn from(from: T) -> Self {
        BergEval::BergVal(from.into())
    }
}

impl<'a> BergEval<'a> {
    pub fn evaluate(self) -> BergResult<'a> {
        match self {
            BergEval::BergVal(value) => Ok(value),
            BergEval::BlockClosure(closure) => closure.evaluate(),
        }
    }

    pub fn ok<E>(self) -> Result<BergEval<'a>, E> {
        Ok(self)
    }
}

impl<'a> fmt::Debug for BergEval<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BergVal(")?;
        match *self {
            BergEval::BergVal(ref value) => write!(f, "{:?}", value)?,
            BergEval::BlockClosure(ref closure) => write!(f, "{:?}", closure)?,
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BergEval<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BergEval::BergVal(ref value) => write!(f, "{}", value),
            BergEval::BlockClosure(ref closure) => write!(f, "{}", closure),
        }
    }
}
