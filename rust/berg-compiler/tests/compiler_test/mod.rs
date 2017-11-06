extern crate berg_compiler;
pub use compiler_test::berg_compiler::*;
pub use compiler_test::berg_compiler::compile_errors::*;

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
            ($error::CODE, compiler_tests!(@at $at))
        ),+ ]);
    };
    (@rule $test:ident type error) => {
        $test.assert_type(Type::Error);
    };
    (@rule $test:ident type nothing) => {
        $test.assert_type(Type::Nothing);
    };
    (@rule $test:ident type $($type:tt)*) => {
        $test.assert_type($($type)*);
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
        compiler.with_sources(|sources| assert_eq!(sources.len(), 1));
        CompilerTest { compiler }
    }

    pub fn assert_type<T: Into<Type>>(&mut self, expected: T) {
        let expected = expected.into();
        self.compiler.with_source(SourceIndex(0), |source| assert_eq!(expected, *source.checked_type()))
    }

    pub fn assert_errors<Err: Into<ExpectedCompileError>>(&mut self, mut expected: Vec<Err>) {
        let mut expected: Vec<ExpectedCompileError> = expected.drain(..).map(|error| error.into()).collect();
        let actual = self.compiler.errors.read().unwrap();
        let actual_count = actual.iter().filter_map(|actual_error| {
            let mut found = false;
            expected.retain(|expected_error| {
                if !found && expected_error.matches(&self.compiler, actual_error) {
                    found = true;
                    false
                } else {
                    true
                }
            });
            if found {
                None
            } else {
                Some(&actual)
            }
        }).count();
        assert!(actual_count == 0, "Unexpected compiler errors!\nExpected: {:?}\nActual: {:?}", expected, actual);
        assert!(expected.len() == 0, "Expected errors not produced!\nExpected: {:?}\nActual: {:?}", expected, actual);
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

#[derive(Debug,Clone)]
pub struct ExpectedCompileError {
    code: u32,
    range: ByteRange,
}
impl ExpectedCompileError {
    fn matches<'t>(&self, compiler: &Compiler, error: &Box<CompileError+'t>) -> bool {
        let message = error.message(compiler);
        match message.location {
            CompileErrorLocation::Generic|CompileErrorLocation::SourceOnly{..} => false,
            CompileErrorLocation::SourceRange{range,..} => self.code == error.code() && self.range == range
        }
    }
}
impl From<(u32, Range<usize>)> for ExpectedCompileError {
    fn from((code, range): (u32, Range<usize>)) -> ExpectedCompileError {
        let range = ByteIndex::from(range.start)..ByteIndex::from(range.end);
        ExpectedCompileError { code, range }
    }
}
