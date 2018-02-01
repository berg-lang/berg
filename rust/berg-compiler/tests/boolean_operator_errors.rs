#![recursion_limit="512"]
#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    not_equal: "!1==1"             => error(UnsupportedOperator@0),
    not_ne: "!1!=1"                => error(UnsupportedOperator@0),
    not_greater: "!1>1"            => error(UnsupportedOperator@0),
    not_less: "!1<1"               => error(UnsupportedOperator@0),
    not_ge: "!1>=1"                => error(UnsupportedOperator@0),
    not_le: "!1<=1"                => error(UnsupportedOperator@0),

    // && errors
    and_true_1: "true&&1"   => error(BadType@6),
    and_false_1: "false&&1" => value(false),
    and_1_true: "1&&true"   => error(UnsupportedOperator@[1-2]),
    and_1_false: "1&&false" => error(UnsupportedOperator@[1-2]),
    and_true_error: "true&&(1/0)" => error(DivideByZero@9),
    and_false_error: "false&&(1/0)" => value(false),
    and_error_true: "(1/0)&&true" => error(DivideByZero@3),
    and_error_false: "(1/0)&&false" => error(DivideByZero@3),
    and_true_nothing: "true&&()"   => error(BadType@[6-7]),
    and_false_nothing: "false&&()" => value(false),
    and_nothing_true: "()&&true"   => error(UnsupportedOperator@[2-3]),
    and_nothing_false: "()&&false" => error(UnsupportedOperator@[2-3]),

    and_1_1: "1&&1"         => error(UnsupportedOperator@[1-2]),
    and_1_nothing: "1&&()"  => error(UnsupportedOperator@[1-2]),
    and_nothing_1: "()&&1"  => error(UnsupportedOperator@[2-3]),
    and_1_error: "1&&(1/0)"  => error(UnsupportedOperator@[1-2]),
    and_error_1: "(1/0)&&1"  => error(DivideByZero@3),
    and_nothing_nothing: "()&&()"   => error(UnsupportedOperator@[2-3]),
    and_nothing_error: "()&&(1/0)"  => error(UnsupportedOperator@[2-3]),
    and_error_nothing: "(1/0)&&()"  => error(DivideByZero@3),
    and_error_error: "(1/0)&&(1/0)"   => error(DivideByZero@3),

    // || errors
    or_true_1: "true||1"   => value(true),
    or_false_1: "false||1" => error(BadType@7),
    or_1_true: "1||true"   => error(UnsupportedOperator@[1-2]),
    or_1_false: "1||false" => error(UnsupportedOperator@[1-2]),
    or_true_error: "true||(1/0)" => value(true),
    or_false_error: "false||(1/0)" => error(DivideByZero@10),
    or_error_true: "(1/0)||true" => error(DivideByZero@3),
    or_error_false: "(1/0)||false" => error(DivideByZero@3),
    or_true_nothing: "true||()"   => value(true),
    or_false_nothing: "false||()" => error(BadType@[7-8]),
    or_nothing_true: "()||true"   => error(UnsupportedOperator@[2-3]),
    or_nothing_false: "()||false" => error(UnsupportedOperator@[2-3]),

    or_1_1: "1||1"         => error(UnsupportedOperator@[1-2]),
    or_1_nothing: "1||()"  => error(UnsupportedOperator@[1-2]),
    or_nothing_1: "()||1"  => error(UnsupportedOperator@[2-3]),
    or_1_error: "1||(1/0)"  => error(UnsupportedOperator@[1-2]),
    or_error_1: "(1/0)||1"  => error(DivideByZero@3),
    or_nothing_nothing: "()||()"   => error(UnsupportedOperator@[2-3]),
    or_nothing_error: "()||(1/0)"  => error(UnsupportedOperator@[2-3]),
    or_error_nothing: "(1/0)||()"  => error(DivideByZero@3),
    or_error_error: "(1/0)||(1/0)"   => error(DivideByZero@3),

    // ! errors
    not_1: "!1"   => error(UnsupportedOperator@0),
    not_nothing: "!()" => error(UnsupportedOperator@0),
    not_error: "!(1/0)" => error(DivideByZero@4),
}
