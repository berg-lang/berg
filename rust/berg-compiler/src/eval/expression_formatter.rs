use eval::Expression;
use std::fmt;
use syntax::{AstRef, ExpressionBoundary, Fixity, Token};
use syntax::identifiers::SEMICOLON;

pub struct ExpressionFormatter<'p, 'a: 'p>(pub Expression, pub &'p AstRef<'a>);

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

pub struct ExpressionTreeFormatter<'p, 'a: 'p>(
    pub Expression,
    pub &'p AstRef<'a>,
    pub usize,
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

