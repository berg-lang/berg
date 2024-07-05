use super::char_data::CharData;
use super::identifiers::keywords;
use super::OperandPosition::*;
use super::{
    AstBlock, BlockIndex, ByteRange, ExpressionToken, Field, FieldIndex, IdentifierIndex,
    OperatorToken, SourceReconstruction, SourceReconstructionReader, Token,
};
use crate::util::indexed_vec::IndexedVec;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::u32;
use string_interner::backend::StringBackend;
use string_interner::{DefaultSymbol, StringInterner, Symbol};

index_type! {
    pub struct AstIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct RawLiteralIndex(pub u32) with Display,Debug <= u32::MAX;
}

pub type LiteralIndex = DefaultSymbol;

pub type Tokens = IndexedVec<Token, AstIndex>;
pub type TokenRanges = IndexedVec<ByteRange, AstIndex>;

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = Delta<AstIndex>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct WhitespaceIndex(NonZeroU32);

#[derive(Debug)]
pub struct Ast {
    pub char_data: CharData,
    pub identifiers: StringInterner<StringBackend<IdentifierIndex>>,
    pub literals: StringInterner<StringBackend<LiteralIndex>>,
    pub raw_literals: IndexedVec<Vec<u8>, RawLiteralIndex>,
    pub tokens: Tokens,
    pub token_ranges: TokenRanges,
    pub blocks: IndexedVec<AstBlock, BlockIndex>,
    pub fields: IndexedVec<Field, FieldIndex>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperandPosition {
    Left,
    Right,
    PrefixOperand,
    PostfixOperand,
}

impl Default for Ast {
    fn default() -> Ast {
        let identifiers = super::identifiers::intern_all();
        let fields = keywords::FIELD_NAMES
            .iter()
            .map(|name| Field {
                name: *name,
                is_public: false,
            })
            .collect();
        Ast {
            identifiers,
            fields,
            char_data: Default::default(),
            literals: Default::default(),
            raw_literals: Default::default(),
            blocks: Default::default(),
            tokens: Default::default(),
            token_ranges: Default::default(),
        }
    }
}

impl Ast {
    pub fn token(&self, index: AstIndex) -> Token {
        self.tokens[index]
    }
    pub fn expression_token(&self, index: AstIndex) -> ExpressionToken {
        match self.tokens[index] {
            Token::Expression(token) => token,
            Token::Operator(_) => unreachable!(),
        }
    }
    pub fn operator_token(&self, index: AstIndex) -> OperatorToken {
        match self.tokens[index] {
            Token::Operator(token) => token,
            Token::Expression(_) => unreachable!(),
        }
    }
    pub fn close_block_index(&self, index: AstIndex) -> BlockIndex {
        match self.tokens[index] {
            Token::Operator(OperatorToken::CloseBlock(block_index, _)) => block_index,
            _ => unreachable!(),
        }
    }
    pub fn token_string(&self, index: AstIndex) -> Cow<str> {
        self.tokens[index].to_string(self)
    }
    pub fn visible_token_string(&self, index: AstIndex) -> Cow<str> {
        self.tokens[index].to_visible_string(self)
    }
    pub fn token_range(&self, index: AstIndex) -> ByteRange {
        self.token_ranges[index].clone()
    }
    pub fn identifier_string(&self, index: IdentifierIndex) -> &str {
        self.identifiers.resolve(index).unwrap()
    }
    pub fn literal_string(&self, index: LiteralIndex) -> &str {
        self.literals.resolve(index).unwrap()
    }
    pub fn raw_literal_string(&self, index: RawLiteralIndex) -> &[u8] {
        &self.raw_literals[index]
    }
    pub fn whitespace_string(&self, index: WhitespaceIndex) -> &str {
        self.char_data.whitespace_characters.resolve(index).unwrap()
    }
    pub fn field_name(&self, index: FieldIndex) -> &str {
        self.identifier_string(self.fields[index].name)
    }

    pub fn root_expression(&self) -> AstIndex {
        AstIndex(0)
    }

    pub fn read_bytes(&self) -> SourceReconstructionReader {
        SourceReconstructionReader::new(self, 0.into()..self.char_data.size)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        SourceReconstruction::new(self, 0.into()..self.char_data.size).to_bytes()
    }

    pub fn push_token(&mut self, token: impl Into<Token>, range: ByteRange) -> AstIndex {
        let token = token.into();
        println!("PUSH {:?}", token);
        // Validate that we push tokens in increasing order
        assert!(
            match self.token_ranges.last() {
                Some(last) => range.start >= last.end,
                None => true,
            },
            "Pushing token {:?} too early! Last token ended at {} and this one starts at {}",
            token,
            self.token_ranges.last().unwrap().end,
            range.start
        );
        self.tokens.push(token);
        self.token_ranges.push(range)
    }

    pub fn insert_token(&mut self, index: AstIndex, token: impl Into<Token>, range: ByteRange) {
        let token = token.into();
        println!("INSERT {:?} AT {}", token, index);
        assert!(index == 0 || range.start >= self.token_ranges[index - 1].end);
        assert!(index == self.token_ranges.len() || range.end <= self.token_ranges[index].start);
        self.tokens.insert(index, token);
        self.token_ranges.insert(index, range);
    }

    pub fn next_index(&self) -> AstIndex {
        self.tokens.next_index()
    }

    pub fn intern_identifier(&mut self, string: impl Into<String> + AsRef<str>) -> IdentifierIndex {
        self.identifiers.get_or_intern(string)
    }
    pub fn intern_literal(&mut self, string: impl Into<String> + AsRef<str>) -> LiteralIndex {
        self.literals.get_or_intern(string)
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Tokens:")?;
        let mut index = AstIndex(0);
        while index < self.tokens.len() {
            let range = self.char_data.range(&self.token_range(index));
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

impl fmt::Display for OperandPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            Left | PostfixOperand => "left side",
            Right | PrefixOperand => "right side",
        };
        write!(f, "{}", string)
    }
}

// For StringInterner
impl Symbol for WhitespaceIndex {
    fn try_from_usize(val: usize) -> Option<Self> {
        if val < u32::MAX as usize {
            Some(WhitespaceIndex(unsafe {
                NonZeroU32::new_unchecked((val + 1) as u32)
            }))
        } else {
            None
        }
    }

    fn to_usize(self) -> usize {
        (self.0.get() as usize) - 1
    }
}
