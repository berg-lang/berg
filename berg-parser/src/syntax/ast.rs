use berg_util::{index_type, IndexedVec};
use std::borrow::Cow;
use std::u32;
use string_interner::backend::StringBackend;
use string_interner::{DefaultSymbol, StringInterner};

use crate::bytes::ByteRange;

use super::block::{AstBlock, BlockIndex, Field, FieldIndex};
use super::char_data::{CharData, WhitespaceIndex};
use super::identifiers::IdentifierIndex;
use super::source_reconstruction::{SourceReconstruction, SourceReconstructionReader};
use super::token::{ExpressionToken, OperatorToken, Token};
use OperandPosition::*;

index_type! {
    pub struct AstIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct RawLiteralIndex(pub u32) with Display,Debug <= u32::MAX;
}

pub type LiteralIndex = DefaultSymbol;

pub type Tokens = IndexedVec<Token, AstIndex>;
pub type TokenRanges = IndexedVec<ByteRange, AstIndex>;

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = Delta<AstIndex>;

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

impl Ast {
    pub fn empty() -> Ast {
        Ast {
            identifiers: StringInterner::new(),
            fields: Default::default(),
            char_data: Default::default(),
            literals: Default::default(),
            raw_literals: Default::default(),
            blocks: Default::default(),
            tokens: Default::default(),
            token_ranges: Default::default(),
        }
    }
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
        SourceReconstructionReader::new(self, 0.into()..self.char_data.lines.eof)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        SourceReconstruction::new(self, 0.into()..self.char_data.lines.eof).to_bytes()
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Tokens:")?;
        let mut index = AstIndex(0);
        while index < self.tokens.len() {
            let range = self.char_data.lines.range(&self.token_range(index));
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
