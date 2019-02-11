use crate::syntax::{ExpressionRef, FieldIndex, IdentifierIndex};
use crate::value::{BergError, BergVal, Error};
use std::fmt;

pub trait ControlValue {
}

#[derive(Debug, Clone)]
pub enum ControlVal<'a> {
    MissingExpression,
    LocalFieldDeclaration(FieldIndex),
    LocalFieldReference(FieldIndex),
    RawIdentifier(IdentifierIndex),
    ObjectFieldReference(BergVal<'a>, IdentifierIndex),
    Error(Error<'a>),
    LocalError(BergError<'a>),
}

impl<'a> fmt::Display for ControlVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ControlVal::*;
        match self {
            Error(error) => write!(f, "{}", error),
            RawIdentifier(identifier) => write!(f, "RawIdentifier({})", identifier),
            ObjectFieldReference(value, name) => write!(f, "ObjectFieldReference({}, {})", value, name),
            MissingExpression | LocalError(_) | LocalFieldDeclaration(_) | LocalFieldReference(_) => write!(f, "{:?}", self),
        }
    }
}

impl<'a> ControlVal<'a> {
    pub fn push_frame(self, expression: ExpressionRef<'a>) -> Self {
        use ControlVal::*;
        match self {
            Error(error) => Error(error.push_frame(expression)),
            LocalError(error) => Error(error.push_frame(expression)),
            MissingExpression => LocalError(BergError::MissingExpression),
            RawIdentifier(_) | ObjectFieldReference(..) | LocalFieldDeclaration(_) | LocalFieldReference(_) => self,
        }
    }
}

impl<'a> ControlValue for ControlVal<'a> {
}

impl<'a> From<Error<'a>> for ControlVal<'a> {
    fn from(from: Error<'a>) -> Self {
        ControlVal::Error(from)
    }
}
impl<'a> From<BergError<'a>> for ControlVal<'a> {
    fn from(from: BergError<'a>) -> Self {
        ControlVal::LocalError(from)
    }
}
