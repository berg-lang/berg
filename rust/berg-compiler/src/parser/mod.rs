mod scanner;
mod tokenizer;

use ast::AstIndex;
use ast::token::{Token,OpenToken};
use ast::token::Token::*;
use compiler::Compiler;
use compiler::compile_errors;
use compiler::source_data::{ByteIndex,ByteRange,ParseData,SourceIndex};
use indexed_vec::IndexedVec;

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
    open_groups: Vec<OpenGroup>,
    compiler: &'p Compiler<'c>,
    source: SourceIndex,
}

#[derive(Debug)]
struct OpenGroup {
    open_index: AstIndex,
    open_token: OpenToken,
    // root: Option<Token>,
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
    fn insert(&mut self, index: AstIndex, token: Token, range: ByteRange) {
        self.tokens.insert(index, token);
        self.token_ranges.insert(index, range);
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

    fn insert_token(&mut self, token: Token, range: ByteRange) {
        // First insert any extra close parens and push open parens on the list
        match token {
            OpenParen(_) => self.on_open_paren(range),
            OpenCompoundTerm(_) => self.on_open_compound_term(),
            CloseParen(_) => self.on_close_paren(range),
            CloseCompoundTerm(_) => self.on_close_compound_term(range),

            // Everything else, just push directly.
            _ => self.push(token, range),
        }
    }

    fn on_open_paren(&mut self, range: ByteRange) {
        let open_token = OpenToken::OpenParen(Default::default());
        let open_index = self.next_index();
        self.open_groups.push(OpenGroup { open_token, open_index});//, root: None });
        self.push(open_token.into(), range);
    }

    fn on_open_compound_term(&mut self) {
        let open_token = OpenToken::OpenCompoundTerm(Default::default());
        let open_index = self.next_index();
        self.open_groups.push(OpenGroup { open_token, open_index});//, root: None });
        // Don't push the compound term yet. Most of them don't need to actually be in the
        // AST since most things go the direction they should.
    }

    fn on_close_paren(&mut self, close_range: ByteRange) {
        while let Some(open_group) = self.open_groups.pop() {
            println!("Popped {:?}", open_group);
            match open_group {
                OpenGroup { open_token: OpenToken::OpenParen(_), open_index, .. } => {
                    return self.close_paren(open_index, close_range)
                },
                OpenGroup { open_token: OpenToken::OpenCompoundTerm(_), open_index, .. } => {
                    self.close_compound_term(open_index, close_range.start..close_range.start);
                },
            }
        }

        // We didn't find an open paren, so we need to insert an open paren and report it!
        let open_index = AstIndex(0);
        self.insert(open_index, OpenParen(Default::default()), ByteIndex(0)..ByteIndex(0));
        self.close_paren(open_index, close_range.clone());
        self.compiler.report(compile_errors::CloseWithoutOpen { source: self.source, close_range, open: String::from("(") });
    }

    fn on_close_compound_term(&mut self, close_range: ByteRange) {
        if let Some(&OpenGroup { open_token: OpenToken::OpenCompoundTerm(_), open_index, .. }) = self.open_groups.last() {
            self.open_groups.pop();
            self.close_compound_term(open_index, close_range);
        }
    }

    fn close_paren(&mut self, open_index: AstIndex, close_range: ByteRange) {
        let close_index = self.next_index();
        let delta = close_index-open_index;
        if let OpenParen(ref mut open_delta) = self.tokens[open_index] {
            *open_delta = close_index-open_index;
        } else {
            unreachable!()
        }
        self.push(CloseParen(delta), close_range);
    }

    fn close_compound_term(&mut self, open_index: AstIndex, close_range: ByteRange) {
        // We're about to insert the open token, so the close_index will move too.
        let close_index = self.next_index()+1;
        let delta = close_index-open_index;
        // NOTE we are certain that there will be a term at this location, because
        // OpenCompoundTerm could not have been generated without another term after it.
        let open_location = self.token_ranges[open_index].start;

        // Insert the open, then append the close
        self.insert(open_index, OpenCompoundTerm(delta), open_location..open_location);
        self.push(CloseCompoundTerm(delta), close_range);
    }

    fn close_all(&mut self) {
        while let Some(OpenGroup { open_token, open_index, .. }) = self.open_groups.pop() {
            let end = self.token_ranges.last().unwrap().end;
            match open_token {
                OpenToken::OpenParen(_) => {
                    self.compiler.report(compile_errors::OpenWithoutClose { source: self.source, open_range: self.token_ranges[open_index].clone(), close: String::from(")") });
                    self.close_paren(open_index, end..end);
                },
                OpenToken::OpenCompoundTerm(_) => self.close_compound_term(open_index, end..end),
            }
        }
    }
}