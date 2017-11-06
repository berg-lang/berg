use ast::intern_pool::InternPool;
use compiler::source_data::ByteSlice;
use parser::scanner::CharType;
use ast::{AstDelta,IdentifierIndex,LiteralIndex};
use ast::intern_pool::{StringPool,Pool};
use ast::operators;
use ast::token::Token::*;
use compiler::compile_errors;
use compiler::source_data::{ByteRange};
use parser::scanner::CharType::*;
use parser::tokenizer::Need::*;
use parser::tokenizer::Need::Operator;
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

    let buffer = compiler.with_source(source, |source_data| {
        let source_spec = source_data.source_spec();
        source_spec.open(compiler, source)
    });

    let mut identifiers = operators::intern_all();
    let mut literals: StringPool<LiteralIndex> = Default::default();

    let mut start = ByteIndex(0);
    let mut index = start;

    let mut prev_char = None;
    let mut current_char = CharType::read(&buffer, &mut index);
    while let Some(char_type) = current_char {
        let (end, next_char) = match char_type {
            Digit       => integer(&buffer, start, &mut index, &mut literals, &mut on_token),
            Operator    => operator(&buffer, start, &mut index, &mut identifiers, prev_char, &mut on_token),
            Open        => token(&buffer, start, &mut index, OpenParen(AstDelta::default()), &mut on_token),
            Close       => token(&buffer, start, &mut index, CloseParen(AstDelta::default()), &mut on_token),
            Space       => Space.read_many(&buffer, &mut index),
            Unsupported => report_unsupported(&buffer, start, &mut index, compiler, source),
            InvalidUtf8 => report_invalid_utf8(&buffer, start, &mut index, compiler, source),
        };
        prev_char = current_char;
        current_char = next_char;
        start = end;
    }

    (identifiers.strings, literals)
}

fn integer<OnToken: FnMut(Token,ByteRange)->()>(
    buffer: &ByteSlice,
    start: ByteIndex,
    index: &mut ByteIndex,
    literals: &mut StringPool<LiteralIndex>,
    on_token: &mut OnToken
) -> (ByteIndex, Option<CharType>) {
    let (end, next_char) = Digit.read_many(buffer, index);
    let literal = unsafe { literals.add_utf8_unchecked(buffer, start, end) };
    on_token(IntegerLiteral(literal), start..end);
    (end, next_char)
}

fn operator<OnToken: FnMut(Token,ByteRange)->()>(
    buffer: &ByteSlice,
    start: ByteIndex,
    index: &mut ByteIndex,
    identifiers: &mut InternPool<IdentifierIndex>,
    prev_char: Option<CharType>,
    on_token: &mut OnToken
) -> (ByteIndex, Option<CharType>) {
    let (end, next_char) = CharType::Operator.read_many(buffer, index);
    println!("OPERATOR {}-{} -> {}", start, end, *index);
    let identifier = unsafe { identifiers.add_utf8_unchecked(buffer, start, end) };
    let token = choose_operator(identifier, prev_char, next_char);
    on_token(token, start..end);
    (end, next_char)
}

fn token<OnToken: FnMut(Token,ByteRange)->()>(
    buffer: &ByteSlice,
    start: ByteIndex,
    index: &mut ByteIndex,
    token: Token,
    on_token: &mut OnToken
) -> (ByteIndex, Option<CharType>) {
    let end = *index;
    on_token(token, start..end);
    (end, CharType::read(buffer, index))
}

fn report_unsupported(
    buffer: &ByteSlice,
    start: ByteIndex,
    index: &mut ByteIndex,
    compiler: &Compiler,
    source: SourceIndex
) -> (ByteIndex, Option<CharType>) {
    let (end, next_char) = Unsupported.read_many(buffer, index);
    compiler.report(compile_errors::UnsupportedCharacters { source, characters: start..end });
    (end, next_char)
}

fn report_invalid_utf8(
    buffer: &ByteSlice,
    start: ByteIndex,
    index: &mut ByteIndex,
    compiler: &Compiler,
    source: SourceIndex
) -> (ByteIndex, Option<CharType>) {
    let (end, next_char) = InvalidUtf8.read_many(buffer, index);
    compiler.report(compile_errors::InvalidUtf8 { source, bytes: start..end });
    (end, next_char)
}

fn choose_operator(identifier: IdentifierIndex, prev_char: Option<CharType>, next_char: Option<CharType>) -> Token {
    let prev_needs = Need::after(prev_char);
    let next_needs = Need::before(next_char);
    match (prev_needs,next_needs) {
        (Operator,Operand)|(Operator,Either) => PostfixOperator(identifier),
        (Operand,Operator)|(Either,Operator) => PrefixOperator(identifier),
        (Operator,Operator)|
        (Either,Either)|
        (Either,Operand)|
        (Operand,Either)|
        (Operand,Operand) => InfixOperator(identifier),
    }
}

#[derive(Debug)]
enum Need {
    Either,
    Operand,
    Operator,
}

impl Need {
    fn after(prev_char: Option<CharType>) -> Need {
        match prev_char {
            Some(Digit)|Some(Close) => Operator,
            None|Some(Open) => Operand,
            Some(Space)|Some(Unsupported)|Some(InvalidUtf8) => Either,
            Some(CharType::Operator) => unreachable!(), // Cannot have two operators in a row, they will parse as one
        }
    }
    fn before(next_char: Option<CharType>) -> Need {
        match next_char {
            Some(Digit)|Some(Open) => Operator,
            None|Some(Close) => Operand,
            Some(Space)|Some(Unsupported)|Some(InvalidUtf8) => Either,
            Some(CharType::Operator) => unreachable!(), // Cannot have two operators in a row, they will parse as one
        }
    }
}
