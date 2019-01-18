use crate::error::{BergResult, EvalResult};
use crate::eval::ScopeRef;
use crate::syntax::{AstRef, IdentifierIndex, Operand};
use crate::util::type_name::TypeName;
use crate::value::*;
use std::iter;

#[derive(Debug, Clone)]
pub struct Tuple<'a>(Vec<BergVal<'a>>);

impl<'a> Tuple<'a> {
    pub fn new(vec: Vec<BergVal<'a>>) -> Self {
        assert!(vec.len() >= 2);
        Tuple(vec)
    }
}

impl<'a> TypeName for Tuple<'a> {
    const TYPE_NAME: &'static str = "Tuple";
}

impl<'a> BergValue<'a> for Tuple<'a> {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        match operator {
            // EQUAL_TO => {
            //     let right = right.evaluate_to::<Tuple<'a>>(scope, ast)?;
            //     (
            //         self.0.len() == right.0.len() &&
            //         self.0.iter().zip(right.0).all(|(left, right)| left.infix(EQUAL_TO, right)?.downcast<bool>())
            //     ).ok()
            // },
            _ => default_infix(self, operator, scope, right, ast),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_postfix(self, operator, scope)
    }
    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_prefix(self, operator, scope)
    }

    // Evaluation: values which need further work to resolve, like blocks, implement this.
    fn result(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        default_result(self, scope)
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        default_set_field(self, name, value)
    }
}

impl<'a> fmt::Display for Tuple<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut is_first = true;
        for elem in &self.0 {
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

impl<'a, T: Clone> From<&[T]> for BergVal<'a>
where
    BergVal<'a>: From<T>,
{
    fn from(value: &[T]) -> Self {
        From::from(Vec::from(value))
    }
}

impl<'a, T> From<Vec<T>> for BergVal<'a>
where
    BergVal<'a>: From<T>,
{
    fn from(mut value: Vec<T>) -> Self {
        match value.len() {
            0 => BergVal::Nothing,
            1 => match value.pop() {
                Some(value) => BergVal::from(value),
                None => unreachable!(),
            },
            _ => {
                let vec = value.drain(..).map(BergVal::from).collect();
                BergVal::Tuple(Tuple::new(vec))
            }
        }
    }
}

impl<'a> From<Tuple<'a>> for BergVal<'a> {
    fn from(value: Tuple<'a>) -> Self {
        BergVal::Tuple(value)
    }
}

impl<T: TypeName> TypeName for Vec<T> {
    const TYPE_NAME: &'static str = "Vec<T>";
}

impl<T: TypeName> TypeName for &[T] {
    const TYPE_NAME: &'static str = "[T]";
}

impl<'a, T: TryFrom<BergVal<'a>, Error = BergVal<'a>>> TryFrom<BergVal<'a>> for Vec<T>
where
    BergVal<'a>: From<T>,
{
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Nothing => Ok(vec![]),
            BergVal::Tuple(Tuple(mut vec)) => {
                let mut converted: Self = Vec::with_capacity(vec.len());

                // Convert all the elements with TryFrom
                let mut elements = vec.drain(..).map(T::try_from);

                // Create the result from that
                let mut next = elements.next();
                while let Some(Ok(to)) = next {
                    converted.push(to);
                    next = elements.next();
                }

                match next {
                    None => Ok(converted),
                    Some(Ok(_)) => unreachable!(),
                    // If we had any errors, convert back.
                    Some(Err(orig)) => {
                        // Convert back what we already put into the vector ...
                        let originals: Vec<BergVal<'a>> = converted
                            .drain(..)
                            .map(BergVal::from)
                            // Then append the error we found ...
                            .chain(iter::once(orig))
                            // Then convert back what remains.
                            .chain(elements.map(|value| match value {
                                Ok(to) => BergVal::from(to),
                                Err(orig) => orig,
                            }))
                            .collect();
                        Err(originals.into())
                    }
                }
            }
            value => match T::try_from(value) {
                Ok(to) => Ok(vec![to]),
                Err(orig) => Err(orig),
            },
        }
    }
}

impl<'a> TryFrom<BergVal<'a>> for Vec<BergVal<'a>> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Nothing => Ok(vec![]),
            BergVal::Tuple(Tuple(vec)) => Ok(vec),
            value => Ok(vec![value]),
        }
    }
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
