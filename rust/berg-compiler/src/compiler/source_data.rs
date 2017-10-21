use public::*;
use parser::IdentifierTokenIndex;
use parser::ParseData;
use parser::LiteralTokenIndex;
use parser::char_data::CharData;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::Range;
use indexed_vec::IndexedVec;
use std::u32;

#[derive(Debug)]
pub struct SourceData<'c> {
    source_spec: SourceSpec,
    pub(crate) parse_data: Option<ParseData>,
    pub(crate) checked_type: Option<Type>,
    phantom: PhantomData<&'c Compiler<'c>>,
}

// SourceDatas is a Vec<SourceData>, indexable by indexes of type `SourceIndex`.
index_type! {
    pub struct SourceIndex(pub u32) <= u32::MAX;
}

pub(crate) type Sources<'c> = IndexedVec<SourceData<'c>, SourceIndex>;

impl<'c> SourceData<'c> {
    pub(crate) fn new(source_spec: SourceSpec) -> Self {
        SourceData {
            source_spec,
            parse_data: None,
            checked_type: None,
            phantom: PhantomData,
        }
    }

    pub fn source_spec(&self) -> &SourceSpec {
        &self.source_spec
    }
    pub fn name(&self) -> &OsStr {
        self.source_spec.name()
    }
    pub fn parsed(&self) -> bool {
        self.parse_data.is_some()
    }
    pub fn checked(&self) -> bool {
        self.checked_type.is_some()
    }
    pub fn checked_type(&self) -> &Type {
        match self.checked_type {
            Some(ref checked_type) => checked_type,
            None => unreachable!(),
        }
    }
    pub fn char_data(&self) -> &CharData {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.char_data,
            None => unreachable!(),
        }
    }
    pub fn num_tokens(&self) -> usize {
        match self.parse_data {
            Some(ref parse_data) => parse_data.tokens.len(),
            None => unreachable!(),
        }
    }
    pub fn token(&self, token: usize) -> &Token {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.tokens[token],
            None => unreachable!(),
        }
    }
    pub fn token_range(&self, token: usize) -> Range<ByteIndex> {
        match self.parse_data {
            Some(ref parse_data) => {
                let range = &parse_data.token_ranges[token];
                Range { start: range.start, end: range.end }
            }
            None => unreachable!(),
        }
    }
    pub fn identifier_token_string(&self, index: IdentifierTokenIndex) -> &str {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.identifier_strings[index],
            None => unreachable!(),
        }       
    }
    pub fn literal_token_string(&self, index: LiteralTokenIndex) -> &str {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.literal_strings[index],
            None => unreachable!(),
        }       
    }
}
