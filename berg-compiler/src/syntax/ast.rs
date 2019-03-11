use crate::eval::RootRef;
use crate::syntax::char_data::CharData;
use crate::syntax::OperandPosition::*;
use crate::syntax::{
    AstBlock, BlockIndex, ByteRange, ExpressionTreeWalker, Field, FieldIndex, IdentifierIndex,
    SourceOpenError, SourceReconstruction, SourceReconstructionReader, SourceRef, ExpressionToken, OperatorToken, Token,
};
use crate::util::indexed_vec::IndexedVec;
use crate::value::BergError;
use std::borrow::Cow;
use std::io;
use std::ops::Deref;
use std::rc::Rc;
use std::u32;
use string_interner::{StringInterner, Sym};

index_type! {
    pub struct AstIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct RawLiteralIndex(pub u32) with Display,Debug <= u32::MAX;
}

pub type LiteralIndex = Sym;

pub type Tokens = IndexedVec<Token, AstIndex>;
pub type TokenRanges = IndexedVec<ByteRange, AstIndex>;

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = Delta<AstIndex>;

// TODO stuff Ast into SourceData, and don't have AstRef anymore.
#[derive(Debug, Clone)]
pub struct AstRef<'a>(Rc<Ast<'a>>);

pub struct Ast<'a> {
    pub source: SourceRef<'a>,
    pub char_data: CharData,
    pub identifiers: StringInterner<IdentifierIndex>,
    pub literals: StringInterner<LiteralIndex>,
    pub raw_literals: IndexedVec<Vec<u8>, RawLiteralIndex>,
    pub tokens: Tokens,
    pub token_ranges: TokenRanges,
    pub blocks: IndexedVec<AstBlock, BlockIndex>,
    pub fields: IndexedVec<Field, FieldIndex>,
    pub source_open_error: Option<SourceOpenError<'a>>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperandPosition {
    Left,
    Right,
    PrefixOperand,
    PostfixOperand,
}

impl<'a> Ast<'a> {
    pub fn new(source: SourceRef<'a>, source_open_error: Option<SourceOpenError<'a>>) -> Ast<'a> {
        let identifiers = source.root().identifiers();
        let fields = source
            .root()
            .field_names()
            .map(|name| Field {
                name: *name,
                is_public: false,
            })
            .collect();
        Ast {
            source,
            identifiers,
            fields,
            source_open_error,

            char_data: Default::default(),
            literals: Default::default(),
            raw_literals: Default::default(),
            blocks: Default::default(),
            tokens: Default::default(),
            token_ranges: Default::default(),
        }
    }
}

impl<'a> AstRef<'a> {
    pub fn new(data: Ast<'a>) -> Self {
        AstRef(Rc::new(data))
    }
}

impl<'a> Deref for AstRef<'a> {
    type Target = Ast<'a>;
    fn deref(&self) -> &Ast<'a> {
        &self.0
    }
}

impl<'a> Ast<'a> {
    pub fn root(&self) -> &RootRef {
        self.source.root()
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
    pub fn open_error(&self) -> &BergError<'a> {
        &self.source_open_error.as_ref().unwrap().0
    }
    pub fn open_io_error(&self) -> &io::Error {
        &self.source_open_error.as_ref().unwrap().1
    }
    pub fn field_name(&self, index: FieldIndex) -> &str {
        self.identifier_string(self.fields[index].name)
    }

    pub fn expression<'p>(&'p self) -> ExpressionTreeWalker<'p, 'a> {
        assert_ne!(self.tokens.len(), 0);
        ExpressionTreeWalker::new((), self, AstIndex(0))
    }

    pub fn read_bytes<'p>(&'p self) -> SourceReconstructionReader<'p, 'a>
    where
        'a: 'p,
    {
        SourceReconstructionReader::new(self, 0.into()..self.char_data.size)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        SourceReconstruction::new(self, 0.into()..self.char_data.size).to_bytes()
    }
    pub fn to_string(&self) -> String {
        SourceReconstruction::new(self, 0.into()..self.char_data.size).to_string()
    }

    pub fn push_token(&mut self, token: impl Into<Token>, range: ByteRange) -> AstIndex {
        self.tokens.push(token.into());
        self.token_ranges.push(range)
    }

    pub fn insert_token(&mut self, index: AstIndex, token: impl Into<Token>, range: ByteRange) {
        self.tokens.insert(index, token.into());
        self.token_ranges.insert(index, range);
    }

    pub fn next_index(&self) -> AstIndex {
        self.tokens.next_index()
    }

    pub fn intern_identifier(&mut self, string: impl Into<String> + AsRef<str>) -> IdentifierIndex {
        self.identifiers.get_or_intern(string)
    }
}

impl<'a> fmt::Debug for Ast<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ast({:?})", self.source.name())
    }
}

impl<'a> fmt::Display for Ast<'a> {
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
