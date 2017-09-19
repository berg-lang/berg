use internals::*;

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
}
