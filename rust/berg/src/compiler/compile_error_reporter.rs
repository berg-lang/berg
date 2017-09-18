use compiler::internals::*;
use std::cell::RefCell;

pub struct CompileErrorReporter {
    errors: RefCell<Vec<CompileError>>,
}

impl CompileErrorReporter {
    pub fn new() -> CompileErrorReporter {
        CompileErrorReporter { errors: RefCell::new(vec![]) }
    }
    // We use borrow_mut() so that multiple classes can reference the CompileErrors
    pub fn report(&self, error: CompileError) {
        let mut errors = self.errors.borrow_mut();
        errors.push(error);
    }
}
