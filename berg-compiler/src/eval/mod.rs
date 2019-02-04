mod block;
mod expression_eval;
mod root;
mod scope;

pub use self::block::BlockRef;
pub use self::expression_eval::ExpressionEvaluator;
pub use self::root::root_fields;
pub use self::root::RootRef;
pub use self::scope::ScopeRef;

use crate::syntax::AstRef;
use crate::value::BergResult;

#[allow(clippy::needless_pass_by_value)]
pub fn evaluate_ast(ast: AstRef) -> BergResult {
    let scope = ScopeRef::AstRef(ast);
    if let ScopeRef::AstRef(ref ast) = scope {
        ExpressionEvaluator::new(&scope, ast, ast.expression()).evaluate()
    } else {
        unreachable!()
    }
}
