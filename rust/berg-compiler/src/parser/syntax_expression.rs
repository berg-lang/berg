use public::*;

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
}

#[derive(Debug)]
pub enum SyntaxExpressionType {
    IntegerLiteral,
}
