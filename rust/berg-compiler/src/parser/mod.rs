pub mod char_data;
pub mod scanner;
pub mod token;

use public::*;
use parser::scanner::Scanner;
use parser::token::Tokens;
use parser::token::TokenStarts;
use std::mem;

pub fn parse<'p>(
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
    pub scanner: Scanner<'p, 'c>,
    pub need_next: NeedNext,
    pub tokens: Tokens,
    pub token_starts: TokenStarts,
}

#[derive(Debug)]
pub struct ParseData {
    pub char_data: CharData,
    pub tokens: Tokens,
    pub token_starts: TokenStarts,
}

#[derive(Debug, PartialEq)]
enum NeedNext {
    InitialTerm,
    Operator,
    Operand,
    Either((ByteIndex, String)),
}

impl<'p, 'c: 'p> Parser<'p, 'c> {
    pub fn new(scanner: Scanner<'p, 'c>, need_next: NeedNext) -> Self {
        let tokens = Default::default();
        let token_starts = Default::default();
        Parser {
            scanner,
            need_next,
            tokens,
            token_starts,
        }
    }

    pub fn parse(mut self) -> ParseData {
        while self.step() {}
        self.close();
        ParseData {
            char_data: self.scanner.char_data,
            tokens: self.tokens,
            token_starts: self.token_starts,
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
            self.term(IntegerLiteral, end)
        } else if let Some(end) = self.scanner.match_all(operator) {
            self.operator(end)
        } else {
            return false;
        }
        true
    }

    fn term(&mut self, term_type: TermType, end: ByteIndex) {
        let string = self.scanner.take_string(end);
        self.transition(|p, need_next| {
            match need_next {
                NeedNext::InitialTerm | NeedNext::Operand => p.push_token(term_type, string),
                NeedNext::Operator => unreachable!(),
                NeedNext::Either(prev_operator) => {
                    p.push_token(Infix, prev_operator);
                    p.push_token(term_type, string);
                }
            }
            NeedNext::Operator
        })
    }

    fn operator(&mut self, end: ByteIndex) {
        let string = self.scanner.take_string(end);
        self.transition(|p, need_next| match need_next {
            NeedNext::InitialTerm | NeedNext::Operand => {
                p.push_token(Prefix, string);
                NeedNext::Operand
            }
            NeedNext::Operator => NeedNext::Either(string),
            NeedNext::Either(_) => unreachable!(),
        })
    }

    fn close(&mut self) {
        self.transition(|p, need_next| {
            match need_next {
                // NOTE: we do not report MissingRightOperand here because it will be reported by the typechecker.
                NeedNext::InitialTerm | NeedNext::Operator | NeedNext::Operand => {}
                NeedNext::Either(prev_string) => p.push_token(Postfix, prev_string),
            }
            NeedNext::Operator
        })
    }

    fn transition<F: FnOnce(&mut Self, NeedNext) -> NeedNext>(&mut self, transition: F) {
        let need_next = mem::replace(&mut self.need_next, NeedNext::InitialTerm);
        self.need_next = transition(self, need_next);
    }

    fn push_token<T: Into<TokenType>>(
        &mut self,
        token_type: T,
        (start, string): (ByteIndex, String),
    ) {
        self.tokens.push(Token::new(token_type.into(), string));
        self.token_starts.push(start);
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
        self.scanner
            .compiler
            .report_at(UnsupportedCharacters, self.scanner.source, start, &string);
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
    (b'0'..=b'9').contains(byte)
}
fn operator(byte: u8) -> bool {
    match byte {
        b'+' | b'-' | b'*' | b'/' => true,
        _ => false,
    }
}
