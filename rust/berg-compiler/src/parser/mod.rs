pub(crate) mod char_data;
           mod scanner;
           mod token_pool;
pub(crate) mod token;

use public::*;
use parser::char_data::CharData;
use parser::scanner::Scanner;
use parser::token_pool::*;
use std::mem;
use std::ops::Range;

pub(crate) fn parse<'p>(
    compiler: &Compiler,
    source: SourceIndex,
    source_spec: &'p SourceSpec,
) -> ParseData {
    source_spec.with_buffer(compiler, source, |raw_buffer| {
        let scanner = Scanner::new(compiler, source, raw_buffer);
        let parser = Parser::new(scanner, NeedNext::InitialTerm);
        parser.parse()
    })
}

/// Shared parsing state
#[derive(Debug)]
struct Parser<'p, 'c: 'p> {
    scanner: Scanner<'p, 'c>,
    need_next: NeedNext,
    tokens: Vec<Token>,
    token_ranges: Vec<Range<ByteIndex>>,
}

#[derive(Debug)]
pub(crate) struct ParseData {
    pub(crate) char_data: CharData,
    pub(crate) token_pool: TokenPool,
    pub(crate) tokens: Vec<Token>,
    pub(crate) token_ranges: Vec<Range<ByteIndex>>,
}

#[derive(Debug, PartialEq)]
enum NeedNext {
    InitialTerm,
    Operator,
    Operand,
    Either(Range<ByteIndex>, TokenIndex),
}

impl<'p, 'c: 'p> Parser<'p, 'c> {
    fn new(scanner: Scanner<'p, 'c>, need_next: NeedNext) -> Self {
        let tokens = Default::default();
        let token_ranges = Default::default();
        Parser {
            scanner,
            need_next,
            tokens,
            token_ranges,
        }
    }

    fn parse(mut self) -> ParseData {
        while self.step() {}
        self.close();
        ParseData {
            char_data: self.scanner.char_data,
            tokens: self.tokens,
            token_pool: self.scanner.token_pool,
            token_ranges: self.token_ranges,
        }
    }

    fn step(&mut self) -> bool {
        if self.scanner.eof() {
            return false;
        }

        if self.scan_token() || self.report_unsupported_characters() {
            true
        } else {
            self.report_invalid_utf8();
            true
        }
    }

    fn scan_token(&mut self) -> bool {
        if let Some(end) = self.scanner.match_all(digit) {
            let (range, string) = self.scanner.take_string(end);
            self.term(range, TermType::IntegerLiteral(string))
        } else if let Some(end) = self.scanner.match_all(operator) {
            self.operator(end)
        } else {
            return false;
        }
        true
    }

    fn term(&mut self, range: Range<ByteIndex>, term_type: TermType) {
        self.transition(|p, need_next| {
            match need_next {
                NeedNext::InitialTerm | NeedNext::Operand => p.push_token(range, Token::Term(term_type)),
                NeedNext::Operator => unreachable!(),
                NeedNext::Either(prev_range, prev_token) => {
                    p.push_token(prev_range, Token::Infix(prev_token));
                    p.push_token(range, Token::Term(term_type));
                }
            }
            NeedNext::Operator
        })
    }

    fn operator(&mut self, end: ByteIndex) {
        let (range, token) = self.scanner.take_token(end);
        self.transition(|p, need_next| match need_next {
            NeedNext::InitialTerm | NeedNext::Operand => {
                p.push_token(range, Token::Prefix(token));
                NeedNext::Operand
            }
            NeedNext::Operator => NeedNext::Either(range, token),
            NeedNext::Either(..) => unreachable!(),
        })
    }

    fn close(&mut self) {
        self.transition(|p, need_next| {
            match need_next {
                // NOTE: we do not report MissingRightOperand here because it will be reported by the typechecker.
                NeedNext::InitialTerm | NeedNext::Operator | NeedNext::Operand => {}
                NeedNext::Either(prev_range, prev_token) => p.push_token(prev_range, Token::Postfix(prev_token)),
            }
            NeedNext::Operator
        })
    }

    fn transition<F: FnOnce(&mut Self, NeedNext) -> NeedNext>(&mut self, transition: F) {
        let need_next = mem::replace(&mut self.need_next, NeedNext::InitialTerm);
        self.need_next = transition(self, need_next);
    }

    fn push_token(
        &mut self,
        range: Range<ByteIndex>,
        token: Token,
    ) {
        self.tokens.push(token);
        self.token_ranges.push(range);
    }

    fn report_unsupported_characters(&mut self) -> bool {
        let start = self.scanner.index;
        let mut string = String::new();
        if !self.scanner.take_valid_char(&mut string) {
            return false;
        }

        // If there are valid UTF-8 chars, they are just unsupported. Report
        while !self.scanner.eof() && !self.scan_token() && self.scanner.take_valid_char(&mut string)
        {
        }
        let end = start + string.len();
        self.scanner
            .compiler
            .report_at(UnsupportedCharacters, self.scanner.source, start..end, &string);
        true
    }

    fn report_invalid_utf8(&mut self) {
        // Invalid UTF-8. Read invalid characters until you find something valid.
        let start = self.scanner.index;
        let mut bytes: Vec<u8> = vec![];
        while !self.scanner.eof() && !self.scanner.is_valid_char() {
            self.scanner.take_byte(&mut bytes);
        }
        self.scanner
            .compiler
            .report_invalid_bytes(InvalidUtf8, self.scanner.source, start, &bytes)
    }
}

fn digit(byte: u8) -> bool {
    byte >= b'0' && byte <= b'9'
}
fn operator(byte: u8) -> bool {
    match byte {
        b'+' | b'-' | b'*' | b'/' => true,
        _ => false,
    }
}
