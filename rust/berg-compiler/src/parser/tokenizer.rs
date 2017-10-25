use public::*;

use parser::LiteralIndex;
use indexed_vec::IndexedVec;
use parser::token_pool::TokenPool;
use compiler::compile_errors::SourceCompileErrors;
use parser::char_type::*;
use parser::IdentifierIndex;
use std::ops::Range;
use parser::token::*;
use parser::char_data::ByteIndex;
use parser::tokenizer::Tokenizer::*;
use std::str;

/// Parses a source file (bytes) into tokens (integers, operators, etc.).
#[derive(Debug)]
pub(crate) enum Tokenizer {
    Start,
    InsertMissing  { start: ByteIndex, index: ByteIndex, token_type: NextTokenType },
    NextToken          { start: ByteIndex, index: ByteIndex, token_type: NextTokenType },
    InfixOrPostfix { start: ByteIndex, index: ByteIndex },
}

#[derive(Debug,Copy,Clone)]
pub(crate) enum NextTokenType {
    Integer,
    Prefix,
    Eof,
}

impl NextTokenType {
    unsafe fn to_token(&self, buffer: &[u8], identifier_pool: &mut TokenPool<IdentifierIndex>, literal_strings: &mut IndexedVec<String,LiteralIndex>) -> Token {
        use parser::tokenizer::NextTokenType::*;
        match *self {
            Integer => Token::IntegerLiteral(Self::literal(buffer, literal_strings)),  
            Prefix => Token::Prefix(identifier_pool.intern_unchecked(buffer)),
            Eof => Token::MissingTerm,
        }
    }

    unsafe fn literal(buffer: &[u8], literal_strings: &mut IndexedVec<String,LiteralIndex>) -> LiteralIndex {
        let literal = literal_strings.len();
        literal_strings.push(str::from_utf8_unchecked(buffer).to_string());
        literal
    }

    fn has_right_operand(&self) -> bool {
        use parser::tokenizer::NextTokenType::*;
        match *self {
            Integer => false,
            Prefix => true,
            Eof => unreachable!(),
        }
    }

    fn has_left_operand(&self) -> bool {
        use parser::tokenizer::NextTokenType::*;
        match *self {
            Integer|Prefix => false,
            Eof => true,
        }
    }
}

impl Default for Tokenizer {
    fn default() -> Self { Tokenizer::Start }
}

impl Tokenizer {
    pub fn next(
        self,
        buffer: &[u8],
        errors: &mut SourceCompileErrors,
        identifier_pool: &mut TokenPool<IdentifierIndex>,
        literal_strings: &mut IndexedVec<String,LiteralIndex>
    ) -> Option<(Token, Range<ByteIndex>, Tokenizer)> {
        use parser::tokenizer::NextTokenType::*;
        match self {
            Start => {
                let index = ByteIndex(0);
                let next = Self::next_state(buffer, index, errors, Some(true));
                if let InsertMissing { start, token_type: Eof, .. } = next {
                    Some((Token::Nothing, start..start, next))
                } else {
                    next.next(buffer, errors, identifier_pool, literal_strings)
                }
            },
            InsertMissing { start, index, token_type } => {
                let range = start..start;
                let token = if token_type.has_left_operand() { Token::MissingTerm } else { Token::MissingInfix };
                let next = NextToken { start, index, token_type };
                Some((token, range, next))
            },
            NextToken { token_type: Eof, .. } => None,
            NextToken { start, index, token_type } => {
                let range = start..index;
                let bytes = &buffer[usize::from(start)..usize::from(index)];
                let token = unsafe { token_type.to_token(bytes, identifier_pool, literal_strings) };
                let next = Self::next_state(buffer, index, errors, Some(token_type.has_right_operand()));
                Some((token, range, next))
            },
            InfixOrPostfix { start, index } => {
                let range = start..index;
                let bytes = &buffer[usize::from(start)..usize::from(index)];
                let operator = unsafe { identifier_pool.intern_unchecked(bytes) };
                let next = Self::next_state(buffer, index, errors, None);

                let token = if next.need_term_prev() {
                    Token::Infix(operator)
                } else {
                    Token::Postfix(operator)
                };

                Some((token, range, next))
            },
        }
    }

    fn next_state(
        buffer: &[u8],
        mut start: ByteIndex,
        errors: &mut SourceCompileErrors,
        need_term_next: Option<bool>,
    ) -> Tokenizer {
        use parser::char_type::CharType::*;
        while let Some((char_type, index)) = CharType::next(buffer, start) {
            match char_type {
                Digit => {
                    let index = CharType::next_while(buffer, index, CharType::Digit);
                    return match need_term_next {
                        Some(true)|None => NextToken { start, index, token_type: NextTokenType::Integer },
                        Some(false) => InsertMissing { start, index, token_type: NextTokenType::Integer },
                    }
                },
                Operator => {
                    let index = CharType::next_while(buffer, index, CharType::Operator);
                    return match need_term_next {
                        Some(true)|None => NextToken { start, index, token_type: NextTokenType::Prefix },
                        Some(false) => InfixOrPostfix { start, index }
                    };
                },
                // Unsupported and invalid UTF-8 are treated like whitespace--we don't generate tokens
                // for them.
                CharType::Unsupported => { start = Self::report_unsupported(buffer, start, index, errors); },
                CharType::InvalidUtf8 => { start = Self::report_invalid_utf8(buffer, start, index, errors); },
            }
        }
        NextToken { start, index: start, token_type: NextTokenType::Eof }
    }

    fn report_unsupported(
        buffer: &[u8],
        start: ByteIndex,
        index: ByteIndex,
        errors: &mut SourceCompileErrors
    ) -> ByteIndex {
        let index = CharType::next_while(buffer, index, CharType::Unsupported);
        let range = start..index;
        let bytes = &buffer[usize::from(start)..usize::from(index)];
        let string = unsafe { str::from_utf8_unchecked(bytes) };
        errors.report_at(CompileErrorType::UnsupportedCharacters, range, string);
        index
    }

    fn report_invalid_utf8(
        buffer: &[u8],
        start: ByteIndex,
        index: ByteIndex,
        errors: &mut SourceCompileErrors
    ) -> ByteIndex {
        let index = CharType::next_while(buffer, index, CharType::InvalidUtf8);
        let range = start..index;
        let bytes = &buffer[usize::from(start)..usize::from(index)];
        errors.report_invalid_utf8(range, bytes);
        index
    }

    fn need_term_prev(&self) -> bool {
        match *self {
            NextToken { token_type, .. } => !token_type.has_left_operand(),
            InfixOrPostfix{..} => true,
            Start|InsertMissing{..} => unreachable!(),
        }
    }
}
