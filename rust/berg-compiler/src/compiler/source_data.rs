use ast::intern_pool::StringPool;
use ast::{AstIndex,IdentifierIndex,LiteralIndex};
use ast::token::PrefixToken::*;
use ast::token::PostfixToken::*;
use indexed_vec::IndexedVec;
use public::*;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::Range;
use std::u32;

index_type! {
    pub struct SourceIndex(pub u32) <= u32::MAX;
    pub struct ByteIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub struct SourceData<'c> {
    source_spec: SourceSpec,
    pub(crate) parse_data: Option<ParseData>,
    pub(crate) checked_type: Option<Type>,
    phantom: PhantomData<&'c Compiler<'c>>,
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
    pub fn num_tokens(&self) -> AstIndex {
        match self.parse_data {
            Some(ref parse_data) => parse_data.tokens.len(),
            None => unreachable!(),
        }
    }
    pub fn token(&self, token: AstIndex) -> &Token {
        match self.parse_data {
            Some(ref parse_data) => &parse_data.tokens[token],
            None => unreachable!(),
        }
    }
    pub fn token_string(&self, token: AstIndex) -> &str {
        use Token::*;
        use TermToken::*;
        use InfixToken::*;
        match self.parse_data().tokens[token] {
            Term(IntegerLiteral(literal)) => self.literal_string(literal),

            Infix(InfixOperator(operator))|
            Postfix(PostfixOperator(operator))|
            Postfix(PostfixToken::Close(operator))|
            Prefix(PrefixOperator(operator))|
            Prefix(PrefixToken::Open(operator)) =>
                self.identifier_string(operator),

            Term(MissingOperand)|Term(NoExpression)|Infix(MissingInfix) => "",
        }
    }
    pub fn token_range(&self, token: AstIndex) -> Range<ByteIndex> {
        let range = &self.parse_data().token_ranges[token];
        Range { start: range.start, end: range.end }
    }
    pub fn identifier_string(&self, index: IdentifierIndex) -> &str {
        &self.parse_data().identifiers[index]
    }
    pub fn literal_string(&self, index: LiteralIndex) -> &str {
        &self.parse_data().literals[index]
    }
    fn parse_data(&self) -> &ParseData {
        self.parse_data.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub(crate) struct ParseData {
    pub char_data: CharData,
    pub identifiers: StringPool<IdentifierIndex>,
    pub literals: StringPool<LiteralIndex>,
    pub tokens: IndexedVec<Token,AstIndex>,
    pub token_ranges: IndexedVec<Range<ByteIndex>,AstIndex>,
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

        let column = index - self.line_starts[line - 1] + 1;
        let line = line as u32;
        LineColumn { line, column }
    }

    pub fn range(&self, range: Range<ByteIndex>) -> LineColumnRange {
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
