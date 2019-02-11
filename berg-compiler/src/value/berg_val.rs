use crate::eval::BlockRef;
use crate::value::*;
use num::BigRational;
use std::fmt;

///
/// A concrete type that can hold any `BergValue`, and delegates operations to the concrete type.
///
#[derive(Clone)]
pub enum BergVal<'a> {
    Nothing,
    Boolean(bool),
    BigRational(BigRational),
    BlockRef(BlockRef<'a>),
    // Error(Box<Error<'a>>),
    Identifier(IdentifierIndex),
    Tuple(Tuple<'a>),
}

#[derive(Debug, Clone)]
pub struct NextVal<'a>(pub Option<(BergVal<'a>, Option<BergVal<'a>>)>);

impl<'a> fmt::Display for NextVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            None => write!(f, "<nothing>"),
            Some((head, None)) => write!(f, "{}", head),
            Some((head, Some(tail))) => write!(f, "{} -> {}", head, tail),
        }
    }
}
impl<'a> NextVal<'a> {
    pub fn head(&self) -> Option<&BergVal<'a>> {
        match self.0 {
            Some((ref head, _)) => Some(head),
            None => None,
        }
    }
    pub fn tail(&self) -> Option<&BergVal<'a>> {
        match self.0 {
            Some((_, Some(ref tail))) => Some(tail),
            _ => None,
        }
    }
    pub fn has_tail(&self) -> bool {
        self.tail().is_some()
    }
    pub fn has_value(&self) -> bool {
        self.head().is_some()
    }
    pub fn into_single(self) -> Option<BergVal<'a>> {
        match self.0 {
            Some((head, None)) => Some(head),
            _ => None,
        }
    }
    pub fn into_head_tail(self) -> (Option<BergVal<'a>>, Option<BergVal<'a>>) {
        match self.0 {
            None => (None, None),
            Some((head, None)) => (Some(head), None),
            Some((head, Some(tail))) => (Some(head), Some(tail)),
        }
    }
    pub fn none() -> NextVal<'a> {
        NextVal(None)
    }
    pub fn single(value: BergVal<'a>) -> NextVal<'a> {
        assert!(match value {
            BergVal::Nothing => false,
            _ => true,
        });
        NextVal(Some((value, None)))
    }
    pub fn head_tail(head: BergVal<'a>, tail: BergVal<'a>) -> NextVal<'a> {
        assert!(match head {
            BergVal::Nothing => false,
            _ => true,
        });
        assert!(match tail {
            BergVal::Nothing => false,
            _ => true,
        });
        NextVal(Some((head, Some(tail))))
    }
}

impl<'a> BergVal<'a> {
    pub fn empty_tuple() -> BergVal<'a> {
        BergVal::from(vec![])
    }
}

impl<'a> BergValue<'a> for BergVal<'a> {
    fn next_val(self) -> BergResult<'a, NextVal<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.next_val(),
            BigRational(value) => value.next_val(),
            BlockRef(value) => value.next_val(),
            Identifier(value) => value.next_val(),
            Nothing => crate::value::Nothing.next_val(),
            Tuple(value) => value.next_val(),
        }
    }

    fn into_val(self) -> BergResult<'a> {
        Ok(self)
    }

    fn into_native<T: TypeName + TryFrom<BergVal<'a>>>(self) -> BergResult<'a, BergResult<'a, T>>
    where
        <T as TryFrom<BergVal<'a>>>::Error: Into<BergVal<'a>>,
    {
        use BergVal::*;
        match self {
            // Blocks have to evaluate before they can into_native()
            BlockRef(value) => value.into_native(),
            // Tuples have a special case of 1 element before they can into_native()
            Tuple(value) => value.into_native(),
            _ => match T::try_from(self) {
                Ok(value) => Ok(Ok(value)),
                Err(original) => {
                    Ok(BergError::BadType(Box::new(original.into()), T::TYPE_NAME).err())
                }
            },
        }
    }

    fn infix<T: BergValue<'a>>(self, operator: IdentifierIndex, right: T) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.infix(operator, right),
            BigRational(value) => value.infix(operator, right),
            BlockRef(value) => value.infix(operator, right),
            Identifier(value) => value.infix(operator, right),
            Nothing => crate::value::Nothing.infix(operator, right),
            Tuple(value) => value.infix(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.postfix(operator),
            BigRational(value) => value.postfix(operator),
            BlockRef(value) => value.postfix(operator),
            Identifier(value) => value.postfix(operator),
            Nothing => crate::value::Nothing.postfix(operator),
            Tuple(value) => value.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.prefix(operator),
            BigRational(value) => value.prefix(operator),
            BlockRef(value) => value.prefix(operator),
            Identifier(value) => value.prefix(operator),
            Nothing => crate::value::Nothing.prefix(operator),
            Tuple(value) => value.prefix(operator),
        }
    }

    fn field(&self, name: IdentifierIndex) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.field(name),
            BigRational(value) => value.field(name),
            BlockRef(value) => value.field(name),
            Identifier(value) => value.field(name),
            Nothing => crate::value::Nothing.field(name),
            Tuple(value) => value.field(name),
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        field_value: BergResult<'a>,
    ) -> BergResult<'a, ()> {
        use BergVal::*;
        match self {
            Boolean(ref mut value) => value.set_field(name, field_value),
            BigRational(ref mut value) => value.set_field(name, field_value),
            BlockRef(ref mut value) => value.set_field(name, field_value),
            Identifier(ref mut value) => value.set_field(name, field_value),
            Nothing => crate::value::Nothing.set_field(name, field_value),
            Tuple(ref mut value) => value.set_field(name, field_value),
        }
    }
}

impl<'a> fmt::Debug for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BergVal::*;
        write!(f, "BergVal(")?;
        match self {
            Boolean(value) => write!(f, "{}", value)?,
            BigRational(ref value) => write!(f, "{}", value)?,
            BlockRef(ref value) => write!(f, "{}", value)?,
            Identifier(value) => write!(f, "{}", value)?,
            Nothing => write!(f, "{}", crate::value::Nothing)?,
            Tuple(ref value) => write!(f, "{}", value)?,
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BergVal::*;
        match *self {
            Boolean(ref value) => value.fmt(f),
            BigRational(ref value) => value.fmt(f),
            BlockRef(ref value) => value.fmt(f),
            Identifier(ref value) => value.fmt(f),
            Nothing => crate::value::Nothing.fmt(f),
            Tuple(ref value) => value.fmt(f),
        }
    }
}
