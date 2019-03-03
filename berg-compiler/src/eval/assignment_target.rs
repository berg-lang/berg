use crate::eval::{ScopeRef, AmbiguousSyntax};
use crate::syntax::{FieldIndex, IdentifierIndex};
use crate::syntax::identifiers::*;
use crate::value::implement::*;

#[derive(Debug, Clone)]
pub enum AssignmentTarget<'a> {
    LocalFieldReference(ScopeRef<'a>, FieldIndex),
    LocalFieldDeclaration(ScopeRef<'a>, FieldIndex),
    ObjectFieldReference(BergVal<'a>, IdentifierIndex),
}

impl<'a> AssignmentTarget<'a> {
    pub fn result<T>(self) -> BergResult<'a, T> {
        AmbiguousSyntax::Target(self).result()
    }

    pub fn get(&self) -> BergResult<'a> {
        self.initialize()?;
        let result = self.get_internal();
        self.declare()?;
        result
    }

    pub fn set(&mut self, value: impl BergValue<'a>) -> BergResult<'a, ()> {
        self.set_internal(value.into_val())?;
        // If it's a declaration, declare it public now that it's been set.
        self.declare()?;
        Ok(())
    }

    pub fn update<F: FnOnce(BergResult<'a>) -> BergResult<'a>>(&mut self, f: F) -> BergResult<'a, ()> {
        self.initialize()?;
        let value = f(self.get_internal());
        self.set(value)?;
        Ok(())
    }

    fn initialize(&self) -> BergResult<'a, ()> {
        use AssignmentTarget::*;
        match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => scope
                .bring_local_field_into_scope(*field, &scope.ast()),
            ObjectFieldReference(..) => Ok(()),
        }
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

    fn set_internal(&mut self, value: BergResult<'a>) -> BergResult<'a, ()> {
        if let Err(ControlVal::ExpressionError(error, position)) = value {
            panic!("ExpressionError({:?}, {:?})", error, position);
        }
        use AssignmentTarget::*;
        match self {
            LocalFieldReference(scope, field) | LocalFieldDeclaration(scope, field) => 
                scope.set_local_field(*field, value, &scope.ast())?,
            ObjectFieldReference(object, name) => { object.set_field(*name, value)?; }
        };
        Ok(())
    }

    fn declare(&self) -> BergResult<'a, ()> {
        use AssignmentTarget::*;
        match self {
            LocalFieldDeclaration(scope, field) => scope
                .declare_field(*field, &scope.ast())?,
            LocalFieldReference(..) | ObjectFieldReference(..) => {}
        }
        Ok(())
    }

    fn point_errors_at_identifier(&self, result: BergResult<'a>) -> BergResult<'a> {
        use AssignmentTarget::*;
        use ExpressionErrorPosition::*;
        match result {
            Err(ControlVal::ExpressionError(error, Expression)) => match self {
                LocalFieldDeclaration(..) | ObjectFieldReference(..) => error.operand_err(RightOperand),
                LocalFieldReference(..) => error.err(),
            },
            _ => result,
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
        self.result()
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
            (COLON, LocalFieldReference(scope, field)) => LocalFieldDeclaration(scope, field).result(),
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
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        self.update(|mut v| v.set_field(name, value).and_then(|_| v))
    }
}
