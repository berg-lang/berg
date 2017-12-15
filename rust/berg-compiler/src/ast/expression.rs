use compiler::source_data::ByteIndex;
use std::ops::Range;
use compiler::source_data::ParseData;
use ast::AstIndex;
use ast::token::{Fixity,Token};

#[derive(Debug,Copy,Clone,PartialEq)]
pub(crate) struct Expression(pub(crate) AstIndex);

impl Expression {
    pub(crate) fn from_source(parse_data: &ParseData) -> Expression {
        Self::find_root(parse_data, parse_data.tokens.len()-1, true)
    }

    pub(crate) fn debug_string(&self, parse_data: &ParseData) -> String {
        format!("{:?} ({:?}..{:?} [{:?}])", parse_data.token_string(self.operator()), self.start(parse_data).0, self.end(parse_data).0, self.operator().0)
    }

    pub(crate) fn range(self, parse_data: &ParseData) -> Range<ByteIndex> {
        let start = parse_data.token_ranges[self.start(parse_data)].start;
        let end = parse_data.token_ranges[self.end(parse_data)].end;
        start..end
    }

    pub(crate) fn operator_range(self, parse_data: &ParseData) -> Range<ByteIndex> {
        parse_data.token_ranges[self.operator()].clone()
    }

    pub(crate) fn operator(self) -> AstIndex {
        self.0
    }

    pub(crate) fn start(self, parse_data: &ParseData) -> AstIndex {
        let token = self.token(parse_data);
        match *token {
            Token::Close(_,delta) => self.operator()-delta,
            _ => {
                let mut left = self;
                while left.token(parse_data).has_left_operand() {
                    left = left.left(parse_data);
                }
                left.operator()
            }
        }
    }

    pub(crate) fn end(self, parse_data: &ParseData) -> AstIndex {
        let token = self.token(parse_data);
        match *token {
            Token::Open(_,delta) => self.operator()+delta,
            _ => {
                let mut right = self;
                while right.token(parse_data).has_right_operand() {
                    right = right.right(parse_data);
                }
                right.operator()
            }
        }
    }

    pub(crate) fn token<'p>(&self, parse_data: &'p ParseData) -> &'p Token {
        &parse_data.tokens[self.operator()]
    }

    pub(crate) fn left(&self, parse_data: &ParseData) -> Expression {
        Self::find_root(parse_data, self.operator()-1, self.token(parse_data).fixity() == Fixity::Infix)
    }

    pub(crate) fn right(&self, parse_data: &ParseData) -> Expression {
        let mut right = self.operator()+1;
        if let Token::Open(_,delta) = parse_data.tokens[right] {
            right += delta;
        }
        // If it's infix, check for postfixes before we go further.
        if parse_data.tokens[self.operator()].fixity() == Fixity::Infix {
            while let Some(token) = parse_data.tokens.get(right + 1) {
                if token.fixity() == Fixity::Postfix {
                    match *token {
                        Token::Close(..) => break,
                        _ => { right += 1; }
                    }
                } else {
                    break;
                }
            }
        }
        Expression(right)
    }

    pub(crate) fn inner(&self, parse_data: &ParseData) -> Expression {
        Self::find_root(parse_data, self.operator()-1, true)
    }

    fn find_root(parse_data: &ParseData, end: AstIndex, allow_infix_children: bool) -> Expression {
        // Pass any postfixes
        let mut index = end;
        let mut has_postfix = false;
        while parse_data.tokens[index].fixity() == Fixity::Postfix {
            match parse_data.tokens[index] {
                Token::Close(_,delta) => { index -= delta; break; },
                _ => { index -= 1; has_postfix = true; },
            }
        }
        // Pass any prefixes and infixes (but not open groups--that's going too far up a level)
        while index > 0 {
            match parse_data.tokens[index-1].fixity() {
                Fixity::Infix if allow_infix_children => { index -= 1; break; },
                Fixity::Prefix => match parse_data.tokens[index-1] {
                    Token::Open(..) => break,
                    _ => { index -= 1; },
                },
                Fixity::Postfix|Fixity::Term|Fixity::Infix => break,
            }
        }
        // If there's a postfix, and no infix on the left, return the postfix.
        if has_postfix && parse_data.tokens[index].fixity() != Fixity::Infix {
            return Expression(end);
        }
        // Otherwise return the infix or the left side of the term (index).
        match parse_data.tokens[index] {
            Token::Open(_,delta) => Expression(index+delta),
            _ => Expression(index),
        }
    }
}
