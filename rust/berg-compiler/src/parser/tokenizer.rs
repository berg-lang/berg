use compiler::source_data::ParseData;
use ast::{IdentifierIndex,LiteralIndex};
use ast::intern_pool::{StringPool,InternPool,Pool};
use ast::identifiers;
use ast::token::Token;
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use compiler::source_data::{ByteIndex,ByteSlice,CharData};
use compiler::compile_errors;
use parser::ast_builder::AstBuilder;
use parser::scanner::Scanner;
use parser::scanner::CharType::*;
use parser::tokenizer::TokenizerState::*;
use std::str;

#[derive(Debug)]
pub(super) struct Tokenizer<'p,'c:'p> {
    ast_builder: AstBuilder<'p,'c>,
    char_data: CharData,
    identifiers: InternPool<IdentifierIndex>,
    literals: StringPool<LiteralIndex>,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum TokenizerState {
    ImmediateLeftOperand,
    ImmediateLeftIdentifier,
    ImmediateLeftOperator,
    Start,
    SpaceAfterOperand,
    SpaceAfterOperator,
    NewlineAfterOperand { newline_start: ByteIndex, newline_length: u8 }
}

impl<'p,'c:'p> Tokenizer<'p,'c> {
    pub(super) fn new(ast_builder: AstBuilder<'p,'c>) -> Self {
        Tokenizer {
            ast_builder,
            char_data: Default::default(),
            identifiers: identifiers::intern_all(),
            literals: Default::default(),
        }
    }

    pub(super) fn tokenize(&mut self) {
        let buffer = self.ast_builder.compiler.with_source(self.ast_builder.source, |source_data| {
            let source_spec = source_data.source_spec();
            source_spec.open(self.ast_builder.compiler, self.ast_builder.source)
        });

        let mut scanner = Scanner::default();
        let mut start = scanner.index;
        let mut state = TokenizerState::Start;

        self.emit_token(File.placeholder_open_token(), state, &buffer, start, &scanner);

        while let Some(char_type) = scanner.next(&buffer) {
            use parser::scanner::CharType::*;
            state = match char_type {
                Digit       => self.integer(state, &buffer, start, &mut scanner),
                Identifier  => self.identifier(state, &buffer, start, &mut scanner),
                Colon       => self.colon(state, &buffer, start, &mut scanner),
                Operator    => self.operator(state, &buffer, start, &mut scanner),
                Separator   => self.separator(state, &buffer, start, &mut scanner),
                Open        => self.emit_token(Parentheses.placeholder_open_token(), state, &buffer, start, &scanner),
                Close       => self.emit_token(Parentheses.placeholder_close_token(), state, &buffer, start, &scanner),
                Newline     => { self.char_data.append_line(scanner.index); state.after_newline(start, scanner.index) },
                Space       => { scanner.next_while(Space, &buffer); state.after_space() },
                Unsupported => self.report_unsupported(state, &buffer, start, &mut scanner),
                InvalidUtf8 => self.report_invalid_utf8(state, &buffer, start, &mut scanner),
            };
         
            start = scanner.index;
        }

        self.emit_token(File.placeholder_close_token(), state, &buffer, start, &scanner);
    }

    pub(super) fn complete(self) -> ParseData {
        let (tokens, token_ranges) = self.ast_builder.complete();
        ParseData {
            char_data: self.char_data,
            identifiers: self.identifiers.strings,
            literals: self.literals,
            tokens: tokens,
            token_ranges: token_ranges,
        }
    }

    fn emit_token(
        &mut self,
        token: Token,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &Scanner
    ) -> TokenizerState {
        if token.has_left_operand() {
            // Insert missing expression if applicable
            if !state.is_left_operand() {
                self.ast_builder.on_token(MissingExpression, start..start)
            }
        } else {
            // Insert newline sequence operator if applicable
            if let NewlineAfterOperand { newline_start, newline_length } = state {
                let newline_end = newline_start+(newline_length as usize);
                self.ast_builder.on_token(NewlineSequence, newline_start..newline_end);

            // Insert missing infix if applicable
            } else if state.is_left_operand() {
                self.ast_builder.on_token(MissingInfix, start..start)
            }

            // Insert open compound term if applicable
            if state.is_space() {
                self.ast_builder.on_token(CompoundTerm.placeholder_open_token(), start..start);
            }
        }

        self.ast_builder.on_token(token, start..scanner.index);

        if token.has_right_operand() {
            TokenizerState::ImmediateLeftOperator
        } else {
            // Insert close compound term if applicable
            if scanner.peek_if_space(buffer) {
                self.ast_builder.on_token(CompoundTerm.placeholder_close_token(), scanner.index..scanner.index);
            }
            TokenizerState::ImmediateLeftOperand
        }
    }

    fn integer(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner
    ) -> TokenizerState {
        scanner.next_while(Digit, buffer);
        let token_type = if scanner.next_while_identifier(buffer) {
            self.ast_builder.compiler.report(compile_errors::IdentifierStartsWithNumber { source: self.ast_builder.source, identifier: start..scanner.index });
            SyntaxErrorTerm
        } else {
            IntegerLiteral
        };
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let literal = self.literals.add(string);
        self.emit_token(token_type(literal), state, buffer, start, scanner)
    }

    fn identifier(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        scanner.next_while_identifier(buffer);
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let identifier = self.identifiers.add(string);
        self.emit_token(PropertyReference(identifier), state, buffer, start, scanner);
        TokenizerState::ImmediateLeftIdentifier
    }

    fn colon(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        // :abc, :_abc, etc. are declarations
        if state != ImmediateLeftIdentifier && scanner.next_while_identifier(buffer) {
            self.declaration(state, buffer, start, scanner)
        // a: b, a:b, a:-b and a : b are the : operator
        } else {
            self.separator(state, buffer, start, scanner)
        }
    }

    fn declaration(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        scanner.next_while_identifier(buffer);
        let string = unsafe { str::from_utf8_unchecked(&buffer[(start+1)..scanner.index]) };
        let identifier = self.identifiers.add(string);
        self.emit_token(PropertyDeclaration(identifier), state, buffer, start, scanner)
    }

    fn operator(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        scanner.next_while(Operator, buffer);
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let identifier = self.identifiers.add(string);
        let left_operand = !state.is_space_or_explicit_operator();
        let right_operand = !scanner.peek_if_space_or_operator(buffer);
        let token = if left_operand && !right_operand {
            PostfixOperator(identifier)
        } else if !left_operand && right_operand {
            PrefixOperator(identifier)
        } else {
            InfixOperator(identifier)
        };
        self.emit_token(token, state, buffer, start, scanner)
    }

    fn separator(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let identifier = self.identifiers.add(string);
        self.emit_token(InfixOperator(identifier), state, buffer, start, scanner)
    }

    fn report_unsupported(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        scanner.next_while(Unsupported, buffer);
        self.ast_builder.compiler.report(compile_errors::UnsupportedCharacters { source: self.ast_builder.source, characters: start..scanner.index });
        state.after_space()
    }
 
    fn report_invalid_utf8(&mut self, state: TokenizerState, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) -> TokenizerState {
        scanner.next_while(InvalidUtf8, buffer);
        self.ast_builder.compiler.report(compile_errors::InvalidUtf8 { source: self.ast_builder.source, bytes: start..scanner.index });
        state.after_space()
    }
}

impl TokenizerState {
    fn after_space(self) -> Self {
        match self {
            Start|SpaceAfterOperator|SpaceAfterOperand|NewlineAfterOperand {..} => self,
            ImmediateLeftOperand|ImmediateLeftIdentifier => SpaceAfterOperand,
            ImmediateLeftOperator => SpaceAfterOperator,
        }
    }
    fn after_newline(self, start: ByteIndex, end: ByteIndex) -> Self {
        match self {
            SpaceAfterOperand|ImmediateLeftOperand|ImmediateLeftIdentifier => NewlineAfterOperand { newline_start: start, newline_length: usize::from(end-start) as u8 },
            ImmediateLeftOperator => SpaceAfterOperator,
            Start|SpaceAfterOperator|NewlineAfterOperand {..} => self,
        }
    }
    fn is_space(self) -> bool {
        match self {
            Start|SpaceAfterOperator|SpaceAfterOperand|NewlineAfterOperand {..} => true,
            ImmediateLeftOperand|ImmediateLeftIdentifier|ImmediateLeftOperator => false,
        }
    }
    fn is_left_operand(self) -> bool {
        match self {
            ImmediateLeftOperand|ImmediateLeftIdentifier|SpaceAfterOperand|NewlineAfterOperand {..} => true,
            ImmediateLeftOperator|Start|SpaceAfterOperator => false,
        }
    }
    fn is_space_or_explicit_operator(self) -> bool {
        match self {
            ImmediateLeftOperator|Start|SpaceAfterOperator|SpaceAfterOperand|NewlineAfterOperand {..} => true,
            ImmediateLeftOperand|ImmediateLeftIdentifier => false,
        }
    }
}
