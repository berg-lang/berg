extern crate berg_compiler;
pub use compiler_test::berg_compiler::*;

use std::ops::Range;

macro_rules! compiler_tests {
    ($($name:ident: $source:tt => $($rule:ident($($arg:tt)*))+,)+) => {
        use compiler_test::*;
        $(
            #[test]
            fn $name() {
                let source = compiler_tests!(@source $source);
                let mut test = CompilerTest::new(source.as_bytes());
                $( compiler_tests!(@rule test $rule $($arg)*); )+
            }
        )+
    };
    (@source [$($e:tt)*]) => { &[$($e)*] };
    (@source $source:tt) => { $source };
    (@rule $test:ident error $error:ident@$at:tt) => {
        compiler_tests!(@rule $test errors $error@$at)
    };
    (@rule $test:ident errors $($error:ident@$at:tt),+) => {
        $test.assert_errors(vec![ $(
            ($error, compiler_tests!(@at $at))
        ),+ ]);
    };
    (@rule $test:ident result nothing) => {
        $test.assert_result(PlatonicValue::Nothing);
    };
    (@rule $test:ident result $($result:tt)*) => {
        $test.assert_result($($result)*);
    };
    (@at [$loc:tt (zero width)]) => { $loc..$loc };
    (@at [$start:tt-$end:tt]) => { $start..$end+1 };
    (@at $loc:tt) => { $loc..$loc+1 };
}

pub struct CompilerTest<'t> {
//    source: &'t [u8],
    compiler: Compiler<'t>,
}
impl<'t> CompilerTest<'t> {
    pub fn new(source: &'t [u8]) -> CompilerTest<'t> {
        let out: Vec<u8> = vec![];
        let err: Vec<u8> = vec![];
        let mut compiler = Compiler::new(None, None, Box::new(out), Box::new(err));
        compiler.add_memory_source("[test expr]", source);
        CompilerTest { compiler }
    }

    pub fn assert_result<T: Into<PlatonicValue>>(&mut self, expected: T) {
        let expected = expected.into();
        self.compiler.with_sources(|sources| assert_eq!(sources.len(), 1));
        assert_eq!(expected, PlatonicRuntime::run(&self.compiler, SourceIndex(0)));
    }

    pub fn assert_errors<Err: Into<ExpectedCompileError>>(&mut self, mut expected: Vec<Err>) {
        let mut expected: Vec<ExpectedCompileError> = expected.drain(..).map(|error| error.into()).collect();
        self.compiler.with_errors(|actual| {
            let mut actual = actual.to_vec().clone();
            actual.retain(|actual_error| {
                let mut found = false;
                expected.retain(|expected_error| {
                    if !found && expected_error.matches(actual_error) {
                        found = true;
                        false
                    } else {
                        true
                    }
                });
                !found
            });
            assert!(expected.len() == 0, "Expected errors not produced!\n{:?}", expected);
            assert!(actual.len() == 0, "Unexpected compiler errors!\n{:?}", actual)
        })
    }
}
pub trait AsBytes {
    fn as_bytes<'t>(&'t self) -> &'t [u8];
}
impl AsBytes for str {
    fn as_bytes<'t>(&'t self) -> &'t [u8] { self.as_bytes() }
}
impl AsBytes for [u8] {
    fn as_bytes<'t>(&'t self) -> &'t [u8] { self }
}

#[derive(Debug)]
pub struct ExpectedCompileError {
    error_type: CompileErrorType,
    messages: Vec<ExpectedCompileErrorMessage>,
}
#[derive(Debug)]
pub struct ExpectedCompileErrorMessage {
    range: Option<Range<ByteIndex>>,
}

impl From<(CompileErrorType, Range<ByteIndex>)> for ExpectedCompileError {
    fn from((error_type, range): (CompileErrorType, Range<ByteIndex>)) -> ExpectedCompileError {
        ExpectedCompileError { error_type, messages: vec![ExpectedCompileErrorMessage { range: Some(range) }]}
    }
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
