use error::{BergError, EvalResult};
use util::try_from::TryFrom;
use util::type_name::TypeName;
use num::BigRational;
use value;
use value::*;

///
/// A concrete type that can hold any `BergValue`, and delegates operations to the concrete type.
///
#[derive(Clone)]
pub enum BergVal<'a> {
    Boolean(bool),
    BigRational(BigRational),
    Identifier(IdentifierIndex),
    Closure(Closure<'a>),
    Nothing,
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
            BergVal::Identifier(value) => value.infix(operator, scope, right, ast),
            BergVal::Closure(value) => value.infix(operator, scope, right, ast),
            BergVal::Nothing => Nothing.infix(operator, scope, right, ast),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match self {
            BergVal::Boolean(value) => value.postfix(operator, scope),
            BergVal::BigRational(value) => value.postfix(operator, scope),
            BergVal::Identifier(value) => value.postfix(operator, scope),
            BergVal::Closure(value) => value.postfix(operator, scope),
            BergVal::Nothing => Nothing.postfix(operator, scope),
        }
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match self {
            BergVal::Boolean(value) => value.prefix(operator, scope),
            BergVal::BigRational(value) => value.prefix(operator, scope),
            BergVal::Identifier(value) => value.prefix(operator, scope),
            BergVal::Closure(value) => value.prefix(operator, scope),
            BergVal::Nothing => Nothing.prefix(operator, scope),
        }
    }

    fn evaluate(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.evaluate(scope),
            BergVal::BigRational(value) => value.evaluate(scope),
            BergVal::Identifier(value) => value.evaluate(scope),
            BergVal::Closure(value) => value.evaluate(scope),
            BergVal::Nothing => Nothing.evaluate(scope),
        }
    }
}

impl<'a> fmt::Debug for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BergVal(")?;
        match *self {
            BergVal::Boolean(value) => write!(f, "{}", value)?,
            BergVal::BigRational(ref value) => write!(f, "{}", value)?,
            BergVal::Identifier(value) => write!(f, "{}", value)?,
            BergVal::Closure(ref value) => write!(f, "{}", value)?,
            BergVal::Nothing => write!(f, "{}", Nothing)?,
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use value::BergVal::*;
        match *self {
            Boolean(ref value) => value.fmt(f),
            BigRational(ref value) => value.fmt(f),
            BergVal::Identifier(ref value) => value.fmt(f),
            BergVal::Closure(ref value) => value.fmt(f),
            Nothing => value::Nothing.fmt(f),
        }
    }
}
