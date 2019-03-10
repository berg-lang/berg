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

    pub fn set(&mut self, value: BergVal<'a>) -> BergResult<'a, ()> {
        self.set_internal(value)?;
        // If it's a declaration, declare it public now that it's been set.
        self.declare()?;
        Ok(())
    }

    pub fn update<F: FnOnce(BergVal<'a>) -> BergResult<'a>>(&mut self, f: F) -> BergResult<'a, ()> {
        // If it's a declaration, declare it so that it gets its initial value (if any).
        self.declare()?;
        let value = f(self.get_internal()?)?;
        self.set(value)?;
        Ok(())
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
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        self.get().infix(operator, right)
    }
    fn infix_assign(self, _operator: IdentifierIndex, _right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        unreachable!()
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use AssignmentTarget::*;
        match (operator, self) {
            (COLON, LocalFieldReference(scope, field)) => LocalFieldDeclaration(scope, field).err(),
            (_, v) => v.get().prefix(operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.get().postfix(operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        self.get().subexpression_result(boundary)
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        self.get().field(name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> BergResult<'a, ()> {
        self.update(|mut v| v.set_field(name, value).and_then(|_| v.ok()))
    }
}
