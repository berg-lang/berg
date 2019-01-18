mod block;
mod expression_eval;
mod expression_formatter;
mod root;
mod scope;

pub use self::block::BlockRef;
pub use self::expression_eval::{ExpressionEval, OperandEval};
pub use self::expression_formatter::{ExpressionFormatter, ExpressionTreeFormatter};
pub use self::root::root_fields;
pub use self::root::RootRef;
pub use self::scope::ScopeRef;

use crate::error::BergResult;
use crate::syntax::AstRef;
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::BergVal;

#[allow(clippy::needless_pass_by_value)]
pub fn evaluate_ast(ast: AstRef) -> BergResult {
    let mut scope = ScopeRef::AstRef(ast.clone());
    ast.expression().result(&mut scope, &ast)
}

#[allow(clippy::needless_pass_by_value)]
pub fn evaluate_ast_to<'a, T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(ast: AstRef<'a>) -> BergResult<'a, T> {
    let mut scope = ScopeRef::AstRef(ast.clone());
    ast.expression().result_to(&mut scope, &ast)
}
