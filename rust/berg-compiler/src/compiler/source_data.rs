use public::*;
use parser::ParseData;
use std::marker::PhantomData;
use std::ffi::OsStr;

#[derive(Debug)]
pub struct SourceData<'c> {
    source_spec: SourceSpec,
    pub(crate) parse_data: Option<ParseData>,
    pub(crate) checked_type: Option<Type>,
    phantom: PhantomData<&'c Compiler<'c>>,
}

impl<'c> SourceData<'c> {
    pub fn new(source_spec: SourceSpec) -> Self {
        SourceData {
            source_spec,
            parse_data: None,
            checked_type: None,
            phantom: PhantomData,
        }
    }

    pub fn source_spec(&self) -> &SourceSpec { &self.source_spec }
    pub fn name(&self) -> &OsStr { self.source_spec.name() }
    pub fn parsed(&self) -> bool { self.parse_data.is_some() }
    pub fn checked(&self) -> bool { self.checked_type.is_some() }
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
            Some(ref parse_data) => parse_data.tokens.len() as TokenIndex,
            None => unreachable!(),
        }
    }
    pub fn token(&self, token: TokenIndex) -> &Token {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.tokens[token as usize],
            None => unreachable!(),
        }
    }
    pub fn token_start(&self, token: TokenIndex) -> ByteIndex {
        match self.parse_data {
            Some(ref parse_data) => parse_data.token_starts[token as usize],
            None => unreachable!()
        }
    }

    pub fn token_range(&self, token: TokenIndex) -> LineColumnRange {
        let start = self.token_start(token);
        let end = start + self.token(token).string.len() as ByteIndex;
        self.char_data().range(start..end)
    }
}
