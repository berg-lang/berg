use ast::Variable;
use ast::VariableIndex;
use util::indexed_vec::IndexedVec;
use interpreter::value::Value;
use ast::intern_pool::InternPool;
use source::line_column::LineColumnRange;
use source::line_column::LineColumn;
use std::fmt::Formatter;
use std::fmt::Display;
use util::indexed_vec::IndexedSlice;
use std::ops::Range;
use ast::token::Token;
use ast::token::Token::*;
use ast::identifiers::*;
use ast::AstIndex;
use ast::TokenRanges;
use ast::Tokens;
use ast::LiteralIndex;
use ast::intern_pool::StringPool;
use ast::IdentifierIndex;
use ast::identifiers;
use source::SourceIndex;
use std;
use std::u32;
use ast;

index_type! {
    pub struct ByteIndex(pub u32) <= u32::MAX;
}
pub type ByteSlice = IndexedSlice<u8, ByteIndex>;
pub type ByteRange = Range<ByteIndex>;

#[derive(Debug)]
pub struct ParseResult {
    pub(crate) index: SourceIndex,
    pub(crate) char_data: CharData,
    pub(crate) identifiers: InternPool<IdentifierIndex>, // NOTE: if needed we can save space by removing or clearing the StringPool after making this readonly.
    pub(crate) literals: StringPool<LiteralIndex>,
    pub(crate) tokens: Tokens,
    pub(crate) token_ranges: TokenRanges,
    pub(crate) variables: IndexedVec<Variable, VariableIndex>,
    pub(crate) open_error: Option<Value>,
    pub(crate) is_parsed: bool,
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

impl ParseResult {
    pub(crate) fn new(index: SourceIndex) -> Self {
        ParseResult {
            index,
            identifiers: identifiers::intern_all(),
            is_parsed: false,

            char_data: Default::default(),
            literals: Default::default(),
            tokens: Default::default(),
            token_ranges: Default::default(),
            variables: Self::root_variables(),
            open_error: None,
        }
    }
    fn root_variables() -> IndexedVec<Variable, VariableIndex> {
        ast::root_variables()
            .iter()
            .map(|variable| Variable { name: variable.0 })
            .collect()
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
            ErrorTerm(_) => "error",

            VariableReference(variable) => self.identifier_string(self.variables[variable].name),

            RawIdentifier(identifier)
            | InfixOperator(identifier)
            | InfixAssignment(identifier)
            | PostfixOperator(identifier)
            | PrefixOperator(identifier) => self.identifier_string(identifier),

            NewlineSequence => "\\n",
            Close {
                boundary: Parentheses,
                ..
            } => self.identifier_string(CLOSE_PAREN),
            Close {
                boundary: CurlyBraces,
                ..
            } => self.identifier_string(CLOSE_CURLY),
            Open {
                boundary: Parentheses,
                ..
            } => self.identifier_string(OPEN_PAREN),
            Open {
                boundary: CurlyBraces,
                ..
            } => self.identifier_string(OPEN_CURLY),
            Open {
                boundary: CompoundTerm,
                ..
            }
            | Close {
                boundary: CompoundTerm,
                ..
            }
            | Open {
                boundary: PrecedenceGroup,
                ..
            }
            | Close {
                boundary: PrecedenceGroup,
                ..
            }
            | Open {
                boundary: Source, ..
            }
            | Close {
                boundary: Source, ..
            }
            | MissingExpression
            | MissingInfix => "",
        }
    }
    pub(crate) fn token_range(&self, token: AstIndex) -> ByteRange {
        self.token_ranges[token].clone()
    }
    pub(crate) fn identifier_string(&self, index: IdentifierIndex) -> &str {
        &self.identifiers[index]
    }
    pub(crate) fn literal_string(&self, index: LiteralIndex) -> &str {
        &self.literals[index]
    }
    pub(crate) fn report_open_error(&mut self, error: Value) {
        assert!(self.open_error.is_none());
        self.open_error = Some(error);
    }

    pub(crate) fn push_token(&mut self, token: Token, range: ByteRange) -> AstIndex {
        println!("TOKEN {:?} at {}", token, self.next_index());
        self.tokens.push(token);
        self.token_ranges.push(range)
    }

    pub(crate) fn insert_token(&mut self, index: AstIndex, token: Token, range: ByteRange) {
        println!("INSERT TOKEN {:?} at {}", token, index);
        self.tokens.insert(index, token);
        self.token_ranges.insert(index, range);
    }

    pub(crate) fn next_index(&self) -> AstIndex {
        self.tokens.next_index()
    }
}

impl Display for ParseResult {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "Tokens:")?;
        let mut index = AstIndex(0);
        while index < self.tokens.len() {
            let range = self.char_data().range(&self.token_range(index));
            writeln!(
                f,
                "[{}] {} {:?}",
                range,
                self.token_string(index),
                self.token(index)
            )?;
            index += 1;
        }
        Ok(())
    }
}

impl Default for CharData {
    fn default() -> Self {
        CharData {
            byte_length: ByteIndex::from(0),
            line_starts: vec![ByteIndex::from(0)],
        }
    }
}

impl CharData {
    pub(crate) fn append_line(&mut self, line_start: ByteIndex) {
        self.line_starts.push(line_start);
    }
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

    pub(crate) fn range(&self, range: &ByteRange) -> LineColumnRange {
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
