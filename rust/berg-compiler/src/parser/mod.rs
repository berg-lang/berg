mod scanner;
mod tokenizer;

use ast::intern_pool::*;
use ast::LiteralIndex;
use ast::operators::Operators;
use compiler::Compiler;
use compiler::compile_errors::SourceCompileErrors;
use compiler::source_data::ParseData;
use compiler::source_spec::SourceSpec;
use indexed_vec::IndexedVec;

pub(crate) fn parse(
    compiler: &Compiler,
    errors: &mut SourceCompileErrors,
    source_spec: &SourceSpec
) -> ParseData {
    let buffer = source_spec.open(compiler, errors);
    let mut identifiers = Operators::intern_all();
    let mut literals: StringPool<LiteralIndex> = Default::default();
    let mut tokens = IndexedVec::default();
    let mut token_ranges = IndexedVec::default();

    // Loop through tokens, inserting term, then operator, then term, then operator ...
    tokenizer::tokenize(buffer.buffer(), errors, &mut identifiers, &mut literals, |token, range| {
        tokens.push(token);
        token_ranges.push(range);
    });

    // Result
    let char_data = Default::default();
    let identifiers = identifiers.strings;
    ParseData { char_data, identifiers, literals, tokens, token_ranges }
}
