use ast::operators::*;
use indexed_vec::IndexedSlice;
use std::fmt::*;
use ast::intern_pool::StringPool;
use ast::{AstIndex,IdentifierIndex,LiteralIndex};
use indexed_vec::IndexedVec;
use public::*;
use std::ffi::OsStr;
use std::ops::Range;
use std::u32;

index_type! {
    pub struct SourceIndex(pub u32) <= u32::MAX;
    pub struct ByteIndex(pub u32) <= u32::MAX;
}

pub type ByteSlice = IndexedSlice<u8,ByteIndex>;
pub type ByteRange = Range<ByteIndex>;

#[derive(Debug)]
pub struct SourceData<'s> {
    source_spec: SourceSpec<'s>,
    pub(crate) parse_data: Option<ParseData>,
    pub(crate) checked_type: Option<Type>,
}

#[derive(Debug)]
pub struct ParseData {
    pub char_data: CharData,
    pub identifiers: StringPool<IdentifierIndex>,
    pub literals: StringPool<LiteralIndex>,
    pub tokens: IndexedVec<Token,AstIndex>,
    pub token_ranges: IndexedVec<ByteRange,AstIndex>,
}

#[derive(Debug)]
pub struct CharData {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    pub byte_length: ByteIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    // Start indices of each line
    pub line_starts: Vec<ByteIndex>,
}

impl<'s> SourceData<'s> {
    pub(crate) fn new(source_spec: SourceSpec<'s>) -> Self {
        SourceData {
            source_spec,
            parse_data: None,
            checked_type: None,
        }
    }

    pub fn source_spec(&self) -> &SourceSpec<'s> {
        &self.source_spec
    }
    pub fn name(&self) -> &OsStr {
        self.source_spec.name()
    }
    pub fn is_parsed(&self) -> bool {
        self.parse_data.is_some()
    }
    pub fn is_checked(&self) -> bool {
        self.checked_type.is_some()
    }
    pub fn checked_type(&self) -> &Type {
        match self.checked_type {
            Some(ref checked_type) => checked_type,
            None => unreachable!(),
        }
    }
    pub fn parse_data(&self) -> Option<&ParseData> {
        self.parse_data.as_ref()
    }
}

impl ParseData {
    pub fn num_tokens(&self) -> AstIndex {
        self.tokens.len()
    }
    pub fn char_data(&self) -> &CharData {
        &self.char_data
    }
    pub fn token(&self, token: AstIndex) -> &Token {
        &self.tokens[token]
    }
    pub fn token_string(&self, token: AstIndex) -> &str {
        use Token::*;
        match self.tokens[token] {
            IntegerLiteral(literal) => self.literal_string(literal),

            InfixOperator(operator)|
            PostfixOperator(operator)|
            PrefixOperator(operator) =>
                self.identifier_string(operator),

            CloseParen(_) => self.identifier_string(CLOSE_PAREN),
            OpenParen(_) => self.identifier_string(OPEN_PAREN),

            MissingExpression|MissingInfix => "",
        }
    }
    pub fn token_range(&self, token: AstIndex) -> ByteRange {
        let range = &self.token_ranges[token];
        Range { start: range.start, end: range.end }
    }
    pub fn identifier_string(&self, index: IdentifierIndex) -> &str {
        &self.identifiers[index]
    }
    pub fn literal_string(&self, index: LiteralIndex) -> &str {
        &self.literals[index]
    }
}

impl Display for ParseData {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "Tokens:")?;
        let mut index = AstIndex(0);
        while index < self.tokens.len() {
            let range = self.char_data().range(self.token_range(index));
            writeln!(f, "[{}] {} {:?}", range, self.token_string(index), self.token(index))?;
            index += 1;
        }
        Ok(())
    }
}

impl Default for CharData {
    fn default() -> Self { CharData { byte_length: ByteIndex::from(0), line_starts: vec![ByteIndex::from(0)] } }
}

impl CharData {
    pub fn append_line(&mut self, line_start_index: ByteIndex) {
        self.line_starts.push(line_start_index);
    }
    pub fn location(&self, index: ByteIndex) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line - 1] > index {
            line -= 1
        }

        let column = index + 1 - self.line_starts[line - 1];
        let line = line as u32;
        LineColumn { line, column }
    }

    pub fn range(&self, range: ByteRange) -> LineColumnRange {
        let start = self.location(range.start);
        if range.start == range.end {
            LineColumnRange { start, end: None }
        } else {
            let end = Some(self.location(range.end - 1));
            LineColumnRange { start, end }
        }
    }

    pub fn byte_length(&self) -> ByteIndex {
        self.byte_length
    }
}
