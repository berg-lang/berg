use parser::*;
use indexed_vec::IndexedVec;
use parser::token_pool::TokenPool;

#[derive(Debug,Default)]
pub(crate) struct AstBuilder {
    pub identifier_pool: TokenPool<IdentifierIndex>,
    pub literal_strings: IndexedVec<String,LiteralIndex>,
    pub tokens: IndexedVec<Token,AstIndex>,
    pub token_ranges: IndexedVec<Range<ByteIndex>,AstIndex>,
}

impl AstBuilder {
    pub fn append_literal<F: Fn(LiteralIndex)->Token>(&mut self, buffer: &[u8], start: ByteIndex, end: ByteIndex, to_token: F) {
        let literal = self.literal(buffer, start, end);
        self.append(to_token(literal), start..end);
    }

    pub fn append_operator<F: Fn(IdentifierIndex)->Token>(&mut self, buffer: &[u8], start: ByteIndex, end: ByteIndex, to_token: F) {
        let identifier = self.identifier(buffer, start, end);
        self.append(to_token(identifier), start..end);
    }

    pub fn append(&mut self, token: Token, range: Range<ByteIndex>) {
        self.tokens.push(token);
        self.token_ranges.push(range);
    }

    fn literal(&mut self, buffer: &[u8], start: ByteIndex, end: ByteIndex) -> LiteralIndex {
        use std::str;
        let literal = self.literal_strings.len();
        let string = unsafe { str::from_utf8_unchecked(&buffer[usize::from(start)..usize::from(end)]) };
        self.literal_strings.push(string.to_string());
        literal
    }

    fn identifier(&mut self, buffer: &[u8], start: ByteIndex, end: ByteIndex) -> IdentifierIndex {
        unsafe { self.identifier_pool.intern_unchecked(&buffer[usize::from(start)..usize::from(end)]) }
    }
}
