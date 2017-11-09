use ast::operators::*;
use indexed_vec::IndexedSlice;
use std::fmt::*;
use ast::intern_pool::StringPool;
use ast::{AstIndex,IdentifierIndex,LiteralIndex};
use ast::token::Token;
use ast::token::Token::*;
use checker::checker_type::Type;
use compiler::source_spec::SourceSpec;
use compiler::line_column::{LineColumn,LineColumnRange};
use indexed_vec::IndexedVec;
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
    pub(crate) char_data: CharData,
    pub(crate) identifiers: StringPool<IdentifierIndex>,
    pub(crate) literals: StringPool<LiteralIndex>,
    pub(crate) tokens: IndexedVec<Token,AstIndex>,
    pub(crate) token_ranges: IndexedVec<ByteRange,AstIndex>,
}

#[derive(Debug)]
pub struct CharData {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    pub(crate) byte_length: ByteIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    // Start indices of each line
    pub(crate) line_starts: Vec<ByteIndex>,
}

impl<'s> SourceData<'s> {
    pub(crate) fn new(source_spec: SourceSpec<'s>) -> Self {
        SourceData {
            source_spec,
            parse_data: None,
            checked_type: None,
        }
    }

    pub(crate) fn source_spec(&self) -> &SourceSpec<'s> {
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
    pub(crate) fn num_tokens(&self) -> AstIndex {
        self.tokens.len()
    }
    pub(crate) fn char_data(&self) -> &CharData {
        &self.char_data
    }
    pub(crate) fn token(&self, token: AstIndex) -> &Token {
        &self.tokens[token]
    }
    pub(crate) fn token_string(&self, token: AstIndex) -> &str {
        use ast::token::ExpressionBoundary::*;
        match self.tokens[token] {
            IntegerLiteral(literal) => self.literal_string(literal),

            InfixOperator(operator)|
            PostfixOperator(operator)|
            PrefixOperator(operator) =>
                self.identifier_string(operator),

            Close(Parentheses,_) => self.identifier_string(CLOSE_PAREN),
            Open(Parentheses,_) => self.identifier_string(OPEN_PAREN),
            Open(CompoundTerm,_)|Close(CompoundTerm,_)|Open(PrecedenceGroup,_)|Close(PrecedenceGroup,_)|Open(File,_)|Close(File,_)|MissingExpression|MissingInfix => "",
        }
    }
    pub(crate) fn token_range(&self, token: AstIndex) -> ByteRange {
        let range = &self.token_ranges[token];
        Range { start: range.start, end: range.end }
    }
    pub(crate) fn identifier_string(&self, index: IdentifierIndex) -> &str {
        &self.identifiers[index]
    }
    pub(crate) fn literal_string(&self, index: LiteralIndex) -> &str {
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
    // pub(crate) fn append_line(&mut self, line_start_index: ByteIndex) {
    //     self.line_starts.push(line_start_index);
    // }
    pub(crate) fn location(&self, index: ByteIndex) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line - 1] > index {
            line -= 1
        }

        let column = index + 1 - self.line_starts[line - 1];
        let line = line as u32;
        LineColumn { line, column }
    }

    pub(crate) fn range(&self, range: ByteRange) -> LineColumnRange {
        let start = self.location(range.start);
        if range.start == range.end {
            LineColumnRange { start, end: None }
        } else {
            let end = Some(self.location(range.end - 1));
            LineColumnRange { start, end }
        }
    }

    // pub(crate) fn byte_length(&self) -> ByteIndex {
    //     self.byte_length
    // }
}
