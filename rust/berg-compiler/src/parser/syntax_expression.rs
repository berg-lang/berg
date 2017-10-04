use public::*;
use std::ops::Range;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug)]
pub struct SyntaxExpression {
    pub expression_type: SyntaxExpressionType,
    pub start: ByteIndex,
    pub string: String,
}

impl SyntaxExpression {
    pub fn new(expression_type: SyntaxExpressionType, start: ByteIndex, string: String) -> SyntaxExpression {
        SyntaxExpression { expression_type, start, string }
    }
    pub fn range(&self) -> Range<ByteIndex> {
        let len = self.string.len() as ByteIndex;
        Range { start: self.start, end: self.start + len }
    }
}

#[derive(Debug)]
pub enum SyntaxExpressionType {
    IntegerLiteral,
}
