mod block;
mod expression_eval;
mod root;

pub use self::block::BlockRef;
pub use self::expression_eval::ExpressionEvaluator;
pub use self::root::keywords;
pub use self::root::RootRef;

use crate::syntax::AstRef;
use crate::value::*;

pub fn evaluate_ast(ast: AstRef) -> BergResult {
    BlockRef::from_ast(ast.clone())?.evaluate()
}
