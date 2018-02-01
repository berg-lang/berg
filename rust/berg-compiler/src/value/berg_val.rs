use value::berg_error::BergErrorStack;
use util::try_from::TryFrom;
use util::type_name::TypeName;
use eval::{BlockClosure, Expression};
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
    Nothing,
    BlockClosure(BlockClosure<'a>),
    BergErrorStack(BergErrorStack<'a>),
}

impl<'a> BergVal<'a> {
    pub fn downcast<T: TryFrom<Self, Error = Self> + TypeName>(self) -> BergResult<'a, T> {
        match T::try_from(self.complete()?) {
            Ok(result) => Ok(result),
            Err(original) => BergError::BadType(Box::new(original), T::TYPE_NAME).err(),
        }
    }

    pub fn fmt_debug_shallow(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BergVal::Boolean(value) => write!(f, "{}", value),
            BergVal::BigRational(ref value) => write!(f, "{}", value),
            BergVal::Nothing => write!(f, "{}", Nothing),
            BergVal::BlockClosure(ref value) => write!(f, "{}", value),
            BergVal::BergErrorStack(ref value) => write!(f, "{:?}", value),
        }
    }

    pub fn complete(self) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.complete(),
            BergVal::BigRational(value) => value.complete(),
            BergVal::Nothing => Nothing.complete(),
            BergVal::BlockClosure(value) => value.complete(),
            BergVal::BergErrorStack(value) => value.complete(),
        }
    }

    pub fn unwind_error(self, ast: AstRef<'a>, expression: Expression) -> BergVal<'a> {
        match self {
            BergVal::Boolean(value) => value.unwind_error(ast, expression),
            BergVal::BigRational(value) => value.unwind_error(ast, expression),
            BergVal::Nothing => Nothing.unwind_error(ast, expression),
            BergVal::BlockClosure(value) => value.unwind_error(ast, expression),
            BergVal::BergErrorStack(value) => value.unwind_error(ast, expression),
        }
    }

    pub fn infix(
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
            BergVal::BlockClosure(value) => value.infix(operator, scope, right, ast),
            BergVal::BergErrorStack(value) => value.infix(operator, scope, right, ast),
        }
    }

    pub fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.postfix(operator, scope),
            BergVal::BigRational(value) => value.postfix(operator, scope),
            BergVal::Nothing => Nothing.postfix(operator, scope),
            BergVal::BlockClosure(value) => value.postfix(operator, scope),
            BergVal::BergErrorStack(value) => value.postfix(operator, scope),
        }
    }

    pub fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match self {
            BergVal::Boolean(value) => value.prefix(operator, scope),
            BergVal::BigRational(value) => value.prefix(operator, scope),
            BergVal::Nothing => Nothing.prefix(operator, scope),
            BergVal::BlockClosure(value) => value.prefix(operator, scope),
            BergVal::BergErrorStack(value) => value.prefix(operator, scope),
        }
    }
}

impl<'a> fmt::Debug for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use value::BergVal::*;
        write!(f, "BergVal(")?;
        match *self {
            Boolean(ref value) => value.fmt(f)?,
            BigRational(ref value) => value.fmt(f)?,
            Nothing => value::Nothing.fmt(f)?,
            BlockClosure(ref value) => value.fmt(f)?,
            BergErrorStack(ref value) => value.fmt(f)?,
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
            Nothing => value::Nothing.fmt(f),
            BlockClosure(ref value) => value.fmt(f),
            BergErrorStack(ref value) => value.fmt(f),
        }
    }
}
