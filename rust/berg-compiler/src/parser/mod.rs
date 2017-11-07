mod scanner;
mod tokenizer;

use ast::AstDelta;
use ast::AstIndex;
use ast::token::Token;
use ast::token::Token::*;
use compiler::Compiler;
use compiler::compile_errors;
use compiler::source_data::{ByteIndex,ByteRange,ParseData,SourceIndex};
use indexed_vec::{Delta,IndexedVec};

pub(super) fn parse<'c>(compiler: &Compiler<'c>, source: SourceIndex) -> ParseData
{
    let parser = Parser {
        tokens: Default::default(),
        token_ranges: Default::default(),
        open_groups: Default::default(),
        compiler,
        source,
    };
    parser.parse()
}

struct Parser<'p,'c:'p> {
    tokens: IndexedVec<Token,AstIndex>,
    token_ranges: IndexedVec<ByteRange,AstIndex>,
    open_groups: Vec<AstIndex>,
    compiler: &'p Compiler<'c>,
    source: SourceIndex,
}

impl<'p,'c:'p> Parser<'p,'c> {

    fn parse(mut self) -> ParseData {
        // Loop through tokens, inserting term, then operator, then term, then operator ...
        let mut need_operand = true;
        let (identifiers, literals) = tokenizer::tokenize(self.compiler, self.source, |token, range| {
            self.insert_missing_expression_or_infix(need_operand, token.has_left_operand(), range.start);
            need_operand = token.has_right_operand();
            self.insert_token(token, range);
        });

        let end = match self.token_ranges.last() { Some(range) => range.end, None => ByteIndex(0) };
        self.insert_missing_expression_or_infix(need_operand, true, end);

        self.close_all();

        let char_data = Default::default();
        ParseData { char_data, identifiers, literals, tokens: self.tokens, token_ranges: self.token_ranges }
    }

    fn push(&mut self, token: Token, range: ByteRange) {
        self.tokens.push(token);
        self.token_ranges.push(range);
    }

    fn next_index(&self) -> AstIndex { self.tokens.len() }

    fn insert_missing_expression_or_infix(&mut self, need_operand: bool, has_left_operand: bool, location: ByteIndex) {
        // Put a MissingExpression or MissingInfix in between if we're missing something.
        match (need_operand, has_left_operand) {
            (true, true) => self.push(MissingExpression, location..location),
            (false, false) => self.push(MissingInfix, location..location),
            (true,false)|(false,true) => {}
        }
    }

    fn insert_token(&mut self, mut token: Token, range: ByteRange) {
        // First insert any extra close parens and push open parens on the list
        match token {
            OpenParen(_)|OpenCompoundTerm(_) => {
                let next_index = self.next_index();
                self.open_groups.push(next_index);
            },
            CloseParen(ref mut close_delta) => *close_delta = self.close_paren(&range),
            CloseCompoundTerm(ref mut close_delta) => {
                if let Some(delta) = self.close_compound_term() {
                    *close_delta = delta;
                } else {
                    return;
                }
            },
            _ => {},
        }

        // Then insert the token
        self.push(token, range);
    }

    fn close_paren(&mut self, range: &ByteRange) -> AstDelta {
        loop {
            match self.open_groups.pop() {
                Some(open_index) => {
                    let close_index = self.next_index();
                    let close_token = match self.tokens[open_index] {
                        OpenParen(ref mut open_delta) => {
                            *open_delta = close_index-open_index;
                            return *open_delta;
                        },
                        OpenCompoundTerm(ref mut open_delta) => {
                            *open_delta = close_index-open_index;
                            CloseCompoundTerm(*open_delta)
                        },
                        _ => unreachable!()
                    };
                    self.push(close_token, range.start..range.start);
                },
                None => return self.handle_close_without_open(range),
            }
        }
    }

    fn close_compound_term(&mut self) -> Option<AstDelta> {
        let open_index = {
            let open_index = self.open_groups.last();
            if open_index.is_none() { return None; }
            *open_index.unwrap()
        };
        let close_index = self.next_index();
        if let OpenCompoundTerm(ref mut open_delta) = self.tokens[open_index] {
            self.open_groups.pop();
            *open_delta = close_index-open_index;
            Some(*open_delta)
        } else {
            None
        }
    }

    fn close_all(&mut self) {
        while let Some(open_index) = self.open_groups.pop() {
            let close_index = self.next_index();
            let close_loc = self.token_ranges[close_index-1].end;
            let delta = close_index-open_index;
            let close_token = match self.tokens[open_index] {
                OpenParen(ref mut open_delta) => {
                    *open_delta = delta;
                    self.compiler.report(compile_errors::OpenWithoutClose { source: self.source, open: self.token_ranges[open_index].clone(), close: String::from(")") });
                    CloseParen(delta)
                },
                OpenCompoundTerm(ref mut open_delta) => {
                    *open_delta = delta;
                    CloseCompoundTerm(delta)
                },
                _ => unreachable!(),
            };
            self.push(close_token, close_loc..close_loc);
        }
    }

    fn handle_close_without_open(&mut self, range: &ByteRange) -> Delta<AstIndex> {
        let open_index = AstIndex(0);
        let open_loc = ByteIndex(0);
        // We're about to shift the close_index over, so we need to set the delta correctly.
        let close_index = self.next_index()+1;
        let delta = close_index-open_index;
        self.tokens.insert(open_index, OpenParen(delta));
        self.token_ranges.insert(open_index, open_loc..open_loc);
        self.compiler.report(compile_errors::CloseWithoutOpen { source: self.source, close: range.clone(), open: String::from("(") });
        delta
    }
}