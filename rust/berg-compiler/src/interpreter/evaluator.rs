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
use ast::VariableIndex;
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
use ast;

index_type! {
    pub struct StackFrameIndex(pub u32) <= u32::MAX;
    pub struct StackIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub(crate) struct ExpressionEvaluator<'e> {
    call_stack: IndexedVec<StackFrame,StackFrameIndex>,
    stack: IndexedVec<VariableValue,StackIndex>,
    compiler: &'e Compiler,
}

#[derive(Debug)]
pub(super) struct StackFrame {
    block: Block,
    index: SourceIndex,
    expression: Expression,
    parent_scope: Option<StackFrameIndex>,
    first_variable: StackIndex,
}

#[derive(Debug)]
struct VariableValue {
    variable: VariableIndex,
    value: Value,
    is_field: bool,
}

// Represents process data--either it's not run, or it's running, or it's
// finished and has produced a value.
#[derive(Debug)]
pub(crate) enum BlockState {
    SourceNotStarted(SourceIndex),
    NotStarted { parent_scope: Block, expression: Expression },
    Running(StackFrameIndex),
    Complete(Value),
}

impl StackFrame {
    fn new(block: Block, index: SourceIndex, expression: Expression, parent_scope: Option<StackFrameIndex>, first_variable: StackIndex) -> Self {
        StackFrame {
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

impl<'e> ExpressionEvaluator<'e> {
    pub(crate) fn new(compiler: &'e Compiler) -> Self {
        ExpressionEvaluator {
            compiler,
            call_stack: Default::default(),
            stack: Default::default(),
        }
    }

    pub(super) fn evaluate_block(&mut self, block: &Block) -> Value {
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
        // First, parse the source.
        let source_rc = self.compiler.source(index);
        let source = source_rc.borrow();
        let parse_result = source.parse(self.compiler);
        let expression = SourceExpression::from_source(&parse_result);

        // Next, create the source frame and push root variables like true and false.
        let frame = StackFrame::new(source.value.clone(), parse_result.index, expression.expression, None, self.stack.next_index());
        let index = self.push_frame(frame);
        for (variable,&(_,ref value)) in ast::root_variables().iter().enumerate() {
            self.stack.push(VariableValue { variable, value: value.clone(), is_field: false });
        }

        // Finally, evaluate the source expression. 
        let value = self.eager_evaluate(expression);

        self.complete_frame(value.clone(), index);
        value
    }

    fn evaluate_child_block(&mut self, block: &Block, parent_scope: &Block, expression: Expression) -> Value {
        // Create and push the new StackFrame
        let parent_scope_index = match *parent_scope.state() {
            Running(index) => index,
            Complete(..) => panic!("Passing a block as a result of a block is not presently supported!"),
            SourceNotStarted(..)|NotStarted{..} => unreachable!(),
        };
        let source_index = self.call_stack[parent_scope_index].index;
        let frame = StackFrame::new(block.clone(), source_index, expression, Some(parent_scope_index), self.stack.next_index());
        let index = self.push_frame(frame);

        // Evaluate the expression.
        let source = self.compiler.source(source_index);
        let source2 = source.borrow();
        let parse_result = source2.parse_result();
        let value = self.eager_evaluate(SourceExpression { parse_result: &parse_result, expression });

        self.complete_frame(value.clone(), index);
        value
    }

    fn push_frame(&mut self, frame: StackFrame) -> StackFrameIndex {
        let index = self.call_stack.push(frame);
        self.call_stack[index].block.set_state(BlockState::Running(index));
        index
    }

    fn complete_frame(&mut self, complete_value: Value, index: StackFrameIndex) {
        let frame = self.call_stack.pop().unwrap();
        self.stack.truncate(frame.first_variable);
        frame.block.set_state(BlockState::Complete(complete_value));
        assert!(self.call_stack.next_index() == index);
    }

    fn eager_evaluate(&mut self, expression: SourceExpression) -> Value {
        let value = self.evaluate(expression);
        match value {
            Value::Block(ref block) => {
                self.call_stack.last_mut().map(|frame| frame.expression = expression.expression);
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
            VariableReference(variable) => match self.get_variable_value(variable) {
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
            MissingInfix => Value::Nothing.include_errors(self.evaluate(expression.left())).include_errors(self.evaluate(expression.right())),
            MissingExpression => Value::Nothing,
            RawIdentifier(_)|ErrorTerm(_) => unreachable!(),
        }
    }

    fn get_variable_value(&self, variable: VariableIndex) -> Option<Value> {
        let mut block = self.call_stack.last_index();
        loop {
            let start = self.call_stack[block].first_variable;
            let end = match self.call_stack.get(block+1) {
                Some(block) => block.first_variable,
                None => self.stack.next_index(),
            };
            println!("block({}).get({}): {}..{} - {:?}", block, variable, start, end, self.call_stack[block]);
            if let Some(variable) = self.stack[start..end].iter().find(|v| { println!("- {:?}", v); v.variable == variable }) {
                return Some(variable.value.clone());
            }
            if let Some(parent) = self.call_stack[block].parent_scope {
                block = parent;
            } else {
                break;
            }
        }
        None
    }

    fn set_variable_value(&mut self, variable: VariableIndex, value: Value) {
        let mut block = self.call_stack.last_index();
        loop {
            let start = self.call_stack[block].first_variable;
            let mut slice = match self.call_stack.get(block+1) {
                Some(block) => self.stack[start..block.first_variable].iter_mut(),
                None => self.stack[start..].iter_mut(),
            };
            if let Some(variable) = slice.find(|v| v.variable == variable) {
                variable.value = value;
                return;
            }
            if let Some(parent) = self.call_stack[block].parent_scope {
                block = parent;
            } else {
                break;
            }
        }
        self.stack.push(VariableValue { variable, value, is_field: false });
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
                    VariableReference(variable) => {
                        // :a = 1 means set a in the *current* block only.
                        let start = self.top_block().first_variable;
                        if let Some(variable) = self.stack[start..].iter_mut().find(|v| v.variable == variable) {
                            variable.is_field = true;
                            variable.value = value;
                            return Value::Nothing;
                        }
                        self.stack.push(VariableValue { variable, value, is_field: true });
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
            VariableReference(variable) => {
                // Expose the variable in the current block only
                let start = self.top_block().first_variable;
                if let Some(variable) = self.stack[start..].iter_mut().find(|v| v.variable == variable) {
                    variable.is_field = true;
                    return variable.value.clone();
                }
                let value = self.field_not_set_error(operand);
                self.stack.push(VariableValue { variable, value: value.clone(), is_field: true });
                value
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

    fn top_block(&self) -> &StackFrame {
        self.call_stack.last().unwrap()
    }

    fn circular_dependency_error(&self, _index: StackFrameIndex) -> Value {
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
