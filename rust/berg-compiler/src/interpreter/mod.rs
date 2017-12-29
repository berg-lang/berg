use interpreter::value::Block;
use interpreter::value::Value;
use compiler::Compiler;
use interpreter::evaluator::ExpressionEvaluator;

pub(crate) mod evaluator;
pub mod value;

pub(crate) fn run(compiler: &Compiler, source_block: &Block) -> Value {
    let mut evaluator = ExpressionEvaluator::new(compiler);
    evaluator.evaluate_block(source_block)
}
