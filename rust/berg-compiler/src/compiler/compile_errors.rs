use std::ops::Deref;
use public::*;
use std::io;
use std::ops::Range;
use std::path::Path;

#[derive(Debug)]
pub struct CompileErrors {
    errors: Vec<CompileError>,
}

impl Default for CompileErrors {
    fn default() -> Self { CompileErrors { errors: Default::default() } }
}

impl CompileErrors {
    pub(crate) fn report(&mut self, error: CompileError) {
        self.errors.push(error)
    }
    pub(crate) fn report_generic(&mut self, error_type: CompileErrorType) {
        self.report(error_type.generic())
    }
    pub(crate) fn extend(&mut self, errors: CompileErrors) {
        self.errors.extend(errors.errors)
    }
}

impl Deref for CompileErrors {
    type Target = Vec<CompileError>;
    fn deref(&self) -> &Vec<CompileError> { &self.errors }
}

impl Deref for SourceCompileErrors {
    type Target = Vec<CompileError>;
    fn deref(&self) -> &Vec<CompileError> { &self.errors }
}

pub(crate) struct SourceCompileErrors {
    errors: CompileErrors,
    source: SourceIndex,
}

impl SourceCompileErrors {
    pub(crate) fn new(source: SourceIndex) -> Self {
        let errors = Default::default();
        SourceCompileErrors { errors, source }
    }
    pub(crate) fn report(&mut self, error: CompileError) {
        self.errors.report(error)
    }
    pub(crate) fn report_at(&mut self, error_type: CompileErrorType, range: Range<ByteIndex>, string: &str) {
        let error = error_type.at(self.source, range, string);
        self.report(error)
    }
    pub(crate) fn report_invalid_utf8(&mut self, range: Range<ByteIndex>, bytes: &[u8]) {
        let error = CompileErrorType::InvalidUtf8.invalid_bytes(self.source, range, bytes);
        self.report(error)
    }
    pub(crate) fn report_io_read(&mut self, index: ByteIndex, error: &io::Error) {
        let error = CompileErrorType::IoReadError.io_read(self.source, index, error);
        self.report(error)
    }
    pub(crate) fn report_io_open(&mut self, error_type: CompileErrorType, error: &io::Error, path: &Path) {
        let error = error_type.io_open(self.source, error, path);
        self.report(error)
    }
    pub(crate) fn report_io_source(&mut self, error_type: CompileErrorType, error: &io::Error) {
        let error = error_type.io_source(self.source, error);
        self.report(error)
    }
    pub(crate) fn close(self) -> CompileErrors {
        self.errors
    }
}
