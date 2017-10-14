use public::*;
use std::marker::PhantomData;
use std::ffi::OsStr;

#[derive(Debug)]
pub struct SourceData<'c> {
    source_spec: SourceSpec,
    parse_data: Option<(CharData, Vec<Token>)>,
    phantom: PhantomData<&'c Compiler<'c>>,
}

impl<'c> SourceData<'c> {
    pub fn new(source_spec: SourceSpec) -> Self {
        SourceData {
            source_spec,
            parse_data: None,
            phantom: PhantomData,
        }
    }

    pub fn source(&self) -> &SourceSpec { &self.source_spec }
    pub fn name(&self) -> &OsStr { self.source_spec.name() }
    pub fn parsed(&self) -> bool { self.parse_data.is_some() }
    pub fn char_data(&self) -> &CharData {
        match self.parse_data {
            Some((ref char_data, _)) => char_data,
            None => unreachable!(),
        }
    }
    pub fn num_tokens(&self) -> TokenIndex {
        match self.parse_data {
            Some((_, ref tokens)) => tokens.len() as TokenIndex,
            None => unreachable!(),
        }
    }
    pub fn token(&self, token: TokenIndex) -> &Token {
        match self.parse_data {
            Some((_, ref tokens)) => &tokens[token as usize],
            None => unreachable!(),
        }
    }
    pub fn token_start(&self, token: TokenIndex) -> ByteIndex {
        self.char_data().token_starts[token as usize]
    }

    pub fn token_range(&self, token: TokenIndex) -> LineColumnRange {
        let start = self.token_start(token);
        let end = start + self.token(token).string.len() as ByteIndex;
        self.char_data().range(start..end)
    }

    pub(crate) fn parse_complete(&mut self, char_data: CharData, tokens: Vec<Token>) {
        self.parse_data = Some((char_data, tokens))
    }
}
