extern crate berg_compiler;
pub use compiler_test::berg_compiler::*;

use std::ops::Range;

macro_rules! compiler_tests {
    ($($name:ident: $source:tt => $rule:ident($($arg:tt)*),)+) => {
        use compiler_test::*;
        $(
            #[test]
            fn $name() {
                compiler_tests!(@rule $source $rule $($arg)*);
            }
        )+
    };
    (@rule $source:tt error $error:ident@$at:tt) => {
        compiler_tests!(@rule $source errors $error@$at)
    };
    (@rule $source:tt errors $($error:ident@$at:tt),+) => {
        expect_compile_errors($source, vec![ $(
            ($error, compiler_tests!(@at $at))
        ),+ ]);
    };
    (@at [$loc:tt (zero width)]) => { $loc..$loc };
    (@at [$start:tt-$end:tt]) => { $start..$end+1 };
    (@at $loc:tt) => { $loc..$loc+1 };
}

// macro_rules! compiler_tests {
//     // Single test
//     ($name:ident: { $($def:tt)+ }) => {
//         #[test]
//         pub fn $name() {
//             compiler_test_body!($($def)*)
//         }
//     };
//     // Test; Test
//     ($name:ident: { $($def:tt)+ }; $($e:tt)*) => {
//         compiler_tests! { $name: { $($def)+ } };
//         compiler_tests! { $($e)* }
//     };
//     // Build up a test
//     ($name:ident: { $($def:tt)+ } $next:tt $($e:tt)*) => {
//         compiler_tests! { $name: { $($def)+ $next } $($e)* }
//     };
//     // First token in a test
//     ($name:ident: $def:tt $($e:tt)*) => {
//         compiler_tests! { $name: { $def } $($e:tt)* }
//     };
//     // Handles empty test_compile!(), or terminating semicolon
//     () => {};
// }
// macro_rules! compiler_test_body {
//     (bytes $bytes:tt $($e:tt)*) => { compiler_test_builder!({ compiler_test::expect_compile($bytes) } $($e)*) };
//     (source $string:tt $($e:tt)*) => { compiler_test_builder!({ compiler_test::expect_compile(&$string.as_bytes()) } $($e)*) };
//     (berg[$($berg:tt)*] $($e:tt)*) => { compiler_test_builder!({ compiler_test::expect_compile(stringify!($($berg)*).as_bytes()) } $($e)*) };
//     (berg $berg:tt $($e:tt)*) => { compiler_test_builder!({ compiler_test::expect_compile(stringify!($berg).as_bytes()) } $($e)*) };
//     ($string:tt $($e:tt)*) => { compiler_test_builder!({ compiler_test::expect_compile(&$string.as_bytes()) } $($e)*) };
// }
// macro_rules! compiler_test_builder {
//     ($builder:tt) => { $builder.run() };
//     (@error $builder:tt) => { $builder.run() };
//     ($builder:tt has error $error_type:ident $($e:tt)*) => { compiler_test_builder!(@error { $builder.to_report(compiler_test::$error_type) } $($e:tt)*) };
//     (@error $builder:tt has error $error_type:ident $($e:tt)*) => { compiler_test_builder!(@error { $builder.and_report($error_type) } $($e:tt)*) };

//     (@error $builder:tt at $start:tt-$end:tt $($e:tt)*) => { compiler_test_builder!(@error { $builder.at($start,$end+1) } $($e)*) };
//     (@error $builder:tt at $start:tt (zero width) $($e:tt)*) => { compiler_test_builder!(@error { $builder.at($start,$start) } $($e)*) };
//     (@error $builder:tt at $start:tt $($e:tt)*) => { compiler_test_builder!(@error { $builder.at($start,$start+1) } $($e)*) };
// }

pub struct CompilerTest<'t> {
    source: &'t [u8],
    compiler: Option<Compiler<'t>>,
}
impl<'t> CompilerTest<'t> {
    fn ensure_compiler<'r>(&'r mut self) -> &'r Compiler<'t> {
        if self.compiler.is_none() {
            let out: Vec<u8> = vec![];
            let err: Vec<u8> = vec![];
            let mut compiler = Compiler::new(None, None, Box::new(out), Box::new(err));
            compiler.add_memory_source("[test expr]", self.source);
            self.compiler = Some(compiler);
        }
        if let Some(ref compiler) = self.compiler {
            compiler
        } else {
            unreachable!()
        }
    }

    fn assert_compile_errors(&mut self, mut expected_errors: Vec<ExpectedCompileError>) {
        self.ensure_compiler().with_errors(|actual_errors| {
            let mut actual_errors = actual_errors.to_vec();
            actual_errors.retain(|actual| {
                let mut found = false;
                expected_errors.retain(|expected| {
                    if !found && expected.matches(actual) {
                        found = true;
                        false
                    } else {
                        true
                    }
                });
                !found
            });
            assert!(expected_errors.len() == 0, "Expected errors not produced!\n{:?}", expected_errors);
            assert!(actual_errors.len() == 0, "Unexpected compiler errors!\n{:?}", actual_errors)
        })
    }
}
impl<'t> From<&'t str> for CompilerTest<'t> {
    fn from(source: &'t str) -> Self {
        CompilerTest { source: source.as_bytes(), compiler: None }
    }
}
impl<'t> From<&'t [u8]> for CompilerTest<'t> {
    fn from(source: &'t [u8]) -> Self {
        CompilerTest { source, compiler: None }
    }
}
impl<'t> From<&'t Vec<u8>> for CompilerTest<'t> {
    fn from(source: &'t Vec<u8>) -> Self {
        CompilerTest { source: source.as_slice(), compiler: None }
    }
}

pub fn expect_compile_errors<'t, Test: Into<CompilerTest<'t>>, Err: Into<ExpectedCompileError>>(test: Test, mut expected: Vec<Err>) {
    let mut test = test.into();
    let expected: Vec<ExpectedCompileError> = expected.drain(..).map(|error| error.into()).collect();
    test.assert_compile_errors(expected);
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
