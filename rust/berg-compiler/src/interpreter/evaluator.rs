// Evaluates a given source expression.
use ast::token::ExpressionBoundary;
use compiler::Compiler;
use util::indexed_vec::IndexedVec;
use ast::expression::SourceExpression;
use source::SourceIndex;
use ast::token::Token::*;
use num::BigRational;
use interpreter::value::*;
use interpreter::evaluator::BlockState::*;
use ast::IdentifierIndex;
use ast::expression::Expression;
use ast::token::ExpressionBoundary::*;
use ast::token::ExpressionBoundaryError;
use interpreter::value::Value;
use std::str::FromStr;
use num::BigInt;
use num::Zero;
use ast::expression::OperandType;
use ast::expression::OperandPosition;
use ast::expression::OperandPosition::*;
use ast::identifiers::*;
use std::u32;
use compile_errors::*;
use util::display_context::DisplayContext;

index_type! {
    pub struct CallStackIndex(pub u32) <= u32::MAX;
    pub struct VariableIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub(super) struct ExpressionEvaluator<'e> {
    call_stack: IndexedVec<BlockData,CallStackIndex>,
    variables: IndexedVec<Variable,VariableIndex>,
    compiler: &'e Compiler,
    root_variable_count: VariableIndex,
}

#[derive(Debug)]
pub(super) struct BlockData {
    block: Block,
    parent_scope: Option<CallStackIndex>,
    index: SourceIndex,
    expression: Expression,
    first_variable: VariableIndex,
}

#[derive(Debug)]
struct Variable {
    name: IdentifierIndex,
    value: Value,
    is_field: bool,
}

// Represents process data--either it's not run, or it's running, or it's
// finished and has produced a value.
#[derive(Debug)]
pub(crate) enum BlockState {
    SourceNotStarted(SourceIndex),
    NotStarted { parent_scope: Block, expression: Expression },
    Running(CallStackIndex),
    Complete(Value),
}

impl BlockData {
    fn new(block: Block, index: SourceIndex, expression: Expression, parent_scope: Option<CallStackIndex>, first_variable: VariableIndex) -> Self {
        BlockData {
            parent_scope,
            index,
            expression,
            block,
            first_variable,
        }
    }
}

impl BlockState {
    // Used so that we don't have to keep a reference around during a match. Not a huge deal
    // since we had to clone most of the stuff anyway (except parent_scope :()
    fn clone_internal(&self) -> Self {
        match *self {
            BlockState::SourceNotStarted(i) => BlockState::SourceNotStarted(i),
            BlockState::NotStarted{ref parent_scope,expression} => BlockState::NotStarted{parent_scope: parent_scope.clone(),expression},
            BlockState::Running(i) => BlockState::Running(i),
            BlockState::Complete(ref v) => BlockState::Complete(v.clone()),
        }
    }
}

// use source::Source;

// use std::cell::Ref;
// use std::cell::RefCell;
// fn blah<'e,F:FnMut(Ref<'e,usize>)->()>(mut f: F) {
//     let x: RefCell<usize+'e> = Default::default();
//     let b = x.borrow();
//     f(b)
// }

impl<'e> ExpressionEvaluator<'e> {
    fn new(compiler: &'e Compiler, variables: IndexedVec<Variable,VariableIndex>) -> Self {
        let root_variable_count = variables.len();
        ExpressionEvaluator {
            compiler,
            call_stack: Default::default(),
            variables,
            root_variable_count,
        }
    }
    pub(crate) fn run(compiler: &'e Compiler, source_block: &Block) -> Value {
        let index = match *source_block.state() {
            BlockState::SourceNotStarted(index) => index,
            BlockState::Complete(ref value) => return value.clone(),
            BlockState::Running(..)|BlockState::NotStarted {..} => unreachable!(),
        };

        let variables = vec![
            Variable { name: TRUE, value: true.into(), is_field: false },
            Variable { name: FALSE, value: false.into(), is_field: false },
        ].into();
        let mut evaluator = Self::new(compiler, variables);

        assert!(source_block.0.try_borrow_mut().is_ok());
        assert!(source_block.0.try_borrow().is_ok());

        evaluator.evaluate_source(index)
    }

    pub(crate) fn evaluate_block(&mut self, block: &Block) -> Value {
        // Decide whether we need to execute the block or not.
        let state = block.state().clone_internal();
        match state {
            SourceNotStarted(index) => self.evaluate_source(index),
            NotStarted{parent_scope,expression} => self.evaluate_child_block(block, &parent_scope, expression),
            Complete(value) => value,
            Running(index) => self.circular_dependency_error(index),
        }
    }

    fn evaluate_source(&mut self, index: SourceIndex) -> Value {
        let source_rc = self.compiler.source(index);
        let source = source_rc.borrow();
        let parse_result = source.parse(self.compiler);
        let expression = Expression::from_source(&parse_result);
        let block_data = BlockData::new(source.value.clone(), parse_result.index, expression, None, self.variables.len());
        self.evaluate_block_data(block_data, SourceExpression { parse_result: &parse_result, expression })
    }

    fn evaluate_child_block(&mut self, block: &Block, parent_scope: &Block, expression: Expression) -> Value {
        // Create and push the new BlockData
        let parent_scope_index = match *parent_scope.state() {
            Running(index) => index,
            Complete(..) => panic!("Passing a block as a result of a block is not presently supported!"),
            SourceNotStarted(..)|NotStarted{..} => unreachable!(),
        };
        let source_index = self.call_stack[parent_scope_index].index;
        let block_data = BlockData::new(block.clone(), source_index, expression, Some(parent_scope_index), self.variables.len());
        let source = self.compiler.source(source_index);
        let source2 = source.borrow();
        let parse_result = source2.parse_result();
        self.evaluate_block_data(block_data, SourceExpression { parse_result: &parse_result, expression })
    }

    fn evaluate_block_data(&mut self, block_data: BlockData, expression: SourceExpression) -> Value {
        let index = self.call_stack.push(block_data);
        self.call_stack[index].block.set_state(BlockState::Running(index));

        // Evaluate the expression
        let value = self.eager_evaluate(expression);

        let block = self.call_stack.pop().unwrap();
        self.variables.truncate(block.first_variable);
        block.block.set_state(BlockState::Complete(value.clone()));
        assert!(self.call_stack.len() == index);

        value
    }

    fn eager_evaluate(&mut self, expression: SourceExpression) -> Value {
        let value = self.evaluate(expression);
        match value {
            Value::Block(ref block) => {
                self.call_stack.last_mut().map(|block| block.expression = expression.expression);
                self.evaluate_block(block)
            },
            _ => value,
        }
    }

    fn evaluate(&mut self, expression: SourceExpression) -> Value {
        match *expression.token().token() {
            Open {boundary,error: ExpressionBoundaryError::None,..}|Close {boundary,error: ExpressionBoundaryError::None,..} => match boundary {
                Parentheses|PrecedenceGroup|CompoundTerm => self.eager_evaluate(expression.inner()),
                CurlyBraces|Source => self.top_block().block.create_child(expression.inner().expression).into(),
            },
            Open {error: ExpressionBoundaryError::CloseWithoutOpen,boundary,..}|Close {error: ExpressionBoundaryError::CloseWithoutOpen,boundary,..} => self.close_without_open_error(expression, boundary),
            Open {error: ExpressionBoundaryError::OpenWithoutClose,boundary,..}|Close {error: ExpressionBoundaryError::OpenWithoutClose,boundary,..} => self.open_without_close_error(expression, boundary),
            Open {error: ExpressionBoundaryError::OpenError,..}|Close {error: ExpressionBoundaryError::OpenError,..} => self.open_error(expression),
            IntegerLiteral(literal) => BigInt::from_str(&expression.parse_result.literals[literal]).unwrap().into(),
            VariableReference(name) => match self.get_variable_value(name) {
                Some(value) => value,
                None => self.no_such_field_error(expression),
            },
            NewlineSequence => self.evaluate_newline(expression),
            InfixAssignment(EMPTY_STRING) => self.evaluate_bare_assignment(expression),
            InfixAssignment(operator) => {
                let value = self.evaluate_infix(expression, operator);
                self.evaluate_assignment(expression, Left, value)
            },
            InfixOperator(operator) => self.evaluate_infix(expression, operator),
            PrefixOperator(PLUS_PLUS) => {
                let value = self.integer_unary(expression, PrefixOperand, |v| v+1);
                self.evaluate_assignment(expression, PrefixOperand, value)
            },
            PrefixOperator(DASH_DASH) => {
                let value = self.integer_unary(expression, PrefixOperand, |v| v-1);
                self.evaluate_assignment(expression, PrefixOperand, value)
            },
            PostfixOperator(PLUS_PLUS) => {
                let value = self.integer_unary(expression, PostfixOperand, |v| v+1);
                self.evaluate_assignment(expression, PostfixOperand, value)
            },
            PostfixOperator(DASH_DASH) => {
                let value = self.integer_unary(expression, PostfixOperand, |v| v-1);
                self.evaluate_assignment(expression, PostfixOperand, value)
            },
            PrefixOperator(operator) => self.evaluate_prefix(expression, operator),
            PostfixOperator(operator) => self.evaluate_postfix(expression, operator),
            ErrorTerm(CompileErrorCode::IdentifierStartsWithNumber) => IdentifierStartsWithNumber::value(expression.into()),
            ErrorTerm(CompileErrorCode::UnsupportedCharacters) => UnsupportedCharacters::value(expression.into()),
            ErrorTerm(CompileErrorCode::InvalidUtf8) => InvalidUtf8::value(expression.into()),
            ErrorTerm(_) => unreachable!(),
            MissingInfix => Value::Nothing.include_errors(self.evaluate(expression.left())).include_errors(self.evaluate(expression.right())),
            MissingExpression => Value::Nothing,
        }
    }

    fn get_variable_value(&self, name: IdentifierIndex) -> Option<Value> {
        let mut block = self.call_stack.len()-1;
        loop {
            let start = self.call_stack[block].first_variable;
            let end = match self.call_stack.get(block+1) {
                Some(block) => block.first_variable,
                None => self.variables.len(),
            };
            println!("Looking for {} in {}..{} (block {})", name, start, end, block);
            if let Some(variable) = self.variables[start..end].iter().find(|v| { println!("- {}", v.name); v.name == name }) {
                return Some(variable.value.clone());
            }
            if let Some(parent) = self.call_stack[block].parent_scope {
                block = parent;
            } else {
                break;
            }
        }
        println!("Looking for {} in {}..{} (root)", name, 0, self.root_variable_count);
        if let Some(variable) = self.variables[VariableIndex(0)..self.root_variable_count].iter().find(|v| v.name == name) {
            Some(variable.value.clone())
        } else {
            None
        }
    }

    fn set_variable_value(&mut self, name: IdentifierIndex, value: Value) {
        let mut block = self.call_stack.len()-1;
        loop {
            let start = self.call_stack[block].first_variable;
            let end = match self.call_stack.get(block+1) {
                Some(block) => block.first_variable,
                None => self.variables.len(),
            };
            if let Some(variable) = self.variables[start..end].iter_mut().find(|v| v.name == name) {
                println!("Set {} to {} in {}..{} (block {})", name, value.disp(self.compiler), start, end, block);
                variable.value = value;
                return;
            }
            if let Some(parent) = self.call_stack[block].parent_scope {
                block = parent;
            } else {
                break;
            }
        }
                println!("Push {} to {} at {} (block {})", name, value.disp(self.compiler), self.variables.len(), block);
        self.variables.push(Variable { name, value, is_field: false });
    }

    fn evaluate_infix(&mut self, expression: SourceExpression, operator: IdentifierIndex) -> Value {
        match operator {
            PLUS => self.rational_infix(expression, |a,b| a+b),
            DASH => self.rational_infix(expression, |a,b| a-b),
            SLASH => self.evaluate_divide(expression),
            STAR => self.rational_infix(expression, |a,b| a*b),
            GREATER_EQUAL => self.rational_infix(expression, |a,b| a>=b),
            GREATER_THAN => self.rational_infix(expression, |a,b| a>b),
            LESS_EQUAL => self.rational_infix(expression, |a,b| a<=b),
            LESS_THAN => self.rational_infix(expression, |a,b| a<b),
            AND_AND => self.evaluate_boolean_and(expression),
            OR_OR => self.evaluate_boolean_or(expression),
            EQUAL_TO => self.evaluate_equal_to(expression, ),
            NOT_EQUAL_TO => match self.evaluate_equal_to(expression) { Value::Boolean(value) => Value::Boolean(!value), value => value },
            SEMICOLON => self.evaluate_semicolon(expression),
            _ => self.unrecognized_operator_error(expression),
        }
    }

    fn evaluate_assignment(&mut self, expression: SourceExpression, position: OperandPosition, value: Value) -> Value {
        let target = expression.operand(position);
        match *target.token().token() {
            PrefixOperator(COLON) => {
                let operand = target.operand(PrefixOperand);
                match *operand.token().token() {
                    VariableReference(name) => {
                        let start = self.top_block().first_variable;
                        if let Some(variable) = self.variables[start..].iter_mut().find(|v| v.name == name) {
                            variable.is_field = true;
                            variable.value = value;
                            return Value::Nothing;
                        }
                        self.variables.push(Variable { name, value, is_field: true });
                        Value::Nothing
                    },
                    _ => self.invalid_target_error(expression, position).include_errors(value),
                }
            },
            VariableReference(name) => {
                self.set_variable_value(name, value);
                Value::Nothing
            },
            MissingExpression => Value::Nothing.include_errors(value),
            _ => self.invalid_target_error(expression, position).include_errors(value),
        }
    }

    fn evaluate_bare_assignment(&mut self, expression: SourceExpression) -> Value {
        let value = self.evaluate_operand(expression, Right);
        let result = self.evaluate_assignment(expression, Left, value);
        if let MissingExpression = *expression.left().token().token() {
            result.include_errors(self.missing_operand_error(expression, Left))
        } else {
            result
        }
    }

    fn evaluate_expose(&mut self, expression: SourceExpression) -> Value {
        let operand = expression.operand(PrefixOperand);
        match *operand.token().token() {
            VariableReference(name) => {
                let start = self.top_block().first_variable;
                if let Some(variable) = self.variables[start..].iter_mut().find(|v| v.name == name) {
                    variable.is_field = true;
                    return variable.value.clone();
                }
                self.variables.push(Variable { name, value: Value::Nothing, is_field: true });
                self.field_not_set_error(operand)
            }
            _ => self.unrecognized_operator_error(expression),
        }
    }

    fn evaluate_prefix(&mut self, expression: SourceExpression, operator: IdentifierIndex) -> Value {
        match operator {
            EXCLAMATION_POINT => self.evaluate_boolean_not(expression),
            DOUBLE_EXCLAMATION_POINT => self.evaluate_as_boolean(expression),
            PLUS => self.rational_unary(expression, PrefixOperand, |a| a),
            DASH => self.rational_unary(expression, PrefixOperand, |a| -a),
            COLON => self.evaluate_expose(expression),
            _ => self.unrecognized_operator_error(expression),
        }
    }

    fn evaluate_postfix(&mut self, expression: SourceExpression, operator: IdentifierIndex) -> Value {
        match operator {
            _ => self.unrecognized_operator_error(expression)
        }
    }

    fn evaluate_operand(&mut self, expression: SourceExpression, position: OperandPosition) -> Value {
        let operand = expression.operand(position);
        if *operand.token().token() == MissingExpression {
            return self.missing_operand_error(expression, position);
        }
        self.evaluate(operand)
    }

    fn eager_evaluate_operand(&mut self, expression: SourceExpression, position: OperandPosition, expected_type: OperandType) -> Value {
        let operand = expression.operand(position);
        if *operand.token().token() == MissingExpression {
            return self.missing_operand_error(expression, position);
        }
        let value = self.eager_evaluate(operand);
        match value {
            Value::Errors(_) => value,
            _ => {
                if expected_type.matches(&value) {
                    value
                } else {
                    self.bad_type_error(value, expression, position, expected_type)
                }
            },
        }
    }

    fn evaluate_semicolon(&mut self, expression: SourceExpression) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Any);
        match left {
            Value::Errors(_) => left,
            _ => {
                // (a; ;) is illegal. (a; ) is not.
                if left == Value::Nothing {
                    let prev = expression.prev();
                    if let MissingExpression = *prev.token().token() {
                        if let InfixOperator(SEMICOLON) = *prev.prev().token().token() {
                            return self.missing_operand_error(expression, Left);
                        }
                    }
                }
                let right_operand = expression.operand(Right);
                self.evaluate(right_operand).include_errors(left)
            }
        }
    }

    fn evaluate_newline(&mut self, expression: SourceExpression) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Any);
        match left {
            Value::Errors(_) => left,
            _ => {
                let right_operand = expression.operand(Right);
                self.evaluate(right_operand)
            }
        }
    }

    fn evaluate_divide(&mut self, expression: SourceExpression) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Number);
        let right = self.eager_evaluate_operand(expression, Right, OperandType::Number);
        match (left, right) {
            (Value::Rational(left), Value::Rational(right)) => {
                if right.is_zero() {
                    self.divide_by_zero_error(expression)
                } else {
                    Value::from(left/right)
                }
            },
            (left, right) => Value::Nothing.include_errors(left).include_errors(right),
        }
    }

    fn rational_infix<T: Into<Value>, F: FnOnce(BigRational,BigRational)->T>(&mut self, expression: SourceExpression, f: F) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Number);
        let right = self.eager_evaluate_operand(expression, Right, OperandType::Number);
        match (left, right) {
            (Value::Rational(left), Value::Rational(right)) => f(left, right).into(),
            (left, right) => Value::Nothing.include_errors(left).include_errors(right),
        }
    }

    fn rational_unary<T: Into<Value>, F: FnOnce(BigRational)->T>(&mut self, expression: SourceExpression, position: OperandPosition, f: F) -> Value {
        let operand = self.eager_evaluate_operand(expression, position, OperandType::Number);
        match operand {
            Value::Rational(operand) => f(operand).into(),
            _ => operand,
        }
    }

    fn integer_unary<T: Into<Value>, F: FnOnce(BigInt)->T>(&mut self, expression: SourceExpression, position: OperandPosition, f: F) -> Value {
        let operand = self.eager_evaluate_operand(expression, position, OperandType::Integer);
        match operand {
            Value::Rational(operand) => f(operand.to_integer()).into(),
            _ => operand,
        }
    }

    fn evaluate_equal_to(&mut self, expression: SourceExpression) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Any);
        let right = self.eager_evaluate_operand(expression, Right, OperandType::Any);
        match (left, right) {
            (Value::Rational(left), Value::Rational(right)) => Value::Boolean(left == right),
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left == right),
            (Value::Nothing, Value::Nothing) => Value::Boolean(true),
            (Value::Errors(left),right) => Value::Nothing.include_errors(left.into()).include_errors(right),
            (left,Value::Errors(right)) => Value::Nothing.include_errors(left).include_errors(right.into()),
            (Value::Block(_),_)|(_,Value::Block(_)) => unreachable!(),
            (_,_) => Value::Boolean(false),
        }
    }

    fn evaluate_boolean_and(&mut self, expression: SourceExpression) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Boolean);
        match left {
            Value::Boolean(false)|Value::Errors(_) => left,
            _ => {
                let right = self.eager_evaluate_operand(expression, Right, OperandType::Boolean);
                right.include_errors(left)
            }
        }
    }

    fn evaluate_boolean_or(&mut self, expression: SourceExpression) -> Value {
        let left = self.eager_evaluate_operand(expression, Left, OperandType::Boolean);
        match left {
            Value::Boolean(true)|Value::Errors(_) => left,
            _ => {
                let right = self.eager_evaluate_operand(expression, Right, OperandType::Boolean);
                right.include_errors(left)
            }
        }
    }

    fn evaluate_boolean_not(&mut self, expression: SourceExpression) -> Value {
        let result = self.eager_evaluate_operand(expression, PrefixOperand, OperandType::Boolean);
        match result {
            Value::Errors(_) => result,
            Value::Boolean(true) => Value::Boolean(false),
            Value::Boolean(false) => Value::Boolean(true),
            Value::Block(_)|Value::Rational(_)|Value::Nothing => unreachable!()
        }
    }

    fn evaluate_as_boolean(&mut self, expression: SourceExpression) -> Value {
        let result = self.eager_evaluate_operand(expression, PrefixOperand, OperandType::Boolean);
        match result {
            Value::Errors(_) => result,
            Value::Boolean(true) => Value::Boolean(true),
            Value::Boolean(false) => Value::Boolean(false),
            Value::Block(_)|Value::Rational(_)|Value::Nothing => unreachable!()
        }
    }

    fn top_block(&self) -> &BlockData {
        self.call_stack.last().unwrap()
    }

    fn circular_dependency_error(&self, _index: CallStackIndex) -> Value {
        CircularDependency::value(SourceRange::Expression { source: self.top_block().index, expression: self.top_block().expression })
    }
    fn no_such_field_error(&self, expression: SourceExpression) -> Value {
        NoSuchField::value(expression.into())
    }
    fn field_not_set_error(&self, expression: SourceExpression) -> Value {
        FieldNotSet::value(expression.into())
    }
    fn divide_by_zero_error(&self, expression: SourceExpression) -> Value {
        DivideByZero::value(expression.token().into())
    }
    fn unrecognized_operator_error(&self, expression: SourceExpression) -> Value {
        UnrecognizedOperator::value(expression.token().into(), expression.token().token().fixity())
    }
    fn missing_operand_error(&self, expression: SourceExpression, position: OperandPosition) -> Value {
        MissingOperand::value(expression.token().into(), position)
    }
    fn bad_type_error(&self, actual_value: Value, expression: SourceExpression, position: OperandPosition, expected_type: OperandType) -> Value {
        BadType::value(expression.token().into(), expression.operand(position).into(), actual_value, expected_type, position)
    }
    fn close_without_open_error(&self, expression: SourceExpression, boundary: ExpressionBoundary) -> Value {
        CloseWithoutOpen::value(expression.close_token().into(), boundary.open_string())
    }
    fn open_without_close_error(&self, expression: SourceExpression, boundary: ExpressionBoundary) -> Value {
        OpenWithoutClose::value(expression.open_token().into(), boundary.close_string())
    }
    fn open_error(&self, expression: SourceExpression) -> Value {
        expression.parse_result.open_error.clone().unwrap()
    }
    fn invalid_target_error(&self, expression: SourceExpression, position: OperandPosition) -> Value {
        let target = expression.operand(position);
        let operator = expression.token();
        match position {
            Left => LeftSideOfAssignmentMustBeIdentifier::value(operator.into(), target.into()),
            PrefixOperand => RightSideOfIncrementOrDecrementMustBeIdentifier::value(operator.into(), target.into()),
            PostfixOperand => LeftSideOfIncrementOrDecrementMustBeIdentifier::value(operator.into(), target.into()),
            Right => unreachable!(),
        }
    }
}
