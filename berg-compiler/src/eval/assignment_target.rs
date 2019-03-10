use crate::eval::{ScopeRef, AmbiguousSyntax};
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::syntax::identifiers::*;
use crate::value::implement::*;
use std::fmt;

#[derive(Debug, Clone)]
pub enum AssignmentTarget<'a> {
    LocalFieldReference(ScopeRef<'a>, FieldIndex),
    LocalFieldDeclaration(ScopeRef<'a>, FieldIndex),
    ObjectFieldReference(BergVal<'a>, IdentifierIndex),
}

impl<'a> AssignmentTarget<'a> {
    pub fn err<T>(self) -> BergResult<'a, T> {
        AmbiguousSyntax::Target(self).err()
    }

    pub fn get(&self) -> BergResult<'a> {
        // If it's a declaration, declare it and get its initial value, if any.
        self.declare()?;
        self.get_internal()
    }

    pub fn set(&mut self, value: BergVal<'a>, operand_position: ExpressionErrorPosition) -> BergResult<'a> {
        match self.set_internal(value).and_then(|_| self.declare()) {
            Ok(()) => BergVal::empty_tuple().ok(),
            Err(error) => error.at_position(operand_position),
        }
    }

    fn declare(&self) -> BergResult<'a, ()> {
        use AssignmentTarget::*;
        match self {
            LocalFieldDeclaration(scope, field) => scope.declare_field(*field, &scope.ast())?,
            LocalFieldReference(..) | ObjectFieldReference(..) => {}
        }
        Ok(())
    }

    fn get_internal(&self) -> BergResult<'a> {
        use AssignmentTarget::*;
        let result = match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => 
                scope.local_field(*field, &scope.ast()),
            ObjectFieldReference(object, name) => object.clone().field(*name)
        };
        self.point_errors_at_identifier(result)
    }

    fn set_internal(&mut self, value: BergVal<'a>) -> BergResult<'a, ()> {
        use AssignmentTarget::*;
        let result = match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => 
                scope.set_local_field(*field, value, &scope.ast()),
            ObjectFieldReference(object, name) => {
                object.set_field(*name, value)
            }
        };
        self.point_errors_at_identifier(result)?;
        Ok(())
    }

    fn point_errors_at_identifier<T: fmt::Debug>(&self, result: BergResult<'a, T>) -> BergResult<'a, T> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::*;
        match result {
            Err(ControlVal::ExpressionError(error, Expression)) => match self {
                LocalFieldDeclaration(..) | ObjectFieldReference(..) => error.operand_err(RightOperand),
                LocalFieldReference(..) => error.err(),
            },
            Err(error) => Err(error),
            Ok(value) => Ok(value),
        }
    }
}

impl<'a> From<AssignmentTarget<'a>> for ControlVal<'a> {
    fn from(from: AssignmentTarget<'a>) -> Self {
        AmbiguousSyntax::Target(from).into()
    }
}

impl<'a> BergValue<'a> for AssignmentTarget<'a> {
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        self.get().next_val()
    }
    fn into_val(self) -> BergResult<'a> {
        self.err()
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.get().into_native()
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.get().try_into_native()
    }
    fn infix(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::LeftOperand;
        match (operator, &self) {
            // Handle <identifier>: <value>
            (COLON, LocalFieldReference(..)) => self.set(right.into_val()?, LeftOperand),
            _ => self.get().infix(operator, right)
        }
    }
    fn infix_assign(mut self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use ExpressionErrorPosition::LeftOperand;
        match operator {
            EMPTY_STRING => self.set(right.into_val()?, LeftOperand),
            operator => self.set(self.get().infix(operator, right)?, LeftOperand),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::RightOperand;
        match (operator, self) {
            (COLON, LocalFieldReference(scope, field)) => LocalFieldDeclaration(scope, field).err(),
            (PLUS_PLUS, mut right) => right.set(right.get().prefix(PLUS_ONE)?, RightOperand),
            (DASH_DASH, mut right) => right.set(right.get().prefix(MINUS_ONE)?, RightOperand),
            (_, right) => right.get().prefix(operator),
        }
    }

    fn postfix(mut self, operator: IdentifierIndex) -> BergResult<'a> {
        use ExpressionErrorPosition::LeftOperand;
        match operator {
            PLUS_PLUS => self.set(self.get().postfix(PLUS_ONE)?, LeftOperand),
            DASH_DASH => self.set(self.get().postfix(MINUS_ONE)?, LeftOperand),
            _ => self.get().postfix(operator)
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        self.get().subexpression_result(boundary)
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        self.get().field(name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> BergResult<'a, ()> {
        use ExpressionErrorPosition::Expression;
        let mut obj = self.get()?;
        obj.set_field(name, value)?;
        self.set(obj, Expression).and(Ok(()))
    }
}
