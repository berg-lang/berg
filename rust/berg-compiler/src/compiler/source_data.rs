use public::*;
use parser::ParseData;
use parser::char_data::CharData;
use std::ffi::OsStr;
use std::marker::PhantomData;
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
index_type!(pub struct SourceIndex(pub u32));
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
    pub fn num_tokens(&self) -> TokenIndex {
        match self.parse_data {
            Some(ref parse_data) => parse_data.tokens.len(),
            None => unreachable!(),
        }
    }
    pub fn token(&self, token: TokenIndex) -> &Token {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.tokens[token],
            None => unreachable!(),
        }
    }
    pub fn token_start(&self, token: TokenIndex) -> ByteIndex {
        match self.parse_data {
            Some(ref parse_data) => parse_data.token_starts[token],
            None => unreachable!(),
        }
    }

    pub fn token_range(&self, token: TokenIndex) -> LineColumnRange {
        let start = self.token_start(token);
        let end = start + self.token(token).string.len() as ByteIndex;
        self.char_data().range(start..end)
    }
}
