pub trait Expression {
    pub fn fixity(&self) -> Fixity;
    pub fn value<T>(&node: ExpressionTreeNode<T>) -> Fixity;
}

pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
}
