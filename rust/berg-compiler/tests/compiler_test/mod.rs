pub extern crate berg_compiler;
pub use compiler_test::berg_compiler::*;
pub use compiler_test::berg_compiler::compile_errors::CompileErrorCode::*;


macro_rules! compiler_tests {
    ($($name:ident: $source:tt => $($rule:ident($($arg:tt)*))+,)+) => {
        use compiler_test::berg_compiler::test::expect;
        $(
            #[test]
            fn $name() {
                println!("\nSOURCE\n======\n{:?}\n======\n", $source);
                let source = compiler_tests!(@source $source);
                let mut test = expect(source);
                $( compiler_tests!(@rule test $rule $($arg)*); )+
                test.run();
            }
        )+
    };
    (@source ($($e:tt)*)) => { $($e)* };
    (@source $source:tt) => { $source };
    (@rule $test:ident error $error:ident@$at:tt) => {
        $test = $test.and_error(compiler_test::berg_compiler::compile_errors::CompileErrorCode::$error, compiler_tests!(@at $at));
    };
    (@rule $test:ident errors $($error:ident@$at:tt),+) => {
        $(compiler_tests!(@rule $test error $error@$at));+
    };
    (@rule $test:ident warning $error:ident@$at:tt) => {
        $test = $test.and_warn(compiler_test::berg_compiler::compile_errors::CompileErrorCode::$error, compiler_tests!(@at $at));
    };
    (@rule $test:ident warnings $($error:ident@$at:tt),+) => {
        $(compiler_tests!(@rule $test warning $error@$at));+
    };
    (@rule $test:ident value error) => {
        $test = $test.to_yield(compiler_test::berg_compiler::Value::Nothing);
    };
    (@rule $test:ident value nothing) => {
        $test = $test.to_yield(compiler_test::berg_compiler::Value::Nothing);
    };
    (@rule $test:ident value undefined) => {
        $test = $test.to_yield(compiler_test::berg_compiler::Value::Nothing);
    };
    (@rule $test:ident value $($value:tt)*) => {
        $test = $test.to_yield($($value)*);
    };
    (@at [$loc:tt (zero width)]) => { $loc..$loc };
    (@at [$start:tt-$end:tt]) => { $start..$end+1 };
    (@at $loc:tt) => { $loc };
}

