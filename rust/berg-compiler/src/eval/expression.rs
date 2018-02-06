use error::{BergError, Error, BergResult, EvalResult, Raw, TakeError};
use eval::{ExpressionFormatter, ScopeRef};
use num::BigRational;
use parser::{ByteRange, ByteSlice};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use syntax::{AstIndex, AstRef, ExpressionBoundary, ExpressionBoundaryError, FieldIndex, Fixity, IdentifierIndex,
             OperandPosition, Token};
use syntax::Fixity::*;
use syntax::OperandPosition::*;
use syntax::identifiers::{CALL, COLON, DASH_DASH, DOT, EMPTY_STRING, NEWLINE, PLUS_PLUS, SEMICOLON};
use util::try_from::TryFrom;
use util::type_name::TypeName;
use value::{BergVal, BergValue};

#[derive(Copy, Clone, PartialEq)]
pub struct Expression(pub AstIndex);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Operand {
    pub expression: Expression,
    pub position: OperandPosition,
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expression({})", (self.0).0)
    }
}

impl Expression {
    pub fn evaluate_local<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        println!("Evaluate {} ...", ExpressionFormatter(self, ast));
        use syntax::Token::*;
        use error::ErrorCode::*;
        use syntax::ExpressionBoundaryError::*;
        let result = match *self.token(ast) {
            IntegerLiteral(literal) => {
                let parsed = BigRational::from_str(ast.literal_string(literal)).unwrap();
                Ok(BergVal::BigRational(parsed))
            },
            FieldReference(field) => scope.local_field(field, ast).take_error(ast, self),
            ErrorTerm(IdentifierStartsWithNumber) => BergError::IdentifierStartsWithNumber.take_error(ast, self),
            ErrorTerm(UnsupportedCharacters) => BergError::UnsupportedCharacters.take_error(ast, self),
            ErrorTerm(InvalidUtf8) => BergError::InvalidUtf8.take_error(ast, self),
            MissingExpression => Ok(BergVal::Nothing),

            InfixOperator(SEMICOLON) => self.evaluate_semicolon(scope, ast),
            InfixOperator(operator) => self.evaluate_infix(operator, scope, ast),
            InfixAssignment(operator) => self.evaluate_infix_assign(operator, scope, ast),
            NewlineSequence => self.evaluate_infix(NEWLINE, scope, ast),
            MissingInfix => self.evaluate_infix(CALL, scope, ast),

            PrefixOperator(PLUS_PLUS) => self.evaluate_prefix_assign(PLUS_PLUS, scope, ast),
            PrefixOperator(DASH_DASH) => self.evaluate_prefix_assign(DASH_DASH, scope, ast),
            PrefixOperator(COLON) => self.evaluate_declare(scope, ast),
            PrefixOperator(operator) => self.evaluate_prefix(operator, scope, ast),

            PostfixOperator(PLUS_PLUS) => self.evaluate_postfix_assign(PLUS_PLUS, scope, ast),
            PostfixOperator(DASH_DASH) => self.evaluate_postfix_assign(DASH_DASH, scope, ast),
            PostfixOperator(operator) => self.evaluate_postfix(operator, scope, ast),

            Open {
                error: OpenError, ..
            }
            | OpenBlock {
                error: OpenError, ..
            } => ast.open_error().clone().take_error(ast, self),
            OpenBlock {
                error: ExpressionBoundaryError::OpenWithoutClose,
                ..
            }
            | Open {
                error: ExpressionBoundaryError::OpenWithoutClose,
                ..
            } => BergError::OpenWithoutClose.take_error(ast, self),
            OpenBlock {
                error: ExpressionBoundaryError::CloseWithoutOpen,
                ..
            }
            | Open {
                error: ExpressionBoundaryError::CloseWithoutOpen,
                ..
            } => BergError::CloseWithoutOpen.take_error(ast, self),
            Open { error: None, .. } => self.inner_expression(ast).evaluate_local(scope, ast),
            OpenBlock { error: None, index, .. } => Ok(scope.create_child_block(self.inner_expression(ast), index).into()),
            RawIdentifier(name) => Ok(name.into()),
            Close { .. } | CloseBlock { .. } | ErrorTerm(_) => unreachable!(),
        };
        println!("Result of {}: {:?}", ExpressionFormatter(self, ast), result);
        result
    }

    fn evaluate_semicolon<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let left = self.left_operand(ast)?;
        let left_value = left.evaluate_local(scope, ast)?;

        // If the left hand side is a semicolon with a missing expression between,
        // raise MissingExpression.
        if let BergVal::Nothing = left_value {
            if left.operator() == self.operator() - 2 {
                if let Token::InfixOperator(SEMICOLON) = *left.token(ast) {
                    let immediate_left = Expression(left.operator() + 1);
                    if let Token::MissingExpression = *immediate_left.token(ast) {
                        return BergError::MissingOperand.take_error(ast, self);
                    }
                }
            }
        }

        let right = Operand {
            position: OperandPosition::Right,
            expression: self.right_expression(ast),
        };
        left_value.infix(SEMICOLON, scope, right, ast).take_error(ast, self)
    }

    fn evaluate_infix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let left = self.left_operand(ast)?;
        let right = self.right_operand(ast)?;
        left.infix(operator, scope, right, ast).take_error(ast, self)
    }

    fn evaluate_infix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let mut target = AssignmentTarget::from_expression(self.left_operand(ast)?.expression, scope, ast)?;
        let right = self.right_operand(ast)?;
        let value = match operator {
            EMPTY_STRING => {
                target.initialize(scope, ast)?;
                right.evaluate_local(scope, ast)
            },
            _ => target.get(scope, ast)?.infix(operator, scope, right, ast).take_error(ast, self),
        };
        target.set(value, scope, ast)
    }

    fn evaluate_prefix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.prefix_operand(ast)?;
        operand.prefix(operator, scope, ast).take_error(ast, self)
    }

    fn evaluate_postfix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.postfix_operand(ast)?;
        operand.postfix(operator, scope, ast).take_error(ast, self)
    }

    fn evaluate_prefix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.prefix_operand(ast)?;
        let mut target = AssignmentTarget::from_expression(operand.expression, scope, ast)?;
        target.initialize(scope, ast)?;
        let value = operand.prefix(operator, scope, ast).take_error(ast, self);
        target.set(value, scope, ast)
    }

    fn evaluate_postfix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.postfix_operand(ast)?;
        let mut target = AssignmentTarget::from_expression(operand.expression, scope, ast)?;
        target.initialize(scope, ast)?;
        let value = operand.postfix(operator, scope, ast).take_error(ast, self);
        target.set(value, scope, ast)
    }

    fn evaluate_declare<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let mut target = AssignmentTarget::from_expression(self, scope, ast)?;
        target.declare(scope, ast)?;
        target.get(scope, ast)
    }

    pub(crate) fn range(self, ast: &AstRef) -> ByteRange {
        let start = ast.token_ranges()[self.first_index(ast)].start;
        let end = ast.token_ranges()[self.last_index(ast)].end;
        start..end
    }

    pub(crate) fn operator(self) -> AstIndex {
        self.0
    }

    pub(crate) fn first_index(self, ast: &AstRef) -> AstIndex {
        let token = self.token(ast);
        match *token {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => self.operator() - delta,
            _ => {
                let mut left = self;
                while left.token(ast).has_left_operand() {
                    left = left.left_expression(ast);
                }
                left.operator()
            }
        }
    }

    pub(crate) fn last_index(self, ast: &AstRef) -> AstIndex {
        let token = self.token(ast);
        match *token {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => self.operator() + delta,
            _ => {
                let mut right = self;
                while right.token(ast).has_right_operand() {
                    right = right.right_expression(ast);
                }
                right.operator()
            }
        }
    }

    pub(crate) fn token<'p>(&self, ast: &'p AstRef) -> &'p Token {
        &ast.tokens()[self.operator()]
    }

    pub(crate) fn open_token<'p>(&self, ast: &'p AstRef) -> &'p Token {
        &ast.tokens()[self.open_operator(ast)]
    }

    pub(crate) fn close_token<'p>(&self, ast: &'p AstRef) -> &'p Token {
        &ast.tokens()[self.close_operator(ast)]
    }

    pub(crate) fn open_operator<'p>(&self, ast: &'p AstRef) -> AstIndex {
        match *self.token(ast) {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => self.operator() - delta,
            _ => self.operator(),
        }
    }

    pub(crate) fn close_operator<'p>(&self, ast: &'p AstRef) -> AstIndex {
        match *self.token(ast) {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => self.operator() + delta,
            _ => self.operator(),
        }
    }

    pub(crate) fn boundary(&self, ast: &AstRef) -> ExpressionBoundary {
        match *self.open_token(ast) {
            Token::Open { boundary, .. } => boundary,
            Token::OpenBlock { index, .. } => ast.blocks()[index].boundary,
            _ => unreachable!(),
        }
    }

    pub(crate) fn operand<'a>(
        self,
        position: OperandPosition,
        ast: &AstRef<'a>,
    ) -> Result<Operand, Error<'a>> {
        let expression = position.get(self, ast);
        match *expression.token(ast) {
            Token::MissingExpression => BergError::MissingOperand.take_error(ast, self),
            _ => Ok(Operand {
                expression,
                position,
            }),
        }
    }

    pub(crate) fn left_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::Left, ast)
    }
    pub(crate) fn right_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::Right, ast)
    }
    pub(crate) fn prefix_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::PrefixOperand, ast)
    }
    pub(crate) fn postfix_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::PostfixOperand, ast)
    }

    pub(crate) fn left_expression(&self, ast: &AstRef) -> Self {
        // Grab the term immediately to our left.
        let allow_infix_children = match self.token(ast).fixity() {
            Fixity::Close | Fixity::Infix => true,
            _ => false,
        };
        let end = self.0 - 1;
        let mut start = end;

        // Pass any postfixes to find the term.
        let mut has_postfix = false;
        while ast.tokens()[start].fixity() == Fixity::Postfix {
            start -= 1;
            has_postfix = true;
        }

        // Jump to the open token if it's a group term (parens, curlies, etc.)
        match ast.tokens()[start] {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => {
                start -= delta;
            }
            _ => {}
        }

        // Pass any prefixes if there is no postfix or infix.
        if !allow_infix_children || !has_postfix {
            while start > 0 && ast.tokens()[start - 1].fixity() == Fixity::Prefix {
                start -= 1;
            }
        }

        // Check for an infix.
        if allow_infix_children && start > 0 && ast.tokens()[start - 1].fixity() == Fixity::Infix {
            return Expression(start - 1);
        }

        // Pick postfix if there is one.
        if has_postfix {
            return Expression(end);
        }

        // Otherwise, it's the leftmost index (either a prefix or term).
        Expression(start)
    }

    pub(crate) fn right_expression(&self, ast: &AstRef) -> Self {
        let start = self.operator() + 1;

        match self.token(ast).fixity() {
            // If this is prefix, it cannot have postfix or infix children, so its immediate right is the child.
            Fixity::Prefix => return Expression(start),
            // If this is a group term, call inner() and return.
            Fixity::Open => return self.inner_expression(ast),
            // Otherwise, it's guaranteed to be infix.
            Fixity::Infix => {}
            _ => unreachable!(),
        }

        // Check whether there is a postfix by skipping prefix and term.
        let mut end = start;
        while ast.tokens()[end].fixity() == Fixity::Prefix {
            end += 1;
        }
        match ast.tokens()[end] {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => {
                end += delta;
            }
            _ => {}
        }
        let mut has_postfix = false;
        while end < ast.tokens().last_index() && ast.tokens()[end + 1].fixity() == Fixity::Postfix {
            end += 1;
            has_postfix = true;
        }

        // If there is at least one postfix, return the outermost postfix.
        if has_postfix {
            return Expression(end);
        }

        // Otherwise, the right child is the immediate right term (or prefix).
        Expression(start)
    }

    pub(crate) fn parent(&self, ast: &AstRef) -> Self {
        // Grab the next and previous expression.
        let first_index = self.first_index(ast);
        let last_index = self.last_index(ast);
        let next = Expression(last_index + 1);
        if first_index == 0 {
            assert!(next.0 <= ast.tokens().last_index());
            return Expression(last_index + 1);
        }
        let prev = Expression(first_index - 1);
        if last_index >= ast.tokens().last_index() {
            return prev;
        }

        // prefix > postfix > left infix > right infix > open+close
        match (prev.token(ast).fixity(), next.token(ast).fixity()) {
            (Infix, Postfix) | (Open, Postfix) | (Open, Infix) => next,

            (Prefix, Postfix)
            | (Prefix, Infix)
            | (Prefix, Close)
            | (Infix, Infix)
            | (Infix, Close)
            | (Open, Close) => prev,

            (Postfix, _) | (Close, _) | (Term, _) | (_, Prefix) | (_, Open) | (_, Term) => {
                unreachable!()
            }
        }
    }

    pub(crate) fn operand_position(&self, ast: &AstRef) -> OperandPosition {
        let parent = self.parent(ast);
        match parent.token(ast).fixity() {
            Prefix | Open => PrefixOperand,
            Postfix | Close => PostfixOperand,
            Infix if self.0 < parent.0 => Left,
            Infix => Right,
            Term => unreachable!(),
        }
    }

    pub(crate) fn inner_expression<'a>(&self, ast: &AstRef<'a>) -> Self {
        Expression(self.close_operator(ast)).left_expression(ast)
    }

    pub(crate) fn to_string<'a>(&self, ast: &'a AstRef) -> Cow<'a, str> {
        if self.token(ast).fixity() == Fixity::Term {
            self.token(ast).to_string(ast).into()
        } else {
            // TODO this is terrible, but until we save comments and spaces in the AST (which we should),
            // we have to reopen the source
            let buffer = ast.source().reopen();
            cow_range_from_utf8_lossy(buffer, self.range(ast))
        }
    }
}

#[derive(Debug, Clone)]
enum AssignmentTarget<'a> {
    Local(FieldIndex, Expression),
    DeclareLocal(FieldIndex, Expression),
    Object(BergVal<'a>, IdentifierIndex, Operand, Expression)
}

impl<'a> AssignmentTarget<'a> {
    fn from_expression(
        expression: Expression,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a, AssignmentTarget<'a>> {
        use syntax::Token::*;
        match *expression.token(ast) {
            FieldReference(field) => Ok(AssignmentTarget::Local(field, expression)),
            PrefixOperator(COLON) => {
                let colon_operand = expression.prefix_operand(ast)?;
                match *colon_operand.token(ast) {
                    FieldReference(field) => Ok(AssignmentTarget::DeclareLocal(field, colon_operand.expression)),
                    _ => BergError::AssignmentTargetMustBeIdentifier.take_error(ast, colon_operand.expression),
                }
            }
            Open{error: ExpressionBoundaryError::None, ..} => AssignmentTarget::from_expression(expression.inner_expression(ast), scope, ast),
            InfixOperator(DOT) => {
                let right = expression.right_operand(ast)?;
                match *right.token(ast) {
                    RawIdentifier(name) => {
                        let object = expression.left_operand(ast)?.evaluate_local(scope, ast)?;
                        Ok(AssignmentTarget::Object(object, name, right, expression))
                    }
                    _ => BergError::AssignmentTargetMustBeIdentifier.take_error(ast, right.expression)
                }
            }
            _ => BergError::AssignmentTargetMustBeIdentifier.take_error(ast, expression),
        }
    }

    fn initialize(&self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a, ()> {
        use eval::expression::AssignmentTarget::*;
        match *self {
            DeclareLocal(field, expression)|Local(field, expression) => scope.bring_local_field_into_scope(field, ast).take_error(ast, expression),
            Object(..) => Ok(())
        }
    }

    fn get(&mut self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        use eval::expression::AssignmentTarget::*;
        use syntax::identifiers::DOT;
        self.initialize(scope, ast)?;
        match *self {
            DeclareLocal(field, expression)|Local(field, expression) => scope.local_field(field, ast).take_error(ast, expression),
            Object(ref object, _, right, expression) => {
                // Infix consumes values, but we still need the object around, so we clone the obj (it's cheap at the moment, a reference or primitive)
                let object = object.clone();
                object.infix(DOT, scope, right, ast).take_error(ast, expression)
            }
        }
    }

    fn set(&mut self, value: BergResult<'a>, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        use eval::expression::AssignmentTarget::*;
        match *self {
            Local(field, expression)|DeclareLocal(field, expression) => scope.set_local_field(field, value, ast).take_error(ast, expression)?,
            Object(ref mut object, name, _, expression) => object.set_field(name, value, scope).take_error(ast, expression)?,
        }
        // If it's a declaration, declare it public now that it's been set.
        self.declare(scope, ast)?;
        Ok(BergVal::Nothing)
    }

    fn declare(&mut self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        use eval::expression::AssignmentTarget::*;
        match *self {
            DeclareLocal(field, expression) => scope.declare_field(field, ast).take_error(ast, expression)?,
            Local(..)|Object(..) => {},
        }
        Ok(BergVal::Nothing)
    }
}

impl Operand {
    pub fn infix<'a>(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>, right: Operand, ast: &AstRef<'a>) -> EvalResult<'a> {
        let value = self.expression.evaluate_local(scope, ast)?;
        value.infix(operator, scope, right, ast)
    }
    pub fn postfix<'a>(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> EvalResult<'a> {
        let value = self.expression.evaluate_local(scope, ast)?;
        value.postfix(operator, scope)
    }
    pub fn prefix<'a>(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> EvalResult<'a> {
        let value = self.expression.evaluate_local(scope, ast)?;
        value.prefix(operator, scope)
    }
    pub fn evaluate_local<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        self.expression.evaluate_local(scope, ast)
    }
    pub fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        self.evaluate_local(scope, ast)?.evaluate(scope)
    }
    pub fn evaluate_to<'a, T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a, T> {
        let value = self.expression.evaluate_local(scope, ast)?.evaluate(scope)?;
        match value.downcast::<T>() {
            Err(Raw(BergError::BadType(value, expected_type))) => BergError::BadOperandType(self.position, value, expected_type).err(),
            result => result,
        }
    }
    pub fn token<'p>(&self, ast: &'p AstRef) -> &'p Token {
        self.expression.token(ast)
    }
    pub fn operator(&self) -> AstIndex {
        self.expression.operator()
    }
}

pub fn cow_range_from_utf8_lossy(input: Cow<ByteSlice>, range: ByteRange) -> Cow<str> {
    match input {
        Cow::Borrowed(bytes) => String::from_utf8_lossy(&bytes[range]),
        Cow::Owned(bytes) => match String::from_utf8_lossy(&bytes[range]) {
            Cow::Borrowed(s) => s.to_string().into(),
            Cow::Owned(s) => s.into(),
        },
    }
}
