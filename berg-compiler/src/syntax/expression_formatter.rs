use crate::syntax::identifiers::SEMICOLON;
use crate::syntax::{ExpressionTreeWalker, ExpressionBoundary, Fixity, Token, ExpressionToken, OperatorToken};
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct ExpressionFormatter;

#[derive(Copy, Clone, Debug)]
pub struct ExpressionTreeFormatter {
    starting_depth: usize,
}

impl<'p, 'a: 'p, Context: Copy + Clone + fmt::Debug> ExpressionTreeWalker<'p, 'a, Context> {
    pub fn format(self) -> ExpressionTreeWalker<'p, 'a, ExpressionFormatter> {
        self.with_context(ExpressionFormatter)
    }
    pub fn format_tree(self) -> ExpressionTreeWalker<'p, 'a, ExpressionTreeFormatter> {
        self.with_context(ExpressionTreeFormatter {
            starting_depth: self.depth(),
        })
    }
}

impl<'p, 'a: 'p> ExpressionTreeWalker<'p, 'a, ExpressionFormatter> {
    fn boundary_strings(self) -> (&'static str, &'static str) {
        let boundary = match self.open_token() {
            ExpressionToken::Open { boundary, .. } => boundary,
            ExpressionToken::OpenBlock { index, .. } => self.ast().blocks[index].boundary,
            _ => unreachable!(),
        };
        match boundary {
            ExpressionBoundary::AutoBlock => ("prec{", "}"),
            ExpressionBoundary::PrecedenceGroup => ("prec(", ")"),
            ExpressionBoundary::CompoundTerm => ("term(", ")"),
            ExpressionBoundary::Parentheses => ("(", ")"),
            ExpressionBoundary::CurlyBraces => ("{ ", " }"),
            ExpressionBoundary::Source => ("source{ ", " }"),
            ExpressionBoundary::Root => ("root{ ", " }"),
        }
    }
}

impl<'p, 'a: 'p> fmt::Display for ExpressionTreeWalker<'p, 'a, ExpressionFormatter> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token = self.token();
        let string = self.token_string();
        match token.fixity() {
            Fixity::Infix => {
                let left = self.left_expression();
                let right = self.right_expression();
                match token {
                    Token::Operator(OperatorToken::InfixOperator(SEMICOLON)) => write!(f, "{}{} {}", left, string, right),
                    Token::Operator(OperatorToken::NewlineSequence) => write!(f, "{}\\n {}", left, right),
                    _ => write!(f, "{} {} {}", left, string, right),
                }
            }
            Fixity::Prefix => {
                let right = self.right_expression();
                if self.ast().tokens[self.root_index() - 1].has_left_operand() {
                    write!(f, " {}{}", string, right)
                } else {
                    write!(f, "{}{}", string, right)
                }
            }
            Fixity::Postfix => {
                let left = self.left_expression();
                if self.ast().tokens[self.root_index() + 1].has_right_operand() {
                    write!(f, " {}{}", left, string)
                } else {
                    write!(f, "{}{}", left, string)
                }
            }
            Fixity::Term => write!(f, "{}", token.to_string(self.ast())),
            Fixity::Open | Fixity::Close => {
                let (open, close) = self.boundary_strings();
                let inner = self.inner_expression();
                write!(f, "{}{}{}", open, inner, close)
            }
        }
    }
}

impl<'p, 'a: 'p> ExpressionTreeWalker<'p, 'a, ExpressionTreeFormatter> {
    fn fmt_self(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token = self.token();
        match token.fixity() {
            Fixity::Open | Fixity::Close => write!(
                f,
                "{:?} at {}-{}",
                token,
                self.open_operator(),
                self.close_operator()
            )?,
            Fixity::Prefix | Fixity::Postfix | Fixity::Infix | Fixity::Term => {
                write!(f, "{:?} at {}", token, self.root_index())?
            }
        }
        writeln!(f, ": {}", self.format())
    }
}

impl<'p, 'a: 'p> fmt::Display for ExpressionTreeWalker<'p, 'a, ExpressionTreeFormatter> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_self(f)?;
        match self.token().fixity() {
            Fixity::Open | Fixity::Close => self.inner_expression().fmt(f),
            Fixity::Infix => {
                self.left_expression().fmt(f)?;
                self.right_expression().fmt(f)
            }
            Fixity::Prefix => self.right_expression().fmt(f),
            Fixity::Postfix => self.left_expression().fmt(f),
            Fixity::Term => Ok(()),
        }
    }
}
