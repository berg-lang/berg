use source::line_column::LineColumnRange;
use source::parse_result::ByteRange;
use interpreter::value::Value;
use std::fmt::Formatter;
use std::fmt::Display;
use std::fmt;
use source::parse_result::ParseResult;
use ast::AstIndex;
use ast::token::{Fixity,Token};
use ast::expression::OperandPosition::*;

#[derive(Debug,Copy,Clone,PartialEq)]
pub struct Expression(pub(crate) AstIndex);

#[derive(Debug,Copy,Clone)]
pub(crate) struct SourceExpression<'e> {
    pub(crate) parse_result: &'e ParseResult,
    pub(crate) expression: Expression,
}

#[derive(Debug,Copy,Clone)]
pub(crate) struct SourceToken<'e> {
    pub(crate) parse_result: &'e ParseResult,
    pub(crate) index: AstIndex,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum OperandType {
    Any,
    Number,
    Boolean,
    Integer,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum OperandPosition {
    Left,
    Right,
    PrefixOperand,
    PostfixOperand,
}

impl OperandType {
    pub(crate) fn matches(self, value: &Value) -> bool {
        match (self, value) {
            (OperandType::Any, _)|(OperandType::Number,&Value::Rational(_))|(OperandType::Boolean,&Value::Boolean(_)) => true,
            (OperandType::Integer,&Value::Rational(ref value)) if value.is_integer() => true,
            (OperandType::Number,_)|(OperandType::Integer,_)|(OperandType::Boolean,_) => false,
        }
    }
}

impl OperandPosition {
    pub(crate) fn get(self, expression: Expression, parse_result: &ParseResult) -> Expression {
        match self {
            Left|PostfixOperand => expression.left(parse_result),
            Right|PrefixOperand => expression.right(parse_result),
        }
    }
}

impl Display for OperandType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ast::expression::OperandType::*;
        let string = match *self {
            Any => "any",
            Number => "number",
            Boolean => "boolean",
            Integer => "integer",
        };
        write!(f, "{}", string)
    }
}

impl Display for OperandPosition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = match *self {
            Left|PostfixOperand => "left side",
            Right|PrefixOperand => "right side",
        };
        write!(f, "{}", string)
    }
}

impl Expression {
    pub(crate) fn from_source(parse_result: &ParseResult) -> Self {
        Self::find_root(parse_result, parse_result.tokens.last_index(), true)
    }

    pub(crate) fn range(self, parse_result: &ParseResult) -> ByteRange {
        let start = parse_result.token_ranges[self.start(parse_result)].start;
        let end = parse_result.token_ranges[self.end(parse_result)].end;
        start..end
    }

    pub(crate) fn operator(self) -> AstIndex {
        self.0
    }
    
    pub(crate) fn start(self, parse_result: &ParseResult) -> AstIndex {
        let token = self.token(parse_result);
        match *token {
            Token::Close{delta,..} => self.operator()-delta,
            _ => {
                let mut left = self;
                while left.token(parse_result).has_left_operand() {
                    left = left.left(parse_result);
                }
                left.operator()
            }
        }
    }

    pub(crate) fn end(self, parse_result: &ParseResult) -> AstIndex {
        let token = self.token(parse_result);
        match *token {
            Token::Open{delta,..} => self.operator()+delta,
            _ => {
                let mut right = self;
                while right.token(parse_result).has_right_operand() {
                    right = right.right(parse_result);
                }
                right.operator()
            }
        }
    }

    pub(crate) fn token<'p>(&self, parse_result: &'p ParseResult) -> &'p Token {
        &parse_result.tokens[self.operator()]
    }

    pub(crate) fn open_operator<'p>(&self, parse_result: &'p ParseResult) -> AstIndex {
        match *self.token(parse_result) {
            Token::Open{..} => self.operator(),
            Token::Close{delta,..} => self.operator()-delta,
            _ => unreachable!(),
        }
    }

    pub(crate) fn close_operator<'p>(&self, parse_result: &'p ParseResult) -> AstIndex {
        match *self.token(parse_result) {
            Token::Open{delta,..} => self.operator()+delta,
            Token::Close{..} => self.operator(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn operand(self, position: OperandPosition, parse_result: &ParseResult) -> Self {
        position.get(self, parse_result)
    }

    pub(crate) fn left(&self, parse_result: &ParseResult) -> Self {
        Self::find_root(parse_result, self.operator()-1, self.token(parse_result).fixity() == Fixity::Infix)
    }

    pub(crate) fn right(&self, parse_result: &ParseResult) -> Self {
        let mut right = self.operator()+1;
        if let Token::Open{delta,..} = parse_result.tokens[right] {
            right += delta;
        }
        // If it's infix, check for postfixes before we go further.
        if parse_result.tokens[self.operator()].fixity() == Fixity::Infix {
            while let Some(token) = parse_result.tokens.get(right + 1) {
                if token.fixity() == Fixity::Postfix {
                    match *token {
                        Token::Close{..} => break,
                        _ => { right += 1; }
                    }
                } else {
                    break;
                }
            }
        }
        Expression(right)
    }

    pub(crate) fn inner(&self, parse_result: &ParseResult) -> Self {
        Self::find_root(parse_result, self.operator()-1, true)
    }

    pub(crate) fn prev(&self) -> Self {
        Expression(self.operator()-1)
    }

    fn find_root(parse_result: &ParseResult, end: AstIndex, allow_infix_children: bool) -> Self {
        // Pass any postfixes
        let mut index = end;
        let mut has_postfix = false;
        while parse_result.tokens[index].fixity() == Fixity::Postfix {
            match parse_result.tokens[index] {
                Token::Close{delta,..} => { index -= delta; break; },
                _ => { index -= 1; has_postfix = true; },
            }
        }
        // Pass any prefixes and infixes (but not open groups--that's going too far up a level)
        while index > 0 {
            match parse_result.tokens[index-1].fixity() {
                Fixity::Infix if allow_infix_children => { index -= 1; break; },
                Fixity::Prefix => match parse_result.tokens[index-1] {
                    Token::Open{..} => break,
                    _ => { index -= 1; },
                },
                Fixity::Postfix|Fixity::Term|Fixity::Infix => break,
            }
        }
        // If there's a postfix, and no infix on the left, return the postfix.
        if has_postfix && parse_result.tokens[index].fixity() != Fixity::Infix {
            return Expression(end);
        }
        // Otherwise return the infix or the left side of the term (index).
        match parse_result.tokens[index] {
            Token::Open{delta,..} => Expression(index+delta),
            _ => Expression(index),
        }
    }
}

impl<'e> SourceExpression<'e> {
    pub(crate) fn from_source(parse_result: &'e ParseResult) -> Self {
        SourceExpression { parse_result, expression: Expression::from_source(parse_result) }
    }
    pub(crate) fn range(self) -> ByteRange {
        self.expression.range(self.parse_result)
    }

    pub(crate) fn token(&self) -> SourceToken<'e> {
        SourceToken { parse_result: self.parse_result, index: self.expression.operator() }
    }

    pub(crate) fn open_token(&self) -> SourceToken<'e> {
        SourceToken { parse_result: self.parse_result, index: self.expression.open_operator(self.parse_result) }
    }

    pub(crate) fn close_token(&self) -> SourceToken<'e> {
        SourceToken { parse_result: self.parse_result, index: self.expression.close_operator(self.parse_result) }
    }

    pub(crate) fn operand(&self, operand: OperandPosition) -> Self {
        SourceExpression { parse_result: self.parse_result, expression: self.expression.operand(operand, self.parse_result) }
    }

    pub(crate) fn inner(&self) -> Self {
        SourceExpression { parse_result: self.parse_result, expression: self.expression.inner(self.parse_result) }
    }

    pub(crate) fn left(&self) -> Self {
        SourceExpression { parse_result: self.parse_result, expression: self.expression.left(self.parse_result) }
    }

    pub(crate) fn right(&self) -> Self {
        SourceExpression { parse_result: self.parse_result, expression: self.expression.right(self.parse_result) }
    }

    pub(crate) fn prev(&self) -> Self {
        SourceExpression { parse_result: self.parse_result, expression: self.expression.prev() }
    }

    pub(crate) fn line_column_range(&self) -> LineColumnRange {
        self.parse_result.char_data().range(&self.range())
    }
}

impl<'e> SourceToken<'e> {
    pub(crate) fn token(&self) -> &Token {
        &self.parse_result.tokens[self.index]
    }

    pub(crate) fn range(&self) -> ByteRange {
        self.parse_result.token_ranges[self.index].clone()
    }

    pub(crate) fn line_column_range(&self) -> LineColumnRange {
        self.parse_result.char_data().range(&self.range())
    }
}