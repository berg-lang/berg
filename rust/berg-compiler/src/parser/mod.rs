pub(crate) mod char_data;
           mod token_pool;
pub(crate) mod token;
           mod tokenizer;

use public::*;
use parser::char_data::CharData;
use parser::tokenizer::Tokenizer;
use parser::token_pool::TokenPool;
use compiler::compile_errors::*;
use indexed_vec::IndexedVec;
use std::ops::Range;
use std::u32;
index_type! {
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub(crate) struct ParseData {
    pub(crate) char_data: CharData,
    pub(crate) tokens: Vec<Token>,
    pub(crate) identifier_strings: IndexedVec<String,IdentifierIndex>,
    pub(crate) literal_strings: IndexedVec<String,LiteralIndex>,
    pub(crate) token_ranges: Vec<Range<ByteIndex>>,
}

pub(crate) fn parse(
    compiler: &Compiler,
    errors: &mut SourceCompileErrors,
    source_spec: &SourceSpec
) -> ParseData {
    let buffer = source_spec.open(compiler, errors);

    // State
    let mut tokenizer = Tokenizer::default();

    // Output
    let char_data = CharData::default();
    let mut identifier_pool = TokenPool::<IdentifierIndex>::default();
    let mut literal_strings = IndexedVec::<String,LiteralIndex>::default();
    let mut tokens = Vec::<Token>::default();
    let mut token_ranges = Vec::<Range<ByteIndex>>::default();

    // Algorithm
    let mut index = ByteIndex(0);
    while let Some((start, token)) = tokenizer.next(buffer.buffer(), &mut index, errors, &mut identifier_pool, &mut literal_strings) {
        tokens.push(token);
        token_ranges.push(start..index);
    }

    // Result
    let identifier_strings = identifier_pool.strings;
    ParseData {
        char_data,
        tokens,
        token_ranges,
        identifier_strings,
        literal_strings,
    }
}
