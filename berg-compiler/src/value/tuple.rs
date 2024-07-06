use crate::value::implement::*;
use berg_parser::IdentifierIndex;
use std::fmt;
use std::iter::FromIterator;

///
/// A discrete series of values.
///
/// Note: Tuples are generally stored in *reverse* order, since the typical
/// operation for a tuple is to take the first value and return the next.
///
#[derive(Debug, Clone)]
pub struct Tuple(Vec<BergVal>);

impl Tuple {
    pub fn from_values(iter: impl DoubleEndedIterator<Item = BergVal>) -> Self {
        Self::from_reversed(iter.rev())
    }
    pub fn from_reversed(iter: impl Iterator<Item = BergVal>) -> Self {
        Tuple(iter.collect())
    }
    pub fn from_reversed_vec(vec: Vec<BergVal>) -> Self {
        Tuple(vec)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IntoIterator for Tuple {
    type Item = BergVal;
    type IntoIter = std::iter::Rev<<Vec<BergVal> as IntoIterator>::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().rev()
    }
}

impl FromIterator<BergVal> for Tuple {
    // Sadly, I don't think there is a way to specialize this for ExactSizeIterators.
    // So we have to build it the old fashioned way.
    fn from_iter<I: IntoIterator<Item = BergVal>>(iter: I) -> Self {
        Tuple::from(iter.into_iter().collect::<Vec<BergVal>>())
    }
}

impl From<Vec<BergVal>> for Tuple {
    fn from(mut from: Vec<BergVal>) -> Self {
        from.reverse();
        Tuple(from)
    }
}

impl<'p> IntoIterator for &'p Tuple {
    type Item = &'p BergVal;
    type IntoIter = std::iter::Rev<<&'p Vec<BergVal> as IntoIterator>::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().rev()
    }
}

impl BergValue for Tuple {}

impl EvaluatableValue for Tuple {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.ok()
    }
}

impl Value for Tuple {
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.ok()
    }
    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        self.ok()
    }
    fn into_native<T: TryFromBergVal>(mut self) -> Result<T, EvalException> {
        if self.0.len() == 1 {
            Ok(self.0.pop().unwrap().into_native()?)
        } else {
            CompilerError::BadOperandType(Box::new(BergVal::Tuple(self)), T::TYPE_NAME).err()
        }
    }
    fn try_into_native<T: TryFromBergVal>(mut self) -> Result<Option<T>, EvalException> {
        if self.0.len() == 1 {
            Ok(Some(self.0.pop().unwrap().into_native()?))
        } else {
            Ok(None)
        }
    }

    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl IteratorValue for Tuple {
    fn next_val(mut self) -> Result<NextVal, EvalException> {
        NextVal {
            head: self.0.pop(),
            tail: self.into(),
        }
        .ok()
    }
}

impl ObjectValue for Tuple {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_field(self, name)
    }
    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal,
    ) -> Result<(), EvalException> {
        default_set_field(self, name, value)
    }
}

impl OperableValue for Tuple {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        default_infix(self, operator, right)
    }
    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        default_infix_assign(self, operator, right)
    }
    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_postfix(self, operator)
    }
    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_prefix(self, operator)
    }
    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut is_first = true;
        for elem in self {
            if is_first {
                is_first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", elem)?;
        }
        write!(f, ")")
    }
}

impl From<Tuple> for BergVal {
    fn from(from: Tuple) -> Self {
        BergVal::Tuple(from)
    }
}

impl From<Tuple> for EvalVal {
    fn from(from: Tuple) -> Self {
        BergVal::from(from).into()
    }
}

impl TryFromBergVal for Tuple {
    const TYPE_NAME: &'static str = "Tuple";
    fn try_from_berg_val(
        from: EvalVal,
    ) -> Result<Result<Self, BergVal>, EvalException> {
        match from.lazy_val()? {
            BergVal::Tuple(value) => Ok(Ok(value)),
            value => Ok(Err(value)),
        }
    }
}

impl FromIterator<BergVal> for BergVal {
    // Sadly, it doesn't seem we can specialize this for the happy case where iter is an ExactSizeIterator.
    fn from_iter<I: IntoIterator<Item = BergVal>>(iter: I) -> Self {
        BergVal::Tuple(Tuple::from_iter(iter))
    }
}
impl From<Vec<BergVal>> for BergVal {
    fn from(from: Vec<BergVal>) -> Self {
        BergVal::Tuple(Tuple::from(from))
    }
}
impl From<Box<[BergVal]>> for BergVal {
    fn from(from: Box<[BergVal]>) -> Self {
        BergVal::from(from.into_vec())
    }
}

macro_rules! from_sized_array {
    ($($size:tt),*) => {
        $(
            impl From<[BergVal; $size]> for BergVal {
                fn from(from: [BergVal; $size]) -> Self {
                    // Put it in a box so we can convert to Vec.
                    let from: Box<[BergVal]> = Box::new(from);
                    BergVal::from(from.into_vec())
                }
            }
        )*
    }
}

from_sized_array! { 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31 }
