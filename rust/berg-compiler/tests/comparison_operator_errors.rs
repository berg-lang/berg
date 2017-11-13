#[macro_use]
pub mod compiler_test;

compiler_tests! {
    // number/boolean
    greater_than_0_false: "0>false" => error(BadTypeRightOperand@1) type(error),
    greater_than_false_0: "false>0" => error(BadTypeLeftOperand@5) type(error),
    greater_than_1_false: "1>false" => error(BadTypeRightOperand@1) type(error),
    greater_than_false_1: "false>1" => error(BadTypeLeftOperand@5) type(error),
    greater_than_0_true: "0>true" => error(BadTypeRightOperand@1) type(error),
    greater_than_true_0: "true>0" => error(BadTypeLeftOperand@4) type(error),
    greater_than_1_true: "1>true" => error(BadTypeRightOperand@1) type(error),
    greater_than_true_1: "true>1" => error(BadTypeLeftOperand@4) type(error),

    greater_or_equal_0_false: "0>=false" => error(BadTypeRightOperand@[1-2]) type(error),
    greater_or_equal_false_0: "false>=0" => error(BadTypeLeftOperand@[5-6]) type(error),
    greater_or_equal_1_false: "1>=false" => error(BadTypeRightOperand@[1-2]) type(error),
    greater_or_equal_false_1: "false>=1" => error(BadTypeLeftOperand@[5-6]) type(error),
    greater_or_equal_0_true: "0>=true" => error(BadTypeRightOperand@[1-2]) type(error),
    greater_or_equal_true_0: "true>=0" => error(BadTypeLeftOperand@[4-5]) type(error),
    greater_or_equal_1_true: "1>=true" => error(BadTypeRightOperand@[1-2]) type(error),
    greater_or_equal_true_1: "true>=1" => error(BadTypeLeftOperand@[4-5]) type(error),

    less_than_0_false: "0<false" => error(BadTypeRightOperand@1) type(error),
    less_than_false_0: "false<0" => error(BadTypeLeftOperand@5) type(error),
    less_than_1_false: "1<false" => error(BadTypeRightOperand@1) type(error),
    less_than_false_1: "false<1" => error(BadTypeLeftOperand@5) type(error),
    less_than_0_true: "0<true" => error(BadTypeRightOperand@1) type(error),
    less_than_true_0: "true<0" => error(BadTypeLeftOperand@4) type(error),
    less_than_1_true: "1<true" => error(BadTypeRightOperand@1) type(error),
    less_than_true_1: "true<1" => error(BadTypeLeftOperand@4) type(error),

    less_or_equal_0_false: "0<=false" => error(BadTypeRightOperand@[1-2]) type(error),
    less_or_equal_false_0: "false<=0" => error(BadTypeLeftOperand@[5-6]) type(error),
    less_or_equal_1_false: "1<=false" => error(BadTypeRightOperand@[1-2]) type(error),
    less_or_equal_false_1: "false<=1" => error(BadTypeLeftOperand@[5-6]) type(error),
    less_or_equal_0_true: "0<=true" => error(BadTypeRightOperand@[1-2]) type(error),
    less_or_equal_true_0: "true<=0" => error(BadTypeLeftOperand@[4-5]) type(error),
    less_or_equal_1_true: "1<=true" => error(BadTypeRightOperand@[1-2]) type(error),
    less_or_equal_true_1: "true<=1" => error(BadTypeLeftOperand@[4-5]) type(error),

    // number/nothing
    greater_than_0_nothing: "0>()" => errors(BadTypeRightOperand@1) type(error),
    greater_than_nothing_0: "()>0" => errors(BadTypeLeftOperand@2) type(error),
    greater_or_equal_0_nothing: "0>=()" => errors(BadTypeRightOperand@[1-2]) type(error),
    greater_or_equal_nothing_0: "()>=0" => errors(BadTypeLeftOperand@[2-3]) type(error),
    less_than_0_nothing: "0<()" => errors(BadTypeRightOperand@1) type(error),
    less_than_nothing_0: "()<0" => errors(BadTypeLeftOperand@2) type(error),
    less_or_equal_0_nothing: "0<=()" => errors(BadTypeRightOperand@[1-2]) type(error),
    less_or_equal_nothing_0: "()<=0" => errors(BadTypeLeftOperand@[2-3]) type(error),

    // number/error
    greater_than_0_error: "0>1/0" => errors(DivideByZero@3) type(error),
    greater_than_error_0: "1/0>0" => errors(DivideByZero@1) type(error),
    greater_or_equal_0_error: "0>=1/0" => errors(DivideByZero@4) type(error),
    greater_or_equal_error_0: "1/0>=0" => errors(DivideByZero@1) type(error),
    less_than_0_error: "0<1/0" => errors(DivideByZero@3) type(error),
    less_than_error_0: "1/0<0" => errors(DivideByZero@1) type(error),
    less_or_equal_0_error: "0<=1/0" => errors(DivideByZero@4) type(error),
    less_or_equal_error_0: "1/0<=0" => errors(DivideByZero@1) type(error),

    // booleans
    greater_than_true_true: "true>true" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    greater_than_true_false: "true>false" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    greater_than_false_true: "false>true" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    greater_than_false_false: "false>false" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),

    greater_or_equal_true_true: "true>=true" => errors(BadTypeLeftOperand@[4-5],BadTypeRightOperand@[4-5]) type(error),
    greater_or_equal_true_false: "true>=false" => errors(BadTypeLeftOperand@[4-5],BadTypeRightOperand@[4-5]) type(error),
    greater_or_equal_false_true: "false>=true" => errors(BadTypeLeftOperand@[5-6],BadTypeRightOperand@[5-6]) type(error),
    greater_or_equal_false_false: "false>=false" => errors(BadTypeLeftOperand@[5-6],BadTypeRightOperand@[5-6]) type(error),

    less_than_true_true: "true<true" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    less_than_true_false: "true<false" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    less_than_false_true: "false<true" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    less_than_false_false: "false<false" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),

    less_or_equal_true_true: "true<=true" => errors(BadTypeLeftOperand@[4-5],BadTypeRightOperand@[4-5]) type(error),
    less_or_equal_true_false: "true<=false" => errors(BadTypeLeftOperand@[4-5],BadTypeRightOperand@[4-5]) type(error),
    less_or_equal_false_true: "false<=true" => errors(BadTypeLeftOperand@[5-6],BadTypeRightOperand@[5-6]) type(error),
    less_or_equal_false_false: "false<=false" => errors(BadTypeLeftOperand@[5-6],BadTypeRightOperand@[5-6]) type(error),

    // boolean/nothing
    greater_than_true_nothing: "true>()" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    greater_than_false_nothing: "false>()" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    greater_than_nothing_true: "()>true" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    greater_than_nothing_false: "()>false" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    greater_or_equal_true_nothing: "true>=()" => errors(BadTypeLeftOperand@[4-5],BadTypeRightOperand@[4-5]) type(error),
    greater_or_equal_false_nothing: "false>=()" => errors(BadTypeLeftOperand@[5-6],BadTypeRightOperand@[5-6]) type(error),
    greater_or_equal_nothing_true: "()>=true" => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    greater_or_equal_nothing_false: "()>=false" => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    less_than_true_nothing: "true<()" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    less_than_false_nothing: "false<()" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    less_than_nothing_true: "()<true" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    less_than_nothing_false: "()<false" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    less_or_equal_true_nothing: "true<=()" => errors(BadTypeLeftOperand@[4-5],BadTypeRightOperand@[4-5]) type(error),
    less_or_equal_false_nothing: "false<=()" => errors(BadTypeLeftOperand@[5-6],BadTypeRightOperand@[5-6]) type(error),
    less_or_equal_nothing_true: "()<=true" => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    less_or_equal_nothing_false: "()<=false" => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),

    // boolean/error
    greater_than_true_error: "true>1/0" => errors(BadTypeLeftOperand@4,DivideByZero@6) type(error),
    greater_than_false_error: "false>1/0" => errors(BadTypeLeftOperand@5,DivideByZero@7) type(error),
    greater_than_error_true: "1/0>true" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    greater_than_error_false: "1/0>false" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    greater_or_equal_true_error: "true>=1/0" => errors(BadTypeLeftOperand@[4-5],DivideByZero@7) type(error),
    greater_or_equal_false_error: "false>=1/0" => errors(BadTypeLeftOperand@[5-6],DivideByZero@8) type(error),
    greater_or_equal_error_true: "1/0>=true" => errors(DivideByZero@1,BadTypeRightOperand@[3-4]) type(error),
    greater_or_equal_error_false: "1/0>=false" => errors(DivideByZero@1,BadTypeRightOperand@[3-4]) type(error),
    less_than_true_error: "true<1/0" => errors(BadTypeLeftOperand@4,DivideByZero@6) type(error),
    less_than_false_error: "false<1/0" => errors(BadTypeLeftOperand@5,DivideByZero@7) type(error),
    less_than_error_true: "1/0<true" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    less_than_error_false: "1/0<false" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    less_or_equal_true_error: "true<=1/0" => errors(BadTypeLeftOperand@[4-5],DivideByZero@7) type(error),
    less_or_equal_false_error: "false<=1/0" => errors(BadTypeLeftOperand@[5-6],DivideByZero@8) type(error),
    less_or_equal_error_true: "1/0<=true" => errors(DivideByZero@1,BadTypeRightOperand@[3-4]) type(error),
    less_or_equal_error_false: "1/0<=false" => errors(DivideByZero@1,BadTypeRightOperand@[3-4]) type(error),

    // nothing
    greater_than_nothing_nothing: "()>()" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    greater_or_equal_nothing_nothing: "()>=()" => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),
    less_than_nothing_nothing: "()<()" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    less_or_equal_nothing_nothing: "()<=()" => errors(BadTypeLeftOperand@[2-3],BadTypeRightOperand@[2-3]) type(error),

    // nothing/error
    greater_than_nothing_error: "()>1/0" => errors(BadTypeLeftOperand@2,DivideByZero@4) type(error),
    greater_than_error_nothing: "1/0>()" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    greater_or_equal_nothing_error: "()>=1/0" => errors(BadTypeLeftOperand@[2-3],DivideByZero@5) type(error),
    greater_or_equal_error_nothing: "1/0>=()" => errors(DivideByZero@1,BadTypeRightOperand@[3-4]) type(error),
    less_than_nothing_error: "()<1/0" => errors(BadTypeLeftOperand@2,DivideByZero@4) type(error),
    less_than_error_nothing: "1/0<()" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    less_or_equal_error_nothing: "()<=1/0" => errors(BadTypeLeftOperand@[2-3],DivideByZero@5) type(error),
    less_or_equal_nothing_error: "1/0<=()" => errors(DivideByZero@1,BadTypeRightOperand@[3-4]) type(error),

    // errors
    greater_than_error_error: "1/0>1/0" => errors(DivideByZero@1,DivideByZero@5) type(error),
    greater_or_equal_error_error: "1/0>=1/0" => errors(DivideByZero@1,DivideByZero@6) type(error),
    less_than_error_error: "1/0<1/0" => errors(DivideByZero@1,DivideByZero@5) type(error),
    less_or_equal_error_error: "1/0<=1/0" => errors(DivideByZero@1,DivideByZero@6) type(error),
}