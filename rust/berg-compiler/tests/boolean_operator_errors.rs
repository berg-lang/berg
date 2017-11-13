#[macro_use]
pub mod compiler_test;

compiler_tests! {
    not_equal: "!1==1"             => error(BadTypeRightOperand@0) type(error),
    not_ne: "!1!=1"                => error(BadTypeRightOperand@0) type(error),
    not_greater: "!1>1"            => error(BadTypeRightOperand@0) type(error),
    not_less: "!1<1"               => error(BadTypeRightOperand@0) type(error),
    not_ge: "!1>=1"                => error(BadTypeRightOperand@0) type(error),
    not_le: "!1<=1"                => error(BadTypeRightOperand@0) type(error),

    // && errors
    and_true_1: "true&&1"   => errors(BadTypeRightOperand@[4-5]) type(error),
    and_false_1: "false&&1" => errors(BadTypeRightOperand@[5-6]) type(error),
    and_1_true: "1&&true"   => errors(BadTypeLeftOperand@[1-2]) type(error),
    and_1_false: "1&&false" => errors(BadTypeLeftOperand@[1-2]) type(error),
    and_true_error: "true&&(1/0)" => errors(DivideByZero@8) type(error),
    and_false_error: "false&&(1/0)" => errors(DivideByZero@9) type(error),
    and_error_true: "(1/0)&&true" => errors(DivideByZero@2) type(error),
    and_error_false: "(1/0)&&false" => errors(DivideByZero@2) type(error),
    and_true_nothing: "true&&()"   => errors(BadTypeRightOperand@[4-5]) type(error),
    and_false_nothing: "false&&()" => errors(BadTypeRightOperand@[5-6]) type(error),
    and_nothing_true: "()&&true"   => errors(BadTypeLeftOperand@[2-3]) type(error),
    and_nothing_false: "()&&false" => errors(BadTypeLeftOperand@[2-3]) type(error),

    and_1_1: "1&&1"         => errors(BadTypeLeftOperand@[1-2],BadTypeRightOperand@[1-2]) type(error),
    and_1_nothing: "1&&()"  => errors(BadTypeLeftOperand@[1-2],BadTypeRightOperand@[1-2]) type(error),
    and_nothing_1: "()&&1"  => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    and_1_error: "1&&(1/0)"  => errors(BadTypeLeftOperand@[1-2],DivideByZero@5) type(error),
    and_error_1: "(1/0)&&1"  => errors(DivideByZero@2,BadTypeRightOperand@[5-6]) type(error),
    and_nothing_nothing: "()&&()"   => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    and_nothing_error: "()&&(1/0)"  => errors(BadTypeLeftOperand@[2-3],DivideByZero@6) type(error),
    and_error_nothing: "(1/0)&&()"  => errors(DivideByZero@2,BadTypeRightOperand@[5-6]) type(error),
    and_error_error: "(1/0)&&(1/0)"   => errors(DivideByZero@2,DivideByZero@9) type(error),

    // || errors
    or_true_1: "true||1"   => errors(BadTypeRightOperand@[4-5]) type(error),
    or_false_1: "false||1" => errors(BadTypeRightOperand@[5-6]) type(error),
    or_1_true: "1||true"   => errors(BadTypeLeftOperand@[1-2]) type(error),
    or_1_false: "1||false" => errors(BadTypeLeftOperand@[1-2]) type(error),
    or_true_error: "true||(1/0)" => errors(DivideByZero@8) type(error),
    or_false_error: "false||(1/0)" => errors(DivideByZero@9) type(error),
    or_error_true: "(1/0)||true" => errors(DivideByZero@2) type(error),
    or_error_false: "(1/0)||false" => errors(DivideByZero@2) type(error),
    or_true_nothing: "true||()"   => errors(BadTypeRightOperand@[4-5]) type(error),
    or_false_nothing: "false||()" => errors(BadTypeRightOperand@[5-6]) type(error),
    or_nothing_true: "()||true"   => errors(BadTypeLeftOperand@[2-3]) type(error),
    or_nothing_false: "()||false" => errors(BadTypeLeftOperand@[2-3]) type(error),

    or_1_1: "1||1"         => errors(BadTypeLeftOperand@[1-2],BadTypeRightOperand@[1-2]) type(error),
    or_1_nothing: "1||()"  => errors(BadTypeLeftOperand@[1-2],BadTypeRightOperand@[1-2]) type(error),
    or_nothing_1: "()||1"  => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    or_1_error: "1||(1/0)"  => errors(BadTypeLeftOperand@[1-2],DivideByZero@5) type(error),
    or_error_1: "(1/0)||1"  => errors(DivideByZero@2,BadTypeRightOperand@[5-6]) type(error),
    or_nothing_nothing: "()||()"   => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    or_nothing_error: "()||(1/0)"  => errors(BadTypeLeftOperand@[2-3],DivideByZero@6) type(error),
    or_error_nothing: "(1/0)||()"  => errors(DivideByZero@2,BadTypeRightOperand@[5-6]) type(error),
    or_error_error: "(1/0)||(1/0)"   => errors(DivideByZero@2,DivideByZero@9) type(error),

    // ! errors
    not_1: "!1"   => errors(BadTypeRightOperand@0) type(error),
    not_nothing: "!()" => errors(BadTypeRightOperand@0) type(error),
    not_error: "!(1/0)" => errors(DivideByZero@3) type(error),
}
