use internals::*;

#[derive(Debug)]
pub struct CompileErrors {
    errors: Vec<CompileError>,
}

impl CompileErrors {
    pub fn new() -> CompileErrors {
        CompileErrors { errors: vec![] }
    }
    pub fn report(&mut self, error: CompileError) {
        self.errors.push(error);
    }
    pub fn all(&self) -> &Vec<CompileError> {
        &self.errors
    }
    pub fn len(&self) -> usize {
        self.errors.len()
    }
}
