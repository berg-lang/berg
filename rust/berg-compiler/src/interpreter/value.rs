use std::cell::Ref;
use util::display_context::DisplayContext;
use compiler::Compiler;
use interpreter::evaluator::BlockState;
use std::rc::Rc;
use std::cell::RefCell;
use num::BigInt;
use num::BigRational;
use source::compile_errors::CompileError;
use ast::expression::Expression;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Block(Block),
    Boolean(bool),
    Errors(Errors),
    Rational(BigRational),
    Nothing,
}

impl<'c> DisplayContext<&'c Compiler> for Value {
    fn fmt(&self, f: &mut Formatter, compiler: &&'c Compiler) -> fmt::Result {
        match *self {
            Value::Block(_) => write!(f, "{}", "{...}"),
            Value::Boolean(ref value) => write!(f, "{}", value),
            Value::Errors(ref errors) => write!(f, "{}", errors.disp(compiler)),
            Value::Rational(ref value) => write!(f, "{}", value),
            Value::Nothing => write!(f, "nothing"),
        }
    }
}

#[derive(Debug,Clone)]
pub struct Block(pub(crate) Rc<RefCell<BlockState>>);
impl From<Block> for Value { fn from(from: Block) -> Value { Value::Block(from) } }

impl PartialEq<Block> for Block {
    fn eq(&self, other: &Block) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Block {
    pub(crate) fn new(state: BlockState) -> Self {
        Block(Rc::new(RefCell::new(state)))
    }
    pub(crate) fn create_child(&self, expression: Expression) -> Self {
        Self::new(BlockState::NotStarted { parent_scope: self.clone(), expression })
    }
    pub(crate) fn state(&self) -> Ref<BlockState> {
        self.0.borrow()
    }
    pub(crate) fn set_state(&self, state: BlockState) {
        *self.0.borrow_mut() = state;
    }
}

impl From<bool> for Value { fn from(from: bool) -> Value { Value::Boolean(from) } }
impl From<BigRational> for Value { fn from(from: BigRational) -> Value { Value::Rational(from) } }
impl From<i64> for Value { fn from(value: i64) -> Value { BigInt::from(value).into() } }
impl From<BigInt> for Value { fn from(value: BigInt) -> Value { BigRational::from(value).into() } }

impl From<Box<CompileError>> for Value {
    fn from(error: Box<CompileError>) -> Value {
        Errors { errors: vec![ error ], value: Box::new(Value::Nothing) }.into()
    }
}
impl<E: CompileError+'static> From<E> for Value {
    fn from(error: E) -> Value {
        Errors { errors: vec![ Box::new(error) ], value: Box::new(Value::Nothing) }.into()
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Errors {
    pub errors: Vec<Box<CompileError>>,
    pub value: Box<Value>,
}

impl<'c> DisplayContext<&'c Compiler> for Errors {
    fn fmt(&self, f: &mut Formatter, compiler: &&'c Compiler) -> fmt::Result {
        write!(f, "Errors (value={}):\n", self.value.disp(compiler))?;
        for error in &self.errors {
            write!(f, "- {}", error.disp(compiler))?
        }
        Ok(())
    }
}

impl From<Errors> for Value { fn from(from: Errors) -> Value { Value::Errors(from) } }

impl Value {
    pub(crate) fn include_errors(self, from: Value) -> Value {
        // If both have errors, transfer the value and errors from the right into the left, and return that.
        if let Value::Errors(Errors { errors: from_errors, .. }) = from {
            if let Value::Errors(Errors { mut errors, value }) = self {
                Value::Errors(Errors {
                    errors: { errors.extend(from_errors); errors },
                    value: value
                })
            } else {
                Value::Errors(Errors {
                    errors: from_errors,
                    value: Box::new(self)
                })
            }
        } else {
            self
        }
    }
}
