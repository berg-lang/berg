mod block;
mod expression_eval;

pub use self::block::BlockRef;
pub use self::expression_eval::ExpressionEvaluator;

use crate::value::*;

pub fn evaluate_ast<'a>(ast: AstRef) -> BergResult<'a> {
    BlockRef::from_ast(ast.clone())?.evaluate()
}
