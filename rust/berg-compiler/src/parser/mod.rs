pub(crate) mod char_data;
pub(crate) mod token;
mod ast_builder;
mod scanner;
mod token_pool;

use public::*;
use parser::char_data::CharData;
use parser::scanner::*;
use parser::ast_builder::AstBuilder;
use compiler::compile_errors::*;
use indexed_vec::IndexedVec;
use std::ops::Range;
use std::u32;
index_type! {
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) <= u32::MAX;
    pub struct AstIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub(crate) struct ParseData {
    pub(crate) char_data: CharData,
    pub(crate) identifier_strings: IndexedVec<String,IdentifierIndex>,
    pub(crate) literal_strings: IndexedVec<String,LiteralIndex>,
    pub(crate) tokens: IndexedVec<Token,AstIndex>,
    pub(crate) token_ranges: IndexedVec<Range<ByteIndex>,AstIndex>,
}

pub(crate) fn parse(
    compiler: &Compiler,
    errors: &mut SourceCompileErrors,
    source_spec: &SourceSpec
) -> ParseData {
    let buffer_guard = source_spec.open(compiler, errors);
    let buffer = buffer_guard.buffer();
    let mut ast = AstBuilder::default();

    // Loop through tokens, inserting term, then operator, then term, then operator ...
    let mut scanner = Scanner::default();
    read_term(&mut scanner, buffer, errors, &mut ast);
    while read_infix(&mut scanner, buffer, errors, &mut ast) {
        read_term(&mut scanner, buffer, errors, &mut ast);
    }

    // Result
    ParseData::from(ast)
}

fn read_term(
    scanner: &mut Scanner,
    buffer: &[u8],
    errors: &mut SourceCompileErrors,
    ast: &mut AstBuilder
) {
    use parser::scanner::TermResult::*;

    // Read through prefixes, inserting them until we find a term.
    let mut term = scanner.next_term(buffer, errors);
    while let Prefix(start) = term {
        ast.append_operator(buffer, start, scanner.index(), Token::Prefix);
        term = scanner.next_term(buffer, errors);
    }

    // Insert the actual term.
    match term {
        IntegerLiteral(start) => ast.append_literal(buffer, start, scanner.index(), Token::IntegerLiteral),
        MissingTerm => ast.append(Token::MissingTerm, scanner.index()..scanner.index()),
        Prefix(_) => unreachable!(),
    }
}

fn read_infix(
    scanner: &mut Scanner,
    buffer: &[u8],
    errors: &mut SourceCompileErrors,
    ast: &mut AstBuilder
) -> bool {
    use parser::scanner::OperatorResult::*;

    // Read through postfixes, appending them until we find an infix operator.
    let mut operator = scanner.next_operator(buffer, errors);
    while let Postfix(start) = operator {
        ast.append_operator(buffer, start, scanner.index(), Token::Postfix);
        operator = scanner.next_operator(buffer, errors);
    }

    // Append the operator, or break out of the loop if we hit EOF.
    match operator {
        Infix(start) => ast.append_operator(buffer, start, scanner.index(), Token::Infix),
        MissingInfix => ast.append(Token::MissingInfix, scanner.index()..scanner.index()),
        Eof => return false,
        OperatorResult::Postfix(_) => unreachable!(),
    }
    true
}

impl ParseData {
    fn from(ast: AstBuilder) -> Self {
        ParseData {
            char_data: Default::default(),
            tokens: ast.tokens,
            token_ranges: ast.token_ranges,
            identifier_strings: ast.identifier_pool.strings,
            literal_strings: ast.literal_strings,
        }
    }
}
