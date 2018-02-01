use eval::ScopeRef;
use num::BigRational;
use parser::{ByteRange, ByteSlice};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use syntax::{AstIndex, AstRef, ExpressionBoundary, FieldIndex, Fixity, IdentifierIndex,
             OperandPosition, Token};
use syntax::Fixity::*;
use syntax::OperandPosition::*;
use syntax::identifiers::{CALL, COLON, DASH_DASH, EMPTY_STRING, NEWLINE, PLUS_PLUS, SEMICOLON};
use util::try_from::TryFrom;
use util::type_name::TypeName;
use value::{BergError, BergErrorStack, BergResult, BergVal, BergValue, ErrorCode};

#[derive(Copy, Clone, PartialEq)]
pub struct Expression(pub AstIndex);

#[derive(Debug, Clone)]
pub struct BlockClosure<'a>(Expression, ScopeRef<'a>);

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
    pub fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        println!("Evaluate {} ...", ExpressionFormatter(self, ast));
        use syntax::ExpressionBoundaryError::*;
        use syntax::Token::*;
        let result = match *self.token(ast) {
            IntegerLiteral(literal) => Ok(BigRational::from_str(ast.literal_string(literal))
                .unwrap()
                .into()),
            FieldReference(field) => self.result(
                match scope.field(field, ast) {
                    Ok(result) => result.clone(),
                    Err(error) => Err(error),
                },
                ast,
            ),
            ErrorTerm(ErrorCode::IdentifierStartsWithNumber) => {
                self.err(BergError::IdentifierStartsWithNumber, ast)
            }
            ErrorTerm(ErrorCode::UnsupportedCharacters) => {
                self.err(BergError::UnsupportedCharacters, ast)
            }
            ErrorTerm(ErrorCode::InvalidUtf8) => self.err(BergError::InvalidUtf8, ast),
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
            } => self.err(ast.open_error().clone(), ast),
            OpenBlock {
                error: OpenWithoutClose,
                ..
            }
            | Open {
                error: OpenWithoutClose,
                ..
            } => self.err(BergError::OpenWithoutClose, ast),
            OpenBlock {
                error: CloseWithoutOpen,
                ..
            }
            | Open {
                error: CloseWithoutOpen,
                ..
            } => self.err(BergError::CloseWithoutOpen, ast),
            Open { error: None, .. } => self.inner_expression(ast).evaluate(scope, ast),
            OpenBlock { error: None, .. } => self.close_over(scope).ok(),
            Close { .. } | CloseBlock { .. } | RawIdentifier(_) | ErrorTerm(_) => unreachable!(),
        };
        println!("Result of {}: {:?}", ExpressionFormatter(self, ast), result);
        result
    }

    fn close_over<'a>(self, scope: &mut ScopeRef<'a>) -> BlockClosure<'a> {
        BlockClosure(self, scope.clone())
    }

    fn result<'a, T>(self, result: BergResult<'a, T>, ast: &AstRef<'a>) -> BergResult<'a, T> {
        match result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.unwind_error(ast.clone(), self)),
        }
    }

    fn err<'a, T>(self, error: BergError<'a>, ast: &AstRef<'a>) -> BergResult<'a, T> {
        let error: BergVal<'a> = error.into();
        Err(error.unwind_error(ast.clone(), self))
    }

    fn assignment_target<'a>(
        self,
        operand: Operand,
        ast: &AstRef<'a>,
    ) -> BergResult<'a, FieldIndex> {
        use syntax::Token::*;
        match *operand.token(ast) {
            PrefixOperator(COLON) => {
                let colon_operand = operand.expression.prefix_operand(ast)?;
                match *colon_operand.token(ast) {
                    FieldReference(field) => Ok(field),
                    _ => colon_operand.err(BergError::AssignmentTargetMustBeIdentifier, ast),
                }
            }
            FieldReference(field) => Ok(field),
            _ => operand.err(BergError::AssignmentTargetMustBeIdentifier, ast),
        }
    }

    fn evaluate_semicolon<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let left = self.left_operand(ast)?.expression;
        let left_value = left.evaluate(scope, ast)?;

        // If the left hand side is a semicolon with a missing expression between,
        // raise MissingExpression.
        if let BergVal::Nothing = left_value {
            if left.operator() == self.operator() - 2 {
                if let Token::InfixOperator(SEMICOLON) = *left.token(ast) {
                    let immediate_left = Expression(left.operator() + 1);
                    if let Token::MissingExpression = *immediate_left.token(ast) {
                        return self.err(BergError::MissingOperand, ast);
                    }
                }
            }
        }
        let right = Operand {
            position: OperandPosition::Right,
            expression: self.right_expression(ast),
        };
        let result = left_value.infix(SEMICOLON, scope, right, ast);
        self.result(result, ast)
    }

    fn evaluate_infix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let left = self.left_operand(ast)?.expression;
        let right = self.right_operand(ast)?;
        let left = left.evaluate(scope, ast)?;
        self.result(left.infix(operator, scope, right, ast), ast)
    }

    fn evaluate_infix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let left = self.left_operand(ast)?;
        let name = self.assignment_target(left, ast)?;
        let value = match operator {
            EMPTY_STRING => self.right_operand(ast)?.evaluate(scope, ast),
            _ => self.evaluate_infix(operator, scope, ast),
        };
        match scope.set_field(name, value, ast) {
            Ok(()) => Ok(BergVal::Nothing),
            Err(error) => self.result(Err(error), ast),
        }
    }

    fn evaluate_prefix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let right = self.prefix_operand(ast)?.evaluate(scope, ast)?;
        self.result(right.prefix(operator, scope), ast)
    }

    fn evaluate_postfix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let left = self.postfix_operand(ast)?.evaluate(scope, ast)?;
        self.result(left.postfix(operator, scope), ast)
    }

    fn evaluate_prefix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.prefix_operand(ast)?;
        let name = self.assignment_target(operand, ast)?;
        let value = operand.evaluate(scope, ast)?.prefix(operator, scope);
        self.result(scope.set_field(name, value, ast), ast)?;
        Ok(BergVal::Nothing)
    }

    fn evaluate_postfix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.postfix_operand(ast)?;
        let name = self.assignment_target(operand, ast)?;
        let value = operand.evaluate(scope, ast)?.postfix(operator, scope);
        self.result(scope.set_field(name, value, ast), ast)?;
        Ok(BergVal::Nothing)
    }

    fn evaluate_declare<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let operand = self.prefix_operand(ast)?;
        let name = self.assignment_target(operand, ast)?;
        self.result(scope.declare_field(name, ast), ast)?;
        operand.evaluate(scope, ast)
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
    ) -> BergResult<'a, Operand> {
        let expression = position.get(self, ast);
        match *expression.token(ast) {
            Token::MissingExpression => self.err(BergError::MissingOperand, ast),
            _ => Ok(Operand {
                expression,
                position,
            }),
        }
    }

    pub(crate) fn left_operand<'a>(self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.operand(OperandPosition::Left, ast)
    }
    pub(crate) fn right_operand<'a>(self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.operand(OperandPosition::Right, ast)
    }
    pub(crate) fn prefix_operand<'a>(self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.operand(OperandPosition::PrefixOperand, ast)
    }
    pub(crate) fn postfix_operand<'a>(self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
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

impl Operand {
    pub fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        self.expression.evaluate(scope, ast)
    }
    pub fn evaluate_to<'a, T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a, T> {
        let result = self.expression.evaluate(scope, ast);
        match result?.downcast::<T>() {
            // Mark the result type with the operand position for happy error messages
            Err(BergVal::BergErrorStack(BergErrorStack {
                error: BergError::BadType(value, expected_type),
                stack,
            })) => {
                let error = BergError::BadOperandType(self.position, value, expected_type);
                Err(BergErrorStack { error, stack }.into())
            }
            result => result,
        }
    }
    pub fn err<'a, T>(&self, error: BergError<'a>, ast: &AstRef<'a>) -> BergResult<'a, T> {
        self.expression.err(error, ast)
    }
    pub fn left_operand<'a>(&self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.expression.left_operand(ast)
    }
    pub fn right_operand<'a>(&self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.expression.right_operand(ast)
    }
    pub fn prefix_operand<'a>(&self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.expression.prefix_operand(ast)
    }
    pub fn postfix_operand<'a>(&self, ast: &AstRef<'a>) -> BergResult<'a, Operand> {
        self.expression.postfix_operand(ast)
    }
    pub fn range(&self, ast: &AstRef) -> ByteRange {
        self.expression.range(ast)
    }
    pub fn token<'p>(&self, ast: &'p AstRef) -> &'p Token {
        self.expression.token(ast)
    }
    pub fn to_string<'a>(&self, ast: &'a AstRef) -> Cow<'a, str> {
        self.expression.to_string(ast)
    }
}

pub(crate) struct ExpressionFormatter<'p, 'a: 'p>(pub(crate) Expression, pub(crate) &'p AstRef<'a>);

impl<'p, 'a: 'p> ExpressionFormatter<'p, 'a> {
    fn boundary_strings(&self) -> (&str, &str) {
        let ExpressionFormatter(ref expression, ast) = *self;
        let boundary = match *Expression(expression.open_operator(ast)).token(ast) {
            Token::Open { boundary, .. } => boundary,
            Token::OpenBlock { index, .. } => ast.blocks()[index].boundary,
            _ => unreachable!(),
        };
        match boundary {
            ExpressionBoundary::PrecedenceGroup => ("prec(", ")"),
            ExpressionBoundary::CompoundTerm => ("term(", ")"),
            ExpressionBoundary::Parentheses => ("(", ")"),
            ExpressionBoundary::CurlyBraces => ("{ ", " }"),
            ExpressionBoundary::Source => ("source{ ", " }"),
            ExpressionBoundary::Root => ("root{ ", " }"),
        }
    }
}

impl<'p, 'a: 'p> fmt::Display for ExpressionFormatter<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ExpressionFormatter(ref expression, ast) = *self;
        let token = expression.token(ast);
        let string = token.to_string(ast);
        match token.fixity() {
            Fixity::Infix => {
                let left = ExpressionFormatter(expression.left_expression(ast), ast);
                let right = ExpressionFormatter(expression.right_expression(ast), ast);
                match *token {
                    Token::InfixOperator(SEMICOLON) => write!(f, "{}{} {}", left, string, right),
                    Token::NewlineSequence => write!(f, "{}\\n {}", left, right),
                    _ => write!(f, "{} {} {}", left, string, right),
                }
            },
            Fixity::Prefix => {
                let right = ExpressionFormatter(expression.right_expression(ast), ast);
                if ast.tokens()[expression.operator() - 1].has_left_operand() {
                    write!(f, " {}{}", string, right)
                } else {
                    write!(f, "{}{}", string, right)
                }
            },
            Fixity::Postfix => {
                let left = ExpressionFormatter(expression.left_expression(ast), ast);
                if ast.tokens()[expression.operator() + 1].has_right_operand() {
                    write!(f, " {}{}", left, string)
                } else {
                    write!(f, "{}{}", left, string)
                }
            },
            Fixity::Term => write!(f, "{}", token.to_string(ast)),
            Fixity::Open | Fixity::Close => {
                let (open, close) = self.boundary_strings();
                let inner = ExpressionFormatter(expression.inner_expression(ast), ast);
                write!(f, "{}{}{}", open, inner, close)
            },
        }
    }
}

pub(crate) struct ExpressionTreeFormatter<'p, 'a: 'p>(
    pub(crate) Expression,
    pub(crate) &'p AstRef<'a>,
    pub(crate) usize,
);

impl<'p, 'a: 'p> ExpressionTreeFormatter<'p, 'a> {
    fn left(&self) -> Self {
        let ExpressionTreeFormatter(ref expression, ast, level) = *self;
        ExpressionTreeFormatter(expression.left_expression(ast), ast, level + 1)
    }
    fn right(&self) -> Self {
        let ExpressionTreeFormatter(ref expression, ast, level) = *self;
        ExpressionTreeFormatter(expression.right_expression(ast), ast, level + 1)
    }
    fn inner(&self) -> Self {
        let ExpressionTreeFormatter(ref expression, ast, level) = *self;
        ExpressionTreeFormatter(expression.inner_expression(ast), ast, level + 1)
    }
    fn fmt_self(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ExpressionTreeFormatter(expression, ast, level) = *self;
        let token = expression.token(ast);
        write!(f, "{:level$}", "  ", level=level)?;
        match token.fixity() {
            Fixity::Open | Fixity::Close => write!(
                f,
                "{:?} at {}-{}",
                token,
                expression.open_operator(ast),
                expression.close_operator(ast)
            )?,
            Fixity::Prefix | Fixity::Postfix | Fixity::Infix | Fixity::Term => {
                write!(f, "{:?} at {}", token, expression.operator())?
            }
        }
        writeln!(f, ": {}", ExpressionFormatter(expression, ast))
    }
}

impl<'p, 'a: 'p> fmt::Display for ExpressionTreeFormatter<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ExpressionTreeFormatter(expression, ast, _) = *self;
        self.fmt_self(f)?;
        match expression.token(ast).fixity() {
            Fixity::Open | Fixity::Close => self.inner().fmt(f),
            Fixity::Infix => {
                self.left().fmt(f)?;
                self.right().fmt(f)
            }
            Fixity::Prefix => self.right().fmt(f),
            Fixity::Postfix => self.left().fmt(f),
            Fixity::Term => Ok(()),
        }
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

impl<'a> fmt::Display for BlockClosure<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string(&self.1.ast()))
    }
}

impl<'a> BlockClosure<'a> {
    fn evaluate(mut self) -> BergResult<'a> {
        let BlockClosure(expression, ref mut scope) = self;
        let ast = scope.ast();
        let mut child_block = match *expression.token(&ast) {
            Token::OpenBlock { index, .. } => scope.create_child_block(index),
            _ => unreachable!(),
        };
        expression
            .inner_expression(&ast)
            .evaluate(&mut child_block, &ast)
    }
}

impl<'a> TypeName for BlockClosure<'a> {
    const TYPE_NAME: &'static str = "BlockClosure";
}

impl<'a> BergValue<'a> for BlockClosure<'a> {
    fn complete(self) -> BergResult<'a> {
        self.evaluate()?.complete()
    }

    #[allow(unused_variables)]
    fn unwind_error(self, ast: AstRef<'a>, expression: Expression) -> BergVal<'a> {
        self.into()
    }

    fn ok<E>(self) -> Result<BergVal<'a>, E> {
        Ok(self.into())
    }
    
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        self.evaluate()?.infix(operator, scope, right, ast)
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        self.evaluate()?.postfix(operator, scope)
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        self.evaluate()?.prefix(operator, scope)
    }
}

impl<'a> From<BlockClosure<'a>> for BergVal<'a> {
    fn from(from: BlockClosure<'a>) -> Self {
        BergVal::BlockClosure(from)
    }
}

impl<'a> TryFrom<BergVal<'a>> for BlockClosure<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::BlockClosure(closure) => Ok(closure),
            _ => Err(from),
        }
    }
}
