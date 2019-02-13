mod ambiguous_syntax;
mod assignment_target;
mod block;
mod expression_eval;
mod root;
mod scope;

pub use self::assignment_target::AssignmentTarget;
pub use self::ambiguous_syntax::AmbiguousSyntax;
pub use self::block::BlockRef;
pub use self::expression_eval::ExpressionEvaluator;
pub use self::root::root_fields;
pub use self::root::RootRef;
pub use self::scope::ScopeRef;

use crate::syntax::AstRef;
use crate::value::BergResult;

pub fn evaluate_ast(ast: AstRef) -> BergResult {
    let scope = ScopeRef::AstRef(ast.clone());
    if let ScopeRef::AstRef(ref ast) = scope {
        ast.expression().with_context(&scope).evaluate()
    } else {
        unreachable!()
    }
}
