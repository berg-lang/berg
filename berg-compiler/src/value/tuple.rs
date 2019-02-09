use crate::syntax::IdentifierIndex;
use crate::util::type_name::TypeName;
use crate::value::*;
use std::iter::FromIterator;

///
/// A discrete series of values.
///
/// Note: Tuples are generally stored in *reverse* order, since the typical
/// operation for a tuple is to take the first value and return the next.
///
#[derive(Debug, Clone)]
pub struct Tuple<'a>(Vec<BergVal<'a>>);

impl<'a> Tuple<'a> {
    pub fn from_values(iter: impl DoubleEndedIterator<Item = BergVal<'a>>) -> Self {
        Self::from_reversed(iter.rev())
    }
    pub fn from_reversed(iter: impl Iterator<Item = BergVal<'a>>) -> Self {
        Tuple(iter.collect())
    }
    pub fn from_reversed_vec(vec: Vec<BergVal<'a>>) -> Self {
        Tuple(vec)
    }
}

impl<'a> IntoIterator for Tuple<'a> {
    type Item = BergVal<'a>;
    type IntoIter = std::iter::Rev<<Vec<BergVal<'a>> as IntoIterator>::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().rev()
    }
}

impl<'a> FromIterator<BergVal<'a>> for Tuple<'a> {
    // Sadly, I don't think there is a way to specialize this for ExactSizeIterators.
    // So we have to build it the old fashioned way.
    fn from_iter<I: IntoIterator<Item = BergVal<'a>>>(iter: I) -> Self {
        Tuple::from(iter.into_iter().collect::<Vec<BergVal<'a>>>())
    }
}

impl<'a> From<Vec<BergVal<'a>>> for Tuple<'a> {
    fn from(mut from: Vec<BergVal<'a>>) -> Self {
        from.reverse();
        Tuple(from)
    }
}

impl<'a, 'p> IntoIterator for &'p Tuple<'a> {
    type Item = &'p BergVal<'a>;
    type IntoIter = std::iter::Rev<<&'p Vec<BergVal<'a>> as IntoIterator>::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).iter().rev()
    }
}
impl<'a> TypeName for Tuple<'a> {
    const TYPE_NAME: &'static str = "Tuple";
}

impl<'a> BergValue<'a> for Tuple<'a> {
    fn infix<T: BergValue<'a>>(self, operator: IdentifierIndex, right: T) -> EvalResult<'a> {
        default_infix(self, operator, right)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        default_postfix(self, operator)
    }
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        default_prefix(self, operator)
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        default_set_field(self, name, value)
    }

    fn next_val(mut self) -> BergResult<'a, NextVal<'a>> {
        match self.0.pop() {
            Some(value) => Ok(if self.0.is_empty() {
                NextVal::single(value)
            } else {
                NextVal::head_tail(value, self.0.into())
            }),
            None => Ok(NextVal::none()),
        }
    }
    fn into_val(self) -> BergResult<'a> {
        Ok(self.into())
    }
    fn into_native<T: TypeName + TryFrom<BergVal<'a>>>(
        mut self,
    ) -> BergResult<'a, EvalResult<'a, T>>
    where
        <T as TryFrom<BergVal<'a>>>::Error: Into<BergVal<'a>>,
    {
        if self.0.len() == 1 {
            self.0.pop().unwrap().into_native()
        } else {
            Ok(BergError::BadType(Box::new(self.into()), T::TYPE_NAME).err())
        }
    }
}

impl<'a> fmt::Display for Tuple<'a> {
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

impl<'a> From<Tuple<'a>> for BergVal<'a> {
    fn from(value: Tuple<'a>) -> Self {
        BergVal::Tuple(value)
    }
}

impl<'a> From<Tuple<'a>> for Vec<BergVal<'a>> {
    fn from(value: Tuple<'a>) -> Self {
        value.0
    }
}

impl<T: TypeName> TypeName for Vec<T> {
    const TYPE_NAME: &'static str = "Vec<T>";
}

impl<T: TypeName> TypeName for &[T] {
    const TYPE_NAME: &'static str = "[T]";
}

impl<'a> TryFrom<BergVal<'a>> for Tuple<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Tuple(tuple) => Ok(tuple),
            _ => Err(from),
        }
    }
}

impl<'a> FromIterator<BergVal<'a>> for BergVal<'a> {
    // Sadly, it doesn't seem we can specialize this for the happy case where iter is an ExactSizeIterator.
    fn from_iter<I: IntoIterator<Item = BergVal<'a>>>(iter: I) -> Self {
        BergVal::Tuple(Tuple::from_iter(iter))
    }
}
impl<'a> From<Vec<BergVal<'a>>> for BergVal<'a> {
    fn from(from: Vec<BergVal<'a>>) -> Self {
        BergVal::Tuple(Tuple::from(from))
    }
}
impl<'a> From<Box<[BergVal<'a>]>> for BergVal<'a> {
    fn from(from: Box<[BergVal<'a>]>) -> Self {
        BergVal::from(from.into_vec())
    }
}

macro_rules! from_sized_array {
    ($($size:tt),*) => {
        $(
            impl<'a> From<[BergVal<'a>; $size]> for BergVal<'a> {
                fn from(from: [BergVal<'a>; $size]) -> Self {
                    // Put it in a box so we can convert to Vec.
                    let from: Box<[BergVal<'a>]> = Box::new(from);
                    BergVal::from(from.into_vec())
                }
            }
        )*
    }
}

from_sized_array! { 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31 }
