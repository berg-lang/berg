use super::AstRef;
use crate::syntax::{AstExpressionTree, AstIndex, ExpressionPosition, ExpressionTreeWalker};
use std::fmt;

#[derive(Clone)]
pub struct ExpressionRef {
    pub ast: AstRef,
    pub root: AstIndex,
}

impl ExpressionRef {
    pub fn new(ast: AstRef, root: AstIndex) -> Self {
        ExpressionRef { ast, root }
    }
    pub fn expression(&self) -> ExpressionTreeWalker {
        ExpressionTreeWalker::basic(&self.ast, self.root)
    }
    pub fn at_position(mut self, position: ExpressionPosition) -> Self {
        self.root = AstExpressionTree::new(&self.ast, self.root)
            .at_position(position)
            .root_index();
        self
    }
}

impl fmt::Debug for ExpressionRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.expression())
    }
}

impl fmt::Display for ExpressionRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expression())
    }
}
