use crate::syntax::Fixity::*;
use crate::syntax::{
    AstIndex, AstRef, ByteRange, ExpressionBoundary, Fixity,
    OperandPosition, SourceReconstruction, Token,
};
use std::fmt;

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

    pub(crate) fn open_operator(self, ast: &AstRef) -> AstIndex {
        match *self.token(ast) {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => self.operator() - delta,
            _ => self.operator(),
        }
    }

    pub(crate) fn close_operator(self, ast: &AstRef) -> AstIndex {
        match *self.token(ast) {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => self.operator() + delta,
            _ => self.operator(),
        }
    }

    pub(crate) fn boundary(self, ast: &AstRef) -> ExpressionBoundary {
        match *self.open_token(ast) {
            Token::Open { boundary, .. } => boundary,
            Token::OpenBlock { index, .. } => ast.blocks()[index].boundary,
            _ => unreachable!(),
        }
    }

    pub(crate) fn left_expression(self, ast: &AstRef) -> Self {
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

    pub(crate) fn right_expression(self, ast: &AstRef) -> Self {
        let start = self.operator() + 1;

        match self.token(ast).fixity() {
            // If this is prefix, it cannot have postfix or infix children, so its immediate right is the child.
            Fixity::Prefix => return Expression(start),
            // If this is a group term, APPLY inner() and return.
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

    pub(crate) fn parent(self, ast: &AstRef) -> Self {
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

    pub(crate) fn operand_position(self, ast: &AstRef) -> OperandPosition {
        use self::OperandPosition::*;
        let parent = self.parent(ast);
        match parent.token(ast).fixity() {
            Prefix | Open => PrefixOperand,
            Postfix | Close => PostfixOperand,
            Infix if self.0 < parent.0 => Left,
            Infix => Right,
            Term => unreachable!(),
        }
    }

    pub(crate) fn inner_expression<'a>(self, ast: &AstRef<'a>) -> Self {
        Expression(self.close_operator(ast)).left_expression(ast)
    }

    pub(crate) fn to_string<'p, 'a: 'p>(self, ast: &'p AstRef<'a>) -> String {
        SourceReconstruction::new(ast, self.range(ast)).to_string()
    }
}

impl Operand {
    pub fn token<'p>(&self, ast: &'p AstRef) -> &'p Token {
        self.expression.token(ast)
    }
    pub fn operator(self) -> AstIndex {
        self.expression.operator()
    }
}
