mod block;
mod expression_eval;

pub use self::block::BlockRef;
pub use self::expression_eval::ExpressionEvaluator;

use crate::value::*;

pub fn evaluate_ast(ast: AstRef) -> BergResult {
    BlockRef::from_ast(ast.clone())?.evaluate()
}
