extern crate berg_compiler;

use berg_compiler::*;

mod compile_tester {
    use berg_compiler::*;
    use std::ops::Range;

    pub fn expect_compile<'t>(source: &'t AsRef<[u8]>) -> CompilerTest<'t> {
        CompilerTest::new(source.as_ref())
    }
    pub struct CompilerTest<'t> {
        source: &'t [u8],
        errors: Vec<ExpectedCompileError>,
    }
    #[derive(Debug)]
    struct ExpectedCompileError {
        error_type: CompileErrorType,
        messages: Vec<ExpectedCompileErrorMessage>,
    }
    #[derive(Debug)]
    struct ExpectedCompileErrorMessage {
        range: Option<Range<ByteIndex>>,
    }

    impl ExpectedCompileError {
        fn matches(&self, actual: &CompileError) -> bool {
            if self.error_type != actual.error_type() || self.messages.len() != actual.messages().len() {
                return false;
            }
            self.messages.iter().all(|expected| actual.messages().iter().any(|actual| expected.matches(actual)))
        }
    }
    impl ExpectedCompileErrorMessage {
        fn matches(&self, actual: &CompileErrorMessage) -> bool {
            self.range == *actual.range()
        }
    }

    impl<'t> CompilerTest<'t> {
        fn new(source: &'t [u8]) -> Self {
            CompilerTest { source, errors: vec![] }
        }
        pub fn run(mut self) {
            let out: Vec<u8> = vec![];
            let err: Vec<u8> = vec![];
            let mut compiler = Compiler::new(None, None, Box::new(out), Box::new(err));

            compiler.add_memory_source("[test string]", self.source);

            compiler.with_errors(|actual_errors| {
                let mut actual_errors = actual_errors.to_vec();
                actual_errors.retain(|actual| {
                    let mut found = false;
                    self.errors.retain(|expected| {
                        if !found && expected.matches(actual) {
                            found = true;
                            false
                        } else {
                            true
                        }
                    });
                    !found
                });
                assert!(self.errors.len() == 0, "Expected errors not produced!\n{:?}", self.errors);
                assert!(actual_errors.len() == 0, "Unexpected compiler errors!\n{:?}", actual_errors)
            })
        }
        pub fn to_report(self, error_type: CompileErrorType) -> ExpectedCompileErrorMessageBuilder<'t> {
            ExpectedCompileErrorMessageBuilder::new(self, error_type)
        }
    }

    pub struct ExpectedCompileErrorMessageBuilder<'t> {
        test: CompilerTest<'t>,
        error: ExpectedCompileError,
        message: ExpectedCompileErrorMessage,
    }
    impl<'t> ExpectedCompileErrorMessageBuilder<'t> {
        pub fn new(test: CompilerTest<'t>, error_type: CompileErrorType) -> Self {
            let error = ExpectedCompileError { error_type, messages: vec![] };
            let message = ExpectedCompileErrorMessage { range: None };
            ExpectedCompileErrorMessageBuilder { test, error, message }
        }
        pub fn run(self) {
            self.complete().run()
        }
        // pub fn and_report(self, error_type: CompileErrorType) -> ExpectedCompileErrorMessageBuilder<'t> {
        //     self.complete().to_report(error_type)
        // }
        pub fn at(mut self, range: Range<ByteIndex>) -> Self {
            self.message.range = Some(range);
            self
        }

        fn complete(mut self) -> CompilerTest<'t> {
            self.error.messages.push(self.message);
            self.test.errors.push(self.error);
            self.test
        }
    }
}

use compile_tester::*;

#[test]
fn unsupported_character() {
    expect_compile(&"`")
        .to_report(UnsupportedCharacters).at(0..1)
        .run();
}
