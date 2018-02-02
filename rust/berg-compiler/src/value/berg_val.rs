use util::try_from::TryFrom;
use util::type_name::TypeName;
use num::BigRational;
use value;
use value::*;

///
/// A concrete type that can hold any `BergValue`, and delegates operations to the concrete type.
///
#[derive(Clone)]
pub enum BergVal {
    Boolean(bool),
    BigRational(BigRational),
    Nothing,
}

impl BergVal {
    pub fn downcast<T: TryFrom<Self, Error = Self> + TypeName>(self) -> BergResult<'static, T> {
        match T::try_from(self) {
            Ok(result) => Ok(result),
            Err(original) => BergError::BadType(Box::new(original), T::TYPE_NAME).err(),
        }
    }
}

impl<'a> BergValue<'a> for BergVal {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.infix(operator, scope, right, ast),
            BergVal::BigRational(value) => value.infix(operator, scope, right, ast),
            BergVal::Nothing => Nothing.infix(operator, scope, right, ast),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.postfix(operator, scope),
            BergVal::BigRational(value) => value.postfix(operator, scope),
            BergVal::Nothing => Nothing.postfix(operator, scope),
        }
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.prefix(operator, scope),
            BergVal::BigRational(value) => value.prefix(operator, scope),
            BergVal::Nothing => Nothing.prefix(operator, scope),
        }
    }
}

impl fmt::Debug for BergVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BergVal(")?;
        match *self {
            BergVal::Boolean(value) => write!(f, "{}", value)?,
            BergVal::BigRational(ref value) => write!(f, "{}", value)?,
            BergVal::Nothing => write!(f, "{}", Nothing)?,
        }
        write!(f, ")")
    }
}

impl fmt::Display for BergVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use value::BergVal::*;
        match *self {
            Boolean(ref value) => value.fmt(f),
            BigRational(ref value) => value.fmt(f),
            Nothing => value::Nothing.fmt(f),
        }
    }
}
