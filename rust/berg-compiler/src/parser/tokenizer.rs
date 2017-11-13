use ast::{IdentifierIndex,LiteralIndex};
use ast::intern_pool::{StringPool,InternPool,Pool};
use ast::identifiers;
use ast::token::Token;
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use compiler::Compiler;
use compiler::source_data::{ByteIndex,ByteSlice,CharData,SourceIndex};
use compiler::compile_errors;
use compiler::source_data::{ByteRange};
use parser::scanner::Scanner;
use parser::scanner::CharType::*;
use parser::tokenizer::TokenizerState::*;
use std::str;

///
/// Breaks a file into a series of Tokens, calling the given function for each
/// token.
/// 
pub(super) fn tokenize<OnToken: FnMut(Token,ByteRange)->()>(
    compiler: &Compiler,
    source: SourceIndex,
    on_token: OnToken
) -> (CharData, StringPool<IdentifierIndex>, StringPool<LiteralIndex>) {
    let tokenizer = Tokenizer::new(compiler, source);
    tokenizer.tokenize(on_token)
}

#[derive(Debug)]
struct Tokenizer<'p,'c:'p> {
    compiler: &'p Compiler<'c>,
    source: SourceIndex,
    char_data: CharData,
    identifiers: InternPool<IdentifierIndex>,
    literals: StringPool<LiteralIndex>,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum TokenizerState {
    ImmediateLeftOperand,
    ImmediateLeftOperator,
    Start,
    SpaceAfterOperand,
    SpaceAfterOperator,
    NewlineAfterOperand { newline_start: ByteIndex, newline_length: u8 }
}

impl<'p,'c:'p> Tokenizer<'p,'c> {
    fn new(compiler: &'p Compiler<'c>, source: SourceIndex) -> Self {
        let char_data = Default::default();
        let identifiers = identifiers::intern_all();
        let literals = Default::default();
        Tokenizer { compiler, source, char_data, identifiers, literals }
    }

    fn tokenize<OnToken: FnMut(Token,ByteRange)->()>(mut self, mut on_token: OnToken) -> (CharData, StringPool<IdentifierIndex>, StringPool<LiteralIndex>) {
        let buffer = self.compiler.with_source(self.source, |source_data| {
            let source_spec = source_data.source_spec();
            source_spec.open(self.compiler, self.source)
        });

        let mut scanner = Scanner::default();
        let mut start = scanner.index;
        let mut state = TokenizerState::Start;

        Self::emit_token(File.placeholder_open_token(), state, &buffer, start, &scanner, &mut on_token);

        while let Some(char_type) = scanner.next(&buffer) {
            use parser::scanner::CharType::*;
            state = match char_type {
                Digit       => self.integer(state, &buffer, start, &mut scanner, &mut on_token),
                Identifier  => self.identifier(state, &buffer, start, &mut scanner, &mut on_token),
                Operator    => self.operator(state, &buffer, start, &mut scanner, &mut on_token),
                Separator   => self.separator(state, &buffer, start, &mut scanner, &mut on_token),
                Open        => Self::emit_token(Parentheses.placeholder_open_token(), state, &buffer, start, &scanner, &mut on_token),
                Close       => Self::emit_token(Parentheses.placeholder_close_token(), state, &buffer, start, &scanner, &mut on_token),
                Newline     => { self.char_data.append_line(scanner.index); state.after_newline(start, scanner.index) },
                Space       => { scanner.next_while(Space, &buffer); state.after_space() },
                Unsupported => self.report_unsupported(state, &buffer, start, &mut scanner),
                InvalidUtf8 => self.report_invalid_utf8(state, &buffer, start, &mut scanner),
            };
         
            start = scanner.index;
        }

        Self::emit_token(File.placeholder_close_token(), state, &buffer, start, &scanner, &mut on_token);

        (self.char_data, self.identifiers.strings, self.literals)
    }

    fn emit_token<OnToken: FnMut(Token,ByteRange)->()>(
        token: Token,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &Scanner,
        on_token: &mut OnToken
    ) -> TokenizerState {
        if token.has_left_operand() {
            // Insert missing expression if applicable
            if !state.is_left_operand() {
                on_token(MissingExpression, start..start)
            }
        } else {
            // Insert newline sequence operator if applicable
            if let NewlineAfterOperand { newline_start, newline_length } = state {
                let newline_end = newline_start+(newline_length as usize);
                on_token(NewlineSequence, newline_start..newline_end);

            // Insert missing infix if applicable
            } else if state.is_left_operand() {
                on_token(MissingInfix, start..start)
            }

            // Insert open compound term if applicable
            if state.is_space() {
                on_token(CompoundTerm.placeholder_open_token(), start..start);
            }
        }

        on_token(token, start..scanner.index);

        if token.has_right_operand() {
            TokenizerState::ImmediateLeftOperator
        } else {
            // Insert close compound term if applicable
            if scanner.peek_if_space(buffer) {
                on_token(CompoundTerm.placeholder_close_token(), scanner.index..scanner.index);
            }
            TokenizerState::ImmediateLeftOperand
        }
    }

    fn integer<OnToken: FnMut(Token,ByteRange)->()>(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner,
        on_token: &mut OnToken
    ) -> TokenizerState {
        scanner.next_while(Digit, buffer);
        let token_type = if scanner.next_while_identifier(buffer) {
            self.compiler.report(compile_errors::IdentifierStartsWithNumber { source: self.source, identifier: start..scanner.index });
            SyntaxErrorTerm
        } else {
            IntegerLiteral
        };
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let literal = self.literals.add(string);
        Self::emit_token(token_type(literal), state, buffer, start, scanner, on_token)
    }

    fn identifier<OnToken: FnMut(Token,ByteRange)->()>(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner,
        on_token: &mut OnToken
    ) -> TokenizerState {
        scanner.next_while_identifier(buffer);
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let identifier = self.identifiers.add(string);
        Self::emit_token(PropertyReference(identifier), state, buffer, start, scanner, on_token)
    }

    fn operator<OnToken: FnMut(Token,ByteRange)->()>(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner,
        on_token: &mut OnToken
    ) -> TokenizerState {
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
        Self::emit_token(token, state, buffer, start, scanner, on_token)
    }

    fn separator<OnToken: FnMut(Token,ByteRange)->()>(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner,
        on_token: &mut OnToken
    ) -> TokenizerState {
        let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
        let identifier = self.identifiers.add(string);
        Self::emit_token(InfixOperator(identifier), state, buffer, start, scanner, on_token)
    }

    fn report_unsupported(
        &self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner
    ) -> TokenizerState {
        scanner.next_while(Unsupported, buffer);
        self.compiler.report(compile_errors::UnsupportedCharacters { source: self.source, characters: start..scanner.index });
        state.after_space()
    }
 
    fn report_invalid_utf8(
        &self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        scanner: &mut Scanner
    ) -> TokenizerState {
        scanner.next_while(InvalidUtf8, buffer);
        self.compiler.report(compile_errors::InvalidUtf8 { source: self.source, bytes: start..scanner.index });
        state.after_space()
    }
}

impl TokenizerState {
    fn after_space(self) -> Self {
        match self {
            Start|SpaceAfterOperator|SpaceAfterOperand|NewlineAfterOperand {..} => self,
            ImmediateLeftOperand => SpaceAfterOperand,
            ImmediateLeftOperator => SpaceAfterOperator,
        }
    }
    fn after_newline(self, start: ByteIndex, end: ByteIndex) -> Self {
        match self {
            SpaceAfterOperand|ImmediateLeftOperand => NewlineAfterOperand { newline_start: start, newline_length: usize::from(end-start) as u8 },
            ImmediateLeftOperator => SpaceAfterOperator,
            Start|SpaceAfterOperator|NewlineAfterOperand {..} => self,
        }
    }
    fn is_space(self) -> bool {
        match self {
            Start|SpaceAfterOperator|SpaceAfterOperand|NewlineAfterOperand {..} => true,
            ImmediateLeftOperand|ImmediateLeftOperator => false,
        }
    }
    fn is_left_operand(self) -> bool {
        match self {
            ImmediateLeftOperand|SpaceAfterOperand|NewlineAfterOperand {..} => true,
            ImmediateLeftOperator|Start|SpaceAfterOperator => false,
        }
    }
    fn is_space_or_explicit_operator(self) -> bool {
        match self {
            ImmediateLeftOperator|Start|SpaceAfterOperator|SpaceAfterOperand|NewlineAfterOperand {..} => true,
            ImmediateLeftOperand => false,
        }
    }
}
