use std::fmt;

use super::expression_tree::ExpressionTreeWalker;
use super::identifiers::SEMICOLON;
use super::token::{ExpressionBoundary, ExpressionToken, Fixity, OperatorToken, TermToken, Token};

#[derive(Copy, Clone, Debug)]
pub struct ExpressionFormatter;

#[derive(Copy, Clone, Debug)]
pub struct ExpressionTreeFormatter {
    pub starting_depth: usize,
}

impl<'a> ExpressionTreeWalker<'a, ExpressionFormatter> {
    fn boundary_strings(self) -> (&'static str, &'static str) {
        let boundary = match self.open_token() {
            ExpressionToken::Open(_, boundary, _) => boundary,
            _ => unreachable!(),
        };
        use ExpressionBoundary::*;
        match boundary {
            AutoBlock => ("auto{", "}"),
            IndentedBlock => ("indent{", "}"),
            IndentedExpression => ("indent(", ")"),
            PrecedenceGroup => ("prec(", ")"),
            CompoundTerm => ("term(", ")"),
            Parentheses => ("(", ")"),
            CurlyBraces => ("{ ", " }"),
            Source => ("source{ ", " }"),
            Root => ("root{ ", " }"),
        }
    }
}

impl<'a> fmt::Display for ExpressionTreeWalker<'a, ExpressionFormatter> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ExpressionToken::*;
        use OperatorToken::*;
        use TermToken::*;
        use Token::*;
        let token = self.token();
        let string = self.token_string();
        match token {
            Expression(token) => match token {
                Term(token) => match token {
                    MissingExpression => write!(f, "<missing>"),
                    _ => write!(f, "{}", token.to_string(self.ast())),
                },
                PrefixOperator(_) => {
                    let right = self.right_expression();
                    if self.ast().tokens[self.root_index() - 1].has_left_operand() {
                        write!(f, " {}{}", string, right)
                    } else {
                        write!(f, "{}{}", string, right)
                    }
                }
                Open(..) => {
                    let (open, close) = self.boundary_strings();
                    let inner = self.inner_expression();
                    write!(f, "{}{}{}", open, inner, close)
                }
            },
            Operator(token) => match token {
                PostfixOperator(_) => {
                    let left = self.left_expression();
                    if self.ast().tokens[self.root_index() + 1].has_right_operand() {
                        write!(f, " {}{}", left, string)
                    } else {
                        write!(f, "{}{}", left, string)
                    }
                }
                Close(..) | CloseBlock(..) => unreachable!(),
                InfixOperator(SEMICOLON) => write!(
                    f,
                    "{}{} {}",
                    self.left_expression(),
                    string,
                    self.right_expression()
                ),
                InfixOperator(_) | InfixAssignment(_) => write!(
                    f,
                    "{} {} {}",
                    self.left_expression(),
                    string,
                    self.right_expression()
                ),
            },
        }
    }
}

impl<'a> ExpressionTreeWalker<'a, ExpressionTreeFormatter> {
    fn fmt_self(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token = self.token();
        match token.fixity() {
            Fixity::Open | Fixity::Close => write!(
                f,
                "{:?} at token range {}-{}",
                token,
                self.open_operator(),
                self.close_operator(),
            )?,
            Fixity::Prefix | Fixity::Postfix | Fixity::Infix | Fixity::Term => {
                write!(f, "{:?} at {}", token, self.root_index())?
            }
        }
        writeln!(f, ": {}", self.format())
    }
}

impl<'a> fmt::Display for ExpressionTreeWalker<'a, ExpressionTreeFormatter> {
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
