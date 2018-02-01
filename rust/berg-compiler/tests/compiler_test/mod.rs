pub extern crate berg_compiler;
pub extern crate num;
pub use compiler_test::berg_compiler::*;
pub use compiler_test::berg_compiler::test::*;
pub use compiler_test::berg_compiler::value::ErrorCode::*;
pub use compiler_test::berg_compiler::value::Nothing;
pub use compiler_test::num::BigRational;
pub use std::str::FromStr;

macro_rules! compiler_tests {
    ( ) => {};
    ( $name:ident: &$source:tt => $($tail:tt)* ) => {
        compiler_tests! { @test_source $name: (&$source) => $($tail)* }
    };
    ( $name:ident: $source:tt => $($tail:tt)* ) => {
        compiler_tests! { @test_source $name: ($source) => $($tail)* }
    };
    ( @test_source $name:ident: ($source:expr) => error($error:ident@$($at:tt)*), $($tail:tt)* ) => {
        #[test]
        fn $name() {
            use compiler_test::berg_compiler::test::expect;
            expect($source).to_error($error, compiler_tests!(@at $($at)*));
        }

        compiler_tests! { $($tail)* }
    };
    ( @test_source $name:ident: ($source:expr) => value($value:expr), $($tail:tt)* ) => {
        #[test]
        fn $name() {
            use compiler_test::berg_compiler::test::expect;
            expect($source).to_yield($value);
        }

        compiler_tests! { $($tail)* }
    };
    (@at [$loc:tt (zero width)]) => ($loc..$loc);
    (@at [$start:tt-$end:tt]) => ($start..$end+1);
    (@at $loc:tt) => ($loc..$loc+1);
}
