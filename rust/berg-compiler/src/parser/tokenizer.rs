use ast::{IdentifierIndex,LiteralIndex};
use ast::intern_pool::{StringPool,InternPool,Pool};
use ast::operators;
use ast::token::Token;
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use compiler::Compiler;
use compiler::source_data::{ByteIndex,ByteSlice,CharData,SourceIndex};
use compiler::compile_errors;
use compiler::source_data::{ByteRange};
use parser::scanner::CharType;
use parser::scanner::CharType::*;
use parser::tokenizer::TokenizerState::*;

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

impl<'p,'c:'p> Tokenizer<'p,'c> {
    fn new(compiler: &'p Compiler<'c>, source: SourceIndex) -> Self {
        let char_data = Default::default();
        let identifiers = operators::intern_all();
        let literals = Default::default();
        Tokenizer { compiler, source, char_data, identifiers, literals }
    }

    fn tokenize<OnToken: FnMut(Token,ByteRange)->()>(mut self, mut on_token: OnToken) -> (CharData, StringPool<IdentifierIndex>, StringPool<LiteralIndex>) {
        let buffer = self.compiler.with_source(self.source, |source_data| {
            let source_spec = source_data.source_spec();
            source_spec.open(self.compiler, self.source)
        });

        let mut start = ByteIndex(0);
        let mut index = start;
        let mut state = TokenizerState::Start;

        Self::emit_token(File.placeholder_open_token(), state, &buffer, start, index, &mut on_token);

        while let Some(char_type) = CharType::read(&buffer, &mut index) {
            use parser::scanner::CharType::*;
            state = match char_type {
                Digit       => self.integer(state, &buffer, start, &mut index, &mut on_token),
                Operator    => self.operator(state, &buffer, start, &mut index, &mut on_token),
                Open        => Self::emit_token(Parentheses.placeholder_open_token(), state, &buffer, start, index, &mut on_token),
                Close       => Self::emit_token(Parentheses.placeholder_close_token(), state, &buffer, start, index, &mut on_token),
                Newline     => { self.char_data.append_line(index); state.after_newline(start, index) },
                Space       => { Space.read_many(&buffer, &mut index); state.after_space() },
                Unsupported => self.report_unsupported(state, &buffer, start, &mut index),
                InvalidUtf8 => self.report_invalid_utf8(state, &buffer, start, &mut index),
            };
         
            start = index;
        }

        Self::emit_token(File.placeholder_close_token(), state, &buffer, start, index, &mut on_token);

        (self.char_data, self.identifiers.strings, self.literals)
    }

    fn emit_token<OnToken: FnMut(Token,ByteRange)->()>(
        token: Token,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        index: ByteIndex,
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

        on_token(token, start..index);

        // Insert close compound term if applicable
        if CharType::peek_if(buffer, index, Self::is_space) {
            on_token(CompoundTerm.placeholder_close_token(), index..index);
        }

        if token.has_right_operand() {
            TokenizerState::ImmediateLeftOperator
        } else {
            TokenizerState::ImmediateLeftOperand
        }
    }

    fn integer<OnToken: FnMut(Token,ByteRange)->()>(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        index: &mut ByteIndex,
        on_token: &mut OnToken
    ) -> TokenizerState {
        let string = Digit.read_many_to_str(buffer, start, index);
        let literal = self.literals.add(string);
        Self::emit_token(IntegerLiteral(literal), state, buffer, start, *index, on_token)
    }

    fn operator<OnToken: FnMut(Token,ByteRange)->()>(
        &mut self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        index: &mut ByteIndex,
        on_token: &mut OnToken
    ) -> TokenizerState {
        let string = Operator.read_many_to_str(buffer, start, index);
        let identifier = self.identifiers.add(string);
        let left_operand = !state.is_space_or_explicit_operator();
        let right_operand = !CharType::peek_if(buffer, *index, Self::is_space_or_explicit_operator);
        let token = if left_operand && !right_operand {
            PostfixOperator(identifier)
        } else if !left_operand && right_operand {
            PrefixOperator(identifier)
        } else {
            InfixOperator(identifier)
        };
        Self::emit_token(token, state, buffer, start, *index, on_token)
    }

    fn report_unsupported(
        &self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        index: &mut ByteIndex
    ) -> TokenizerState {
        Unsupported.read_many(buffer, index);
        self.compiler.report(compile_errors::UnsupportedCharacters { source: self.source, characters: start..*index });
        state.after_space()
    }
 
    fn report_invalid_utf8(
        &self,
        state: TokenizerState,
        buffer: &ByteSlice,
        start: ByteIndex,
        index: &mut ByteIndex
    ) -> TokenizerState {
        InvalidUtf8.read_many(buffer, index);
        self.compiler.report(compile_errors::InvalidUtf8 { source: self.source, bytes: start..*index });
        state.after_space()
    }

    fn is_space(char_type: Option<CharType>) -> bool {
        use parser::scanner::CharType::*;
        if let Some(char_type) = char_type {
            match char_type {
                Space|Newline|Unsupported|InvalidUtf8 => true,
                Open|Close|Operator|Digit => false,
            }
        } else {
            true
        }
    }

    fn is_space_or_explicit_operator(char_type: Option<CharType>) -> bool {
        use parser::scanner::CharType::*;
        if let Some(char_type) = char_type {
            match char_type {
                Close|Space|Newline|Operator|Unsupported|InvalidUtf8 => true,
                Open|Digit => false,
            }
        } else {
            true
        }
    }
}
