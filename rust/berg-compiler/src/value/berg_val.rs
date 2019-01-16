use crate::error::{BergError, EvalResult};
use crate::eval::BlockRef;
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::*;
use num::BigRational;

///
/// A concrete type that can hold any `BergValue`, and delegates operations to the concrete type.
///
#[derive(Clone)]
pub enum BergVal<'a> {
    Boolean(bool),
    BigRational(BigRational),
    BlockRef(BlockRef<'a>),
    // Error(Box<Error<'a>>),
    Identifier(IdentifierIndex),
    Nothing,
    Tuple(Tuple<'a>),
}

impl<'a> BergVal<'a> {
    pub fn downcast<T: TryFrom<Self, Error = Self> + TypeName>(self) -> EvalResult<'a, T> {
        match T::try_from(self) {
            Ok(result) => Ok(result),
            Err(original) => BergError::BadType(Box::new(original), T::TYPE_NAME).err(),
        }
    }
}

impl<'a> BergValue<'a> for BergVal<'a> {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        match self {
            BergVal::Boolean(value) => value.infix(operator, scope, right, ast),
            BergVal::BigRational(value) => value.infix(operator, scope, right, ast),
            BergVal::BlockRef(value) => value.infix(operator, scope, right, ast),
            // BergVal::Error(value) => value.infix(operator, scope, right, ast),
            BergVal::Identifier(value) => value.infix(operator, scope, right, ast),
            BergVal::Nothing => Nothing.infix(operator, scope, right, ast),
            BergVal::Tuple(value) => value.infix(operator, scope, right, ast),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match self {
            BergVal::Boolean(value) => value.postfix(operator, scope),
            BergVal::BigRational(value) => value.postfix(operator, scope),
            BergVal::BlockRef(value) => value.postfix(operator, scope),
            // BergVal::Error(value) => value.postfix(operator, scope),
            BergVal::Identifier(value) => value.postfix(operator, scope),
            BergVal::Nothing => Nothing.postfix(operator, scope),
            BergVal::Tuple(value) => value.postfix(operator, scope),
        }
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match self {
            BergVal::Boolean(value) => value.prefix(operator, scope),
            BergVal::BigRational(value) => value.prefix(operator, scope),
            BergVal::BlockRef(value) => value.prefix(operator, scope),
            // BergVal::Error(value) => value.prefix(operator, scope),
            BergVal::Identifier(value) => value.prefix(operator, scope),
            BergVal::Nothing => Nothing.prefix(operator, scope),
            BergVal::Tuple(value) => value.prefix(operator, scope),
        }
    }

    fn result(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.result(scope),
            BergVal::BigRational(value) => value.result(scope),
            BergVal::BlockRef(value) => value.result(scope),
            // BergVal::Error(value) => value.result(scope),
            BergVal::Identifier(value) => value.result(scope),
            BergVal::Nothing => Nothing.result(scope),
            BergVal::Tuple(value) => value.result(scope),
        }
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        match self {
            BergVal::Boolean(value) => value.field(name),
            BergVal::BigRational(value) => value.field(name),
            BergVal::BlockRef(value) => value.field(name),
            // BergVal::Error(value) => value.field(name),
            BergVal::Identifier(value) => value.field(name),
            BergVal::Nothing => Nothing.field(name),
            BergVal::Tuple(value) => value.field(name),
        }
    }
    fn set_field(
        &mut self,
        name: IdentifierIndex,
        field_value: BergResult<'a>,
    ) -> EvalResult<'a, ()> {
        match self {
            BergVal::Boolean(ref mut value) => value.set_field(name, field_value),
            BergVal::BigRational(ref mut value) => value.set_field(name, field_value),
            BergVal::BlockRef(ref mut value) => value.set_field(name, field_value),
            // BergVal::Error(ref mut value) => value.set_field(name, field_value),
            BergVal::Identifier(ref mut value) => value.set_field(name, field_value),
            BergVal::Nothing => Nothing.set_field(name, field_value),
            BergVal::Tuple(ref mut value) => value.set_field(name, field_value),
        }
    }
}

impl<'a> fmt::Debug for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BergVal(")?;
        match self {
            BergVal::Boolean(value) => write!(f, "{}", value)?,
            BergVal::BigRational(ref value) => write!(f, "{}", value)?,
            BergVal::BlockRef(ref value) => write!(f, "{}", value)?,
            // BergVal::Error(ref value) => write!(f, "{}", value)?,
            BergVal::Identifier(value) => write!(f, "{}", value)?,
            BergVal::Nothing => write!(f, "{}", Nothing)?,
            BergVal::Tuple(ref value) => write!(f, "{}", value)?,
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BergVal::Boolean(ref value) => value.fmt(f),
            BergVal::BigRational(ref value) => value.fmt(f),
            BergVal::BlockRef(ref value) => value.fmt(f),
            // BergVal::Error(ref value) => value.fmt(f),
            BergVal::Identifier(ref value) => value.fmt(f),
            BergVal::Nothing => Nothing.fmt(f),
            BergVal::Tuple(ref value) => value.fmt(f),
        }
    }
}
