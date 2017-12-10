use indexed_vec::IndexedVec;
use ast::AstIndex;
use compiler::source_data::SourceIndex;
use compiler::Compiler;
use compiler::source_data::ByteRange;
use parser::scanner::CharType;
use ast::{IdentifierIndex,LiteralIndex};
use ast::intern_pool::{StringPool,InternPool,Pool};
use ast::identifiers;
use ast::token::{ExpressionBoundary,Token};
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use compiler::source_data::{ByteIndex,ByteSlice,CharData,ParseData};
use compiler::compile_errors;
use parser::ast_builder::AstBuilder;
use parser::scanner::Scanner;
use parser::scanner::CharType::*;
use std::str;

// Chunks up the source into sequences: space, newlines, operators, etc.
// Passes these to the Tokenizer to handle expression and whitespace rules.
#[derive(Debug)]
pub(super) struct Sequencer<'p,'c:'p> {
    tokenizer: Tokenizer<'p,'c>,
    char_data: CharData,
    identifiers: InternPool<IdentifierIndex>,
    literals: StringPool<LiteralIndex>,
}

// This builds up a valid expression from the incoming sequences, doing two things:
// 1. Inserting apply, newline sequence, and missing expression as appropriate
//    when two operators or two terms are next to each other.
// 2. Opening and closing terms (series of tokens with no space between operators/operands).
#[derive(Debug)]
pub(super) struct Tokenizer<'p,'c:'p> {
    ast_builder: AstBuilder<'p,'c>,
    in_term: bool,
    operator: bool,
    newline_start: ByteIndex,
    newline_length: u8,
}

impl<'p,'c:'p> Sequencer<'p,'c> {
    pub(super) fn new(tokenizer: Tokenizer<'p,'c>) -> Self {
        Sequencer {
            tokenizer,
            char_data: Default::default(),
            identifiers: identifiers::intern_all(),
            literals: Default::default(),
        }
    }

    pub(super) fn tokenize(&mut self) {
        let buffer = self.compiler().with_source(self.source(), |source_data| {
            let source_spec = source_data.source_spec();
            source_spec.open(self.compiler(), self.source())
        });
        let mut scanner = Scanner::default();
        let mut start = scanner.index;

        self.tokenizer.on_source_start(start);

        loop {
            let char_type = scanner.next(&buffer);
            println!("CHAR TYPE #{:?}", char_type);

            match char_type {
                Digit       => self.integer(&buffer, start, &mut scanner),
                Identifier  => self.identifier(&buffer, start, &mut scanner),
                Operator    => self.operator(&buffer, start, &mut scanner),
                Separator   => self.separator(&buffer, start, &mut scanner),
                Colon       => self.colon(&buffer, start, &mut scanner),
                OpenParen   => self.tokenizer.on_open(Parentheses, start..scanner.index),
                CloseParen  => self.tokenizer.on_close(Parentheses, start..scanner.index),
                OpenCurly   => self.tokenizer.on_open(CurlyBraces, start..scanner.index),
                CloseCurly  => self.tokenizer.on_close(CurlyBraces, start..scanner.index),
                Newline     => self.newline(&buffer, start, &scanner),
                Space       => self.space(&buffer, start, &mut scanner),
                Unsupported => self.unsupported(&buffer, start, &mut scanner),
                InvalidUtf8 => self.invalid_utf8(&buffer, start, &mut scanner),
                Eof         => break,
            };
         
            start = scanner.index;
        }

        assert!(start == scanner.index);
        assert!(scanner.index == buffer.len()); 

        self.tokenizer.on_source_end(scanner.index)
    }

    fn compiler(&self) -> &Compiler<'c> {
        self.tokenizer.ast_builder.compiler
    }

    fn source(&self) -> SourceIndex {
        self.tokenizer.ast_builder.source
    }

    pub(super) fn complete(self) -> ParseData {
        let (tokens, token_ranges) = self.tokenizer.complete();
        ParseData {
            char_data: self.char_data,
            identifiers: self.identifiers.strings,
            literals: self.literals,
            tokens: tokens,
            token_ranges: token_ranges,
        }
    }

    fn integer(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        scanner.next_while(Digit, buffer);
        let token_type = if scanner.next_while_identifier(buffer) {
            self.compiler().report(compile_errors::IdentifierStartsWithNumber { source: self.source(), identifier: start..scanner.index });
            SyntaxErrorTerm
        } else {
            IntegerLiteral
        };
        let range = start..scanner.index;
        let string = unsafe { str::from_utf8_unchecked(&buffer[&range]) };
        let literal = self.literals.add(string);
        self.tokenizer.on_term_token(token_type(literal), range)
    }

    fn identifier(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        scanner.next_while_identifier(buffer);
        let range = start..scanner.index;
        let string = unsafe { str::from_utf8_unchecked(&buffer[&range]) };
        let identifier = self.identifiers.add(string);
        self.tokenizer.on_term_token(PropertyReference(identifier), range)
    }

    fn make_identifier(&mut self, slice: &[u8]) -> IdentifierIndex {
        let string = unsafe { str::from_utf8_unchecked(slice) };
        self.identifiers.add(string)
    }

    fn operator(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        scanner.next_while(CharType::Operator, buffer);

        let term_is_about_to_end = {
            let char_type = scanner.peek(buffer);
            char_type.is_space() || char_type.is_close() || char_type.is_separator() ||
            (char_type == Colon && !scanner.peek_at(buffer, 1).is_right_term_operand())
        };

        let range = start..scanner.index;
        if self.tokenizer.in_term && term_is_about_to_end {
            let identifier = self.make_identifier(&buffer[&range]);
            self.tokenizer.on_term_token(PostfixOperator(identifier), range);
        } else if !self.tokenizer.in_term && !term_is_about_to_end {
            let identifier = self.make_identifier(&buffer[&range]);
            self.tokenizer.on_term_token(PrefixOperator(identifier), range);
        } else {
            let token = if Self::is_assignment_operator(&buffer[&range]) {
                InfixAssignment(self.make_identifier(&buffer[start..scanner.index-1]))
            } else {
                InfixOperator(self.make_identifier(&buffer[&range]))
            };
            // If the infix operator is like a+b, it's inside the term. If it's
            // like a + b, it's outside (like a separator).
            if self.tokenizer.in_term {
                self.tokenizer.on_term_token(token, range);
            } else {
                self.tokenizer.on_separator(token, range);
            }
        }
    }

    fn separator(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        let string = self.make_identifier(&buffer[start..scanner.index]);
        self.tokenizer.on_separator(InfixOperator(string), start..scanner.index)
    }

    // Colon is, sadly, just a little ... special.
    // If we're succeeded by an operand, and we're not in a term ("1 + :a", "a :b"), we are a prefix.
    // If we're succeeded by an operand, and we're in a term, and we're preceded by an operator ("1+:a"), we are a prefix.
    // Else, we are separator. ("a:b", a:-b", "a: b", "a:")
    // See where the "operator" function calculates whether the term is about to end for the other
    // relevant silliness to ensure "a+:b" means "(a) + (:b)".
    fn colon(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        let range = start..scanner.index;
        let identifier = self.make_identifier(&buffer[&range]);
        if (!self.tokenizer.in_term || self.tokenizer.operator) && scanner.peek(buffer).is_right_term_operand() {
            self.tokenizer.on_term_token(PrefixOperator(identifier), range);
        } else {
            self.tokenizer.on_separator(InfixOperator(identifier), range);
        }
    }

    // Anything ending with exactly one = is assignment, EXCEPT
    // >=, != and <=.
    fn is_assignment_operator(slice: &[u8]) -> bool {
        if slice[slice.len()-1] != b'=' { return false; }
        if slice.len() < 2 { return true; }
        let prev_ch = slice[slice.len()-2];
        if prev_ch == b'=' { return false; }
        if slice.len() > 2 { return true; }
        match prev_ch { b'!'|b'>'|b'<' => false, _ => true }
    }

    fn newline(&mut self, _: &ByteSlice, start: ByteIndex, scanner: &Scanner) {
        self.char_data.append_line(scanner.index);
        self.tokenizer.on_newline(start, ((scanner.index - start).0).0 as u8)
    }

    fn space(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        scanner.next_while(Space, buffer);
        self.tokenizer.on_space(start)
    }

    fn unsupported(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        scanner.next_while(Unsupported, buffer);
        self.compiler().report(compile_errors::UnsupportedCharacters { source: self.source(), characters: start..scanner.index });
        self.tokenizer.on_space(start)
    }
 
    fn invalid_utf8(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
        scanner.next_while(InvalidUtf8, buffer);
        self.compiler().report(compile_errors::InvalidUtf8 { source: self.source(), bytes: start..scanner.index });
        self.tokenizer.on_space(start)
    }
}

impl<'p,'c:'p> Tokenizer<'p,'c> {
    pub(super) fn new(ast_builder: AstBuilder<'p,'c>) -> Self {
        Tokenizer {
            ast_builder,
            in_term: false,
            operator: true,
            newline_start: ByteIndex(0),
            newline_length: 0,
        }
    }

    // The start of source emits the "open source" token.
    fn on_source_start(&mut self, start: ByteIndex) {
        println!("on_source_start(in_term: {})", self.in_term);
        let token = ExpressionBoundary::Source.placeholder_open_token();
        self.emit_token(token, start..start)
    }

    // The end of the source closes any open terms, just like space. Also emits "close source."
    fn on_source_end(&mut self, end: ByteIndex) {
        println!("on_source_end(in_term: {})", self.in_term);
        self.close_term(end);
        let close_token = ExpressionBoundary::Source.placeholder_close_token();
        self.emit_token(close_token, end..end)
    }

    // +, foo, 123. If a term hasn't started, this will start it.
    fn on_term_token(&mut self, token: Token, range: ByteRange) {
        println!("on_term_token(in_term: {}): {:?}", self.in_term, token);
        assert!(range.start < range.end);
        self.open_term(range.start);
        self.emit_token(token, range);
    }

    // Space after a term closes it.
    fn on_space(&mut self, start: ByteIndex) {
        println!("on_space(in_term: {})", self.in_term);
        self.close_term(start)
    }

    // Newline is space, so it closes terms just like space. If the last line ended in a complete
    // expression, we may be about to create a newline sequence. Save the first newline until we know
    // whether the next real line is an operator (continuation) or a new expression.
    fn on_newline(&mut self, start: ByteIndex, length: u8) {
        println!("on_newline(in_term: {})", self.in_term);
        self.close_term(start);
        if !self.operator && self.newline_length == 0 {
            self.newline_start = start;
            self.newline_length = length;
        }
    }

    // ( or {. If the ( is after a space, opens a new term. But once we're in the ( a new term will
    // be started.
    fn on_open(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        println!("on_open(in_term: {}): {:?}", self.in_term, boundary);
        assert!(range.start < range.end);
        let token = boundary.placeholder_open_token();
        self.open_term(range.start);
        self.emit_token(token, range);
        self.in_term = false;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    fn on_close(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        println!("on_close(in_term: {}): {:?}", self.in_term, boundary);
        assert!(range.start < range.end);
        let token = boundary.placeholder_close_token();
        self.close_term(range.start);
        self.emit_token(token, range);
        self.in_term = true;
    }

    // ; or :. If the : is in a term, it closes it. Afterwards, we are looking to start a new term,
    // so it's still closed.
    fn on_separator(&mut self, token: Token, range: ByteRange) {
        println!("on_separator(in_term: {}): {:?}", self.in_term, token);
        assert!(range.start < range.end);
        self.close_term(range.start);
        self.emit_token(token, range);
    }

    fn complete(self) -> (IndexedVec<Token,AstIndex>,IndexedVec<ByteRange,AstIndex>) {
        self.ast_builder.complete()
    }

    fn open_term(&mut self, index: ByteIndex) {
        if !self.in_term {
            self.emit_token(ExpressionBoundary::CompoundTerm.placeholder_open_token(), index..index);
            self.in_term = true;
        }
    }

    fn close_term(&mut self, index: ByteIndex) {
        if self.in_term {
            self.in_term = false;
            self.emit_token(ExpressionBoundary::CompoundTerm.placeholder_close_token(), index..index)
        }
    }

    fn emit_token(&mut self, token: Token, range: ByteRange) {
        if self.operator {
            if token.has_left_operand() {
                self.emit_token(MissingExpression, range.start..range.start);
            }
        } else if !token.has_left_operand() {
            if self.newline_length > 0 {
                let newline_start = self.newline_start;
                let newline_end = newline_start + (self.newline_length as usize);
                self.emit_token(NewlineSequence, newline_start..newline_end);
            } else {
                self.emit_token(MissingInfix, range.start..range.start);
            }
        }
        self.ast_builder.on_token(token, range);
        self.operator = token.has_right_operand();
    }
}
