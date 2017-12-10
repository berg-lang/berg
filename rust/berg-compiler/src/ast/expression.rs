use compiler::source_data::ByteIndex;
use std::ops::Range;
use compiler::source_data::ParseData;
use ast::AstIndex;
use ast::token::Fixity;
use ast::token::Token;
use ast::expression::Expression::*;

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum Expression {
    Infix { start: AstIndex, end: AstIndex, operator: AstIndex },
    Postfix { start: AstIndex, end: AstIndex },
    Prefix { start: AstIndex, end: AstIndex },
    Group { start: AstIndex, end: AstIndex },
    Term { index: AstIndex },
}

impl Expression {
    pub(crate) fn from_source(parse_data: &ParseData) -> Expression {
        let start = AstIndex(0);
        let end = parse_data.tokens.len()-1;
        Self::from_range(parse_data, start, end)
    }

    pub(crate) fn debug_string(&self, parse_data: &ParseData) -> String {
        format!("{:?} ({:?}..{:?} [{:?}])", parse_data.token_string(self.operator()), self.start().0, self.end().0, self.operator().0)
    }

    pub(crate) fn from_range(parse_data: &ParseData, start: AstIndex, end: AstIndex) -> Expression {
        if start == end { return Term { index: start }; }

        // Skip to the beginning of the last term
        let mut index = end;
        let mut has_postfix = false;
        while index > start && parse_data.tokens[index].fixity() == Fixity::Postfix {
            match parse_data.tokens[index] {
                Token::Close(_,delta) => { index -= delta; break; },
                _ => { index -= 1; has_postfix = true; }
            }
        }

        // Skip any prefixes before the last term
        let mut has_prefix = false;
        while index > start && parse_data.tokens[index-1].fixity() == Fixity::Prefix {
            index -= 1;
            has_prefix = true;
        }

        // Return infix > postfix > prefix/term
        if index > start && parse_data.tokens[index-1].fixity() == Fixity::Infix {
            Infix { start, end, operator: index-1 }
        } else if has_postfix {
            Postfix { start, end }
        } else if has_prefix {
            Prefix { start, end }
        } else {
            assert!(match parse_data.tokens[start] { Token::Open(..,delta) => start+delta == end, _ => false });
            Group { start, end }
        }
    }

    pub(crate) fn range(&self, parse_data: &ParseData) -> Range<ByteIndex> {
        match *self {
            Infix{start,end,..}|Prefix{start,end}|Postfix{start,end}|Group{start,end} => {
                let start = parse_data.token_ranges[start].start;
                let end = parse_data.token_ranges[end].end;
                start..end
            },
            Term{index} => parse_data.token_ranges[index].clone(),
        }
    }

    pub(crate) fn operator_range(&self, parse_data: &ParseData) -> Range<ByteIndex> {
        parse_data.token_ranges[self.operator()].clone()
    }

    pub(crate) fn operator(&self) -> AstIndex {
        match *self {
            Infix{operator,..} => operator,
            Prefix{start,..}|Group{start,..} => start,
            Postfix{end,..} => end,
            Term{index} => index,
        }
    }

    pub(crate) fn start(&self) -> AstIndex {
        match *self {
            Infix{start,..}|Prefix{start,..}|Postfix{start,..}|Group{start,..} => start,
            Term{index} => index,
        }
    }

    pub(crate) fn end(&self) -> AstIndex {
        match *self {
            Infix{end,..}|Prefix{end,..}|Postfix{end,..}|Group{end,..} => end,
            Term{index} => index,
        }
    }

    pub(crate) fn token<'p>(&self, parse_data: &'p ParseData) -> &'p Token {
        &parse_data.tokens[self.operator()]
    }

    pub(crate) fn left(&self, parse_data: &ParseData) -> Expression {
        match *self {
            Infix{start,operator,..} => Expression::from_range(parse_data,start,operator-1),
            Postfix{start,end} => Expression::from_range(parse_data,start,end-1),
            _ => unreachable!(),
        }
    }

    pub(crate) fn right(&self, parse_data: &ParseData) -> Expression {
        match *self {
            Infix{operator,end,..} => Expression::from_range(parse_data,operator+1,end),
            Prefix{start,end} => Expression::from_range(parse_data,start+1,end),
            _ => unreachable!(),
        }
    }

    pub(crate) fn inner(&self, parse_data: &ParseData) -> Expression {
        match *self {
            Group{start,end} => Expression::from_range(parse_data,start+1,end-1),
            _ => unreachable!(),
        }
    }
}
