use compiler::source_data::ByteSlice;
use parser::scanner::CharType;
use ast::{AstDelta,IdentifierIndex,LiteralIndex};
use ast::intern_pool::{StringPool,Pool};
use ast::operators;
use ast::token::Token::*;
use compiler::compile_errors;
use compiler::source_data::{ByteRange};
use parser::scanner::CharType::*;
use public::*;

///
/// Breaks a file into a series of Tokens, calling the given function for each
/// token.
/// 
pub(crate) fn tokenize<OnToken: FnMut(Token,ByteRange)->()>(
    compiler: &Compiler,
    source: SourceIndex,
    mut on_token: OnToken
) -> (StringPool<IdentifierIndex>, StringPool<LiteralIndex>) {
    use parser::scanner::CharType::Operator;
    use parser::scanner::CharType::Space;

    let buffer = compiler.with_source(source, |source_data| {
        let source_spec = source_data.source_spec();
        source_spec.open(compiler, source)
    });

    let mut identifiers = operators::intern_all();
    let mut literals: StringPool<LiteralIndex> = Default::default();

    let mut start = ByteIndex(0);
    let mut index = start;

    let mut prev_char = None;
    while let Some(char_type) = CharType::read(&buffer, &mut index) {
        let token = match char_type {
            Digit       => {
                let literal = literals.add(Digit.read_many_to_str(&buffer, start, &mut index));
                Some(IntegerLiteral(literal))
            },
            Operator    => {
                let identifier = identifiers.add(CharType::Operator.read_many_to_str(&buffer, start, &mut index));
                Some(choose_operator(identifier, prev_char, &buffer, index))
            },
            Open        => Some(OpenParen(AstDelta::default())),
            Close       => Some(CloseParen(AstDelta::default())),
            Space       => { Space.read_many(&buffer, &mut index); None },
            Unsupported => {
                Unsupported.read_many(&buffer, &mut index);
                compiler.report(compile_errors::UnsupportedCharacters { source, characters: start..index });
                None
            },
            InvalidUtf8 => {
                InvalidUtf8.read_many(&buffer, &mut index);
                compiler.report(compile_errors::InvalidUtf8 { source, bytes: start..index });
                None
            },
        };
        
        if let Some(token) = token {
            // Insert open compound term if applicable
            if !token.has_left_operand() && is_space(prev_char) {
                on_token(OpenCompoundTerm(Default::default()), start..start);
            }
            on_token(token, start..index);
            if !token.has_right_operand() && CharType::peek_if(&buffer, index, is_space) {
                on_token(CloseCompoundTerm(Default::default()), index..index);
            }
        }
        prev_char = Some(char_type);
        start = index;
    }

    (identifiers.strings, literals)
}

fn choose_operator(identifier: IdentifierIndex, prev_char: Option<CharType>, buffer: &ByteSlice, index: ByteIndex) -> Token {
    let prev_is_operand = prev_is_operand(prev_char);
    let next_is_operand = CharType::peek_if(buffer, index, next_is_operand);
    if prev_is_operand && !next_is_operand {
        PostfixOperator(identifier)
    } else if !prev_is_operand && next_is_operand {
        PrefixOperator(identifier)
    } else {
        InfixOperator(identifier)
    }
}

fn is_space(char_type: Option<CharType>) -> bool {
    match char_type {
        None|Some(Space)|Some(Unsupported)|Some(InvalidUtf8) => true,
        Some(Open)|Some(Close)|Some(Digit)|Some(Operator) => false,
    }
}

fn prev_is_operand(prev_char: Option<CharType>) -> bool {
    match prev_char {
        Some(Close)|Some(Digit) => true,
        None|Some(Open)|Some(Space)|Some(Operator)|Some(Unsupported)|Some(InvalidUtf8) => false,
    }
}

fn next_is_operand(next_char: Option<CharType>) -> bool {
    match next_char {
        Some(Open)|Some(Digit) => true,
        None|Some(Close)|Some(Space)|Some(Operator)|Some(Unsupported)|Some(InvalidUtf8) => false,
    }
}
