use crate::eval::RootRef;
use crate::syntax::char_data::CharData;
use crate::syntax::identifiers;
use crate::syntax::OperandPosition::*;
use crate::syntax::{
    AstBlock, BlockIndex, ByteRange, Expression, Field, FieldIndex, SourceOpenError, SourceReconstruction,
    SourceReconstructionReader, SourceRef, Token,
};
use crate::util::indexed_vec::IndexedVec;
use crate::util::intern_pool::{InternPool, StringPool};
use crate::value::BergError;
use std::borrow::Cow;
use std::io;
use std::rc::Rc;
use std::u32;

index_type! {
    pub struct AstIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct RawLiteralIndex(pub u32) with Display,Debug <= u32::MAX;
}

pub type Tokens = IndexedVec<Token, AstIndex>;
pub type TokenRanges = IndexedVec<ByteRange, AstIndex>;

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = Delta<AstIndex>;

// TODO stuff AstData into SourceData, and don't have AstRef anymore.
#[derive(Clone)]
pub struct AstRef<'a>(Rc<AstData<'a>>);

#[derive(Debug)]
pub struct AstData<'a> {
    pub source: SourceRef<'a>,
    pub char_data: CharData,
    pub identifiers: InternPool<IdentifierIndex>, // NOTE: if needed we can save space by removing or clearing the StringPool after making this readonly.
    pub literals: StringPool<LiteralIndex>,
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

impl<'a> AstData<'a> {
    pub fn new(
        source: SourceRef<'a>,
        source_open_error: Option<SourceOpenError<'a>>,
    ) -> AstData<'a> {
        let identifiers = source.root().identifiers();
        let fields = source
            .root()
            .field_names()
            .map(|name| Field {
                name: *name,
                is_public: false,
            })
            .collect();
        AstData {
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
    pub fn new(data: AstData<'a>) -> Self {
        AstRef(Rc::new(data))
    }

    pub fn source(&self) -> &SourceRef<'a> {
        &self.0.source
    }
    pub fn root(&self) -> &RootRef {
        self.source().root()
    }

    pub fn expression(&self) -> Expression {
        assert_ne!(self.0.tokens.len(), 0);
        Expression(AstIndex(0))
    }
    pub fn read_bytes<'p>(&'p self) -> SourceReconstructionReader<'p, 'a>
    where
        'a: 'p,
    {
        SourceReconstructionReader::new(self, 0.into()..self.char_data().size)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        SourceReconstruction::new(self, 0.into()..self.char_data().size).to_bytes()
    }
    pub fn to_string(&self) -> String {
        SourceReconstruction::new(self, 0.into()..self.char_data().size).to_string()
    }

    pub fn char_data(&self) -> &CharData {
        &self.0.char_data
    }
    pub fn identifiers(&self) -> &InternPool<IdentifierIndex> {
        &self.0.identifiers
    }
    pub fn literals(&self) -> &StringPool<LiteralIndex> {
        &self.0.literals
    }
    pub fn raw_literals(&self) -> &IndexedVec<Vec<u8>, RawLiteralIndex> {
        &self.0.raw_literals
    }
    pub fn tokens(&self) -> &IndexedVec<Token, AstIndex> {
        &self.0.tokens
    }
    pub fn token_ranges(&self) -> &IndexedVec<ByteRange, AstIndex> {
        &self.0.token_ranges
    }
    pub fn fields(&self) -> &IndexedVec<Field, FieldIndex> {
        &self.0.fields
    }
    pub fn blocks(&self) -> &IndexedVec<AstBlock, BlockIndex> {
        &self.0.blocks
    }

    pub fn token(&self, index: AstIndex) -> &Token {
        &self.0.tokens[index]
    }
    pub fn token_string(&self, index: AstIndex) -> Cow<str> {
        self.0.tokens[index].to_string(self)
    }
    pub fn token_range(&self, index: AstIndex) -> ByteRange {
        self.0.token_ranges[index].clone()
    }
    pub fn identifier_string(&self, index: IdentifierIndex) -> &str {
        &self.0.identifiers[index]
    }
    pub fn literal_string(&self, index: LiteralIndex) -> &str {
        &self.0.literals[index]
    }
    pub fn open_error(&self) -> &BergError<'a> {
        &self.0.source_open_error.as_ref().unwrap().0
    }
    pub fn open_io_error(&self) -> &io::Error {
        &self.0.source_open_error.as_ref().unwrap().1
    }
    pub fn field_name(&self, index: FieldIndex) -> &str {
        self.identifier_string(self.fields()[index].name)
    }
}

impl<'a> fmt::Debug for AstRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ast({:?})", self.source().name())
    }
}

impl<'a> AstData<'a> {
    pub fn push_token(&mut self, token: Token, range: ByteRange) -> AstIndex {
        self.tokens.push(token);
        self.token_ranges.push(range)
    }

    pub fn insert_token(&mut self, index: AstIndex, token: Token, range: ByteRange) {
        self.tokens.insert(index, token);
        self.token_ranges.insert(index, range);
    }

    pub fn next_index(&self) -> AstIndex {
        self.tokens.next_index()
    }
}

impl<'a> fmt::Display for AstRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Tokens:")?;
        let mut index = AstIndex(0);
        while index < self.tokens().len() {
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

impl fmt::Display for IdentifierIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < identifiers::LEN as u32 {
            write!(f, "{}", identifiers::identifier_string(*self))
        } else {
            write!(f, "#{}", self.0)
        }
    }
}
impl fmt::Debug for IdentifierIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < identifiers::LEN as u32 {
            write!(f, "{}", identifiers::identifier_string(*self))
        } else {
            write!(f, "#{}", self.0)
        }
    }
}

impl OperandPosition {
    pub(crate) fn get(self, expression: Expression, ast: &AstRef) -> Expression {
        match self {
            Left | PostfixOperand => expression.left_expression(ast),
            Right | PrefixOperand => expression.right_expression(ast),
        }
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
