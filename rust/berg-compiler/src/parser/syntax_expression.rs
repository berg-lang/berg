// ExpressionType, String, LeftChild, RightChild
pub struct SyntaxExpression {
    pub expression_type: SyntaxExpressionType,
    pub string: String,
    pub start: usize,
}

pub enum SyntaxExpressionType {
    IntegerLiteral,
}
