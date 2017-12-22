#[macro_use]
pub mod compiler_test;

compiler_tests! {
    // number/boolean
    greater_than_0_false: "0>false" => error(BadType@1),
    greater_than_false_0: "false>0" => error(BadType@5),
    greater_than_1_false: "1>false" => error(BadType@1),
    greater_than_false_1: "false>1" => error(BadType@5),
    greater_than_0_true: "0>true" => error(BadType@1),
    greater_than_true_0: "true>0" => error(BadType@4),
    greater_than_1_true: "1>true" => error(BadType@1),
    greater_than_true_1: "true>1" => error(BadType@4),

    greater_or_equal_0_false: "0>=false" => error(BadType@[1-2]),
    greater_or_equal_false_0: "false>=0" => error(BadType@[5-6]),
    greater_or_equal_1_false: "1>=false" => error(BadType@[1-2]),
    greater_or_equal_false_1: "false>=1" => error(BadType@[5-6]),
    greater_or_equal_0_true: "0>=true" => error(BadType@[1-2]),
    greater_or_equal_true_0: "true>=0" => error(BadType@[4-5]),
    greater_or_equal_1_true: "1>=true" => error(BadType@[1-2]),
    greater_or_equal_true_1: "true>=1" => error(BadType@[4-5]),

    less_than_0_false: "0<false" => error(BadType@1),
    less_than_false_0: "false<0" => error(BadType@5),
    less_than_1_false: "1<false" => error(BadType@1),
    less_than_false_1: "false<1" => error(BadType@5),
    less_than_0_true: "0<true" => error(BadType@1),
    less_than_true_0: "true<0" => error(BadType@4),
    less_than_1_true: "1<true" => error(BadType@1),
    less_than_true_1: "true<1" => error(BadType@4),

    less_or_equal_0_false: "0<=false" => error(BadType@[1-2]),
    less_or_equal_false_0: "false<=0" => error(BadType@[5-6]),
    less_or_equal_1_false: "1<=false" => error(BadType@[1-2]),
    less_or_equal_false_1: "false<=1" => error(BadType@[5-6]),
    less_or_equal_0_true: "0<=true" => error(BadType@[1-2]),
    less_or_equal_true_0: "true<=0" => error(BadType@[4-5]),
    less_or_equal_1_true: "1<=true" => error(BadType@[1-2]),
    less_or_equal_true_1: "true<=1" => error(BadType@[4-5]),

    // number/nothing
    greater_than_0_nothing: "0>()" => errors(BadType@1),
    greater_than_nothing_0: "()>0" => errors(BadType@2),
    greater_or_equal_0_nothing: "0>=()" => errors(BadType@[1-2]),
    greater_or_equal_nothing_0: "()>=0" => errors(BadType@[2-3]),
    less_than_0_nothing: "0<()" => errors(BadType@1),
    less_than_nothing_0: "()<0" => errors(BadType@2),
    less_or_equal_0_nothing: "0<=()" => errors(BadType@[1-2]),
    less_or_equal_nothing_0: "()<=0" => errors(BadType@[2-3]),

    // number/error
    greater_than_0_error: "0>1/0" => errors(DivideByZero@3),
    greater_than_error_0: "1/0>0" => errors(DivideByZero@1),
    greater_or_equal_0_error: "0>=1/0" => errors(DivideByZero@4),
    greater_or_equal_error_0: "1/0>=0" => errors(DivideByZero@1),
    less_than_0_error: "0<1/0" => errors(DivideByZero@3),
    less_than_error_0: "1/0<0" => errors(DivideByZero@1),
    less_or_equal_0_error: "0<=1/0" => errors(DivideByZero@4),
    less_or_equal_error_0: "1/0<=0" => errors(DivideByZero@1),

    // booleans
    greater_than_true_true: "true>true" => errors(BadType@4,BadType@4),
    greater_than_true_false: "true>false" => errors(BadType@4,BadType@4),
    greater_than_false_true: "false>true" => errors(BadType@5,BadType@5),
    greater_than_false_false: "false>false" => errors(BadType@5,BadType@5),

    greater_or_equal_true_true: "true>=true" => errors(BadType@[4-5],BadType@[4-5]),
    greater_or_equal_true_false: "true>=false" => errors(BadType@[4-5],BadType@[4-5]),
    greater_or_equal_false_true: "false>=true" => errors(BadType@[5-6],BadType@[5-6]),
    greater_or_equal_false_false: "false>=false" => errors(BadType@[5-6],BadType@[5-6]),

    less_than_true_true: "true<true" => errors(BadType@4,BadType@4),
    less_than_true_false: "true<false" => errors(BadType@4,BadType@4),
    less_than_false_true: "false<true" => errors(BadType@5,BadType@5),
    less_than_false_false: "false<false" => errors(BadType@5,BadType@5),

    less_or_equal_true_true: "true<=true" => errors(BadType@[4-5],BadType@[4-5]),
    less_or_equal_true_false: "true<=false" => errors(BadType@[4-5],BadType@[4-5]),
    less_or_equal_false_true: "false<=true" => errors(BadType@[5-6],BadType@[5-6]),
    less_or_equal_false_false: "false<=false" => errors(BadType@[5-6],BadType@[5-6]),

    // boolean/nothing
    greater_than_true_nothing: "true>()" => errors(BadType@4,BadType@4),
    greater_than_false_nothing: "false>()" => errors(BadType@5,BadType@5),
    greater_than_nothing_true: "()>true" => errors(BadType@2,BadType@2),
    greater_than_nothing_false: "()>false" => errors(BadType@2,BadType@2),
    greater_or_equal_true_nothing: "true>=()" => errors(BadType@[4-5],BadType@[4-5]),
    greater_or_equal_false_nothing: "false>=()" => errors(BadType@[5-6],BadType@[5-6]),
    greater_or_equal_nothing_true: "()>=true" => errors(BadType@[2-3],BadType@[2-3]),
    greater_or_equal_nothing_false: "()>=false" => errors(BadType@[2-3],BadType@[2-3]),
    less_than_true_nothing: "true<()" => errors(BadType@4,BadType@4),
    less_than_false_nothing: "false<()" => errors(BadType@5,BadType@5),
    less_than_nothing_true: "()<true" => errors(BadType@2,BadType@2),
    less_than_nothing_false: "()<false" => errors(BadType@2,BadType@2),
    less_or_equal_true_nothing: "true<=()" => errors(BadType@[4-5],BadType@[4-5]),
    less_or_equal_false_nothing: "false<=()" => errors(BadType@[5-6],BadType@[5-6]),
    less_or_equal_nothing_true: "()<=true" => errors(BadType@[2-3],BadType@[2-3]),
    less_or_equal_nothing_false: "()<=false" => errors(BadType@[2-3],BadType@[2-3]),

    // boolean/error
    greater_than_true_error: "true>1/0" => errors(BadType@4,DivideByZero@6),
    greater_than_false_error: "false>1/0" => errors(BadType@5,DivideByZero@7),
    greater_than_error_true: "1/0>true" => errors(DivideByZero@1,BadType@3),
    greater_than_error_false: "1/0>false" => errors(DivideByZero@1,BadType@3),
    greater_or_equal_true_error: "true>=1/0" => errors(BadType@[4-5],DivideByZero@7),
    greater_or_equal_false_error: "false>=1/0" => errors(BadType@[5-6],DivideByZero@8),
    greater_or_equal_error_true: "1/0>=true" => errors(DivideByZero@1,BadType@[3-4]),
    greater_or_equal_error_false: "1/0>=false" => errors(DivideByZero@1,BadType@[3-4]),
    less_than_true_error: "true<1/0" => errors(BadType@4,DivideByZero@6),
    less_than_false_error: "false<1/0" => errors(BadType@5,DivideByZero@7),
    less_than_error_true: "1/0<true" => errors(DivideByZero@1,BadType@3),
    less_than_error_false: "1/0<false" => errors(DivideByZero@1,BadType@3),
    less_or_equal_true_error: "true<=1/0" => errors(BadType@[4-5],DivideByZero@7),
    less_or_equal_false_error: "false<=1/0" => errors(BadType@[5-6],DivideByZero@8),
    less_or_equal_error_true: "1/0<=true" => errors(DivideByZero@1,BadType@[3-4]),
    less_or_equal_error_false: "1/0<=false" => errors(DivideByZero@1,BadType@[3-4]),

    // nothing
    greater_than_nothing_nothing: "()>()" => errors(BadType@2,BadType@2),
    greater_or_equal_nothing_nothing: "()>=()" => errors(BadType@[2-3],BadType@[2-3]),
    less_than_nothing_nothing: "()<()" => errors(BadType@2,BadType@2),
    less_or_equal_nothing_nothing: "()<=()" => errors(BadType@[2-3],BadType@[2-3]),

    // nothing/error
    greater_than_nothing_error: "()>1/0" => errors(BadType@2,DivideByZero@4),
    greater_than_error_nothing: "1/0>()" => errors(DivideByZero@1,BadType@3),
    greater_or_equal_nothing_error: "()>=1/0" => errors(BadType@[2-3],DivideByZero@5),
    greater_or_equal_error_nothing: "1/0>=()" => errors(DivideByZero@1,BadType@[3-4]),
    less_than_nothing_error: "()<1/0" => errors(BadType@2,DivideByZero@4),
    less_than_error_nothing: "1/0<()" => errors(DivideByZero@1,BadType@3),
    less_or_equal_error_nothing: "()<=1/0" => errors(BadType@[2-3],DivideByZero@5),
    less_or_equal_nothing_error: "1/0<=()" => errors(DivideByZero@1,BadType@[3-4]),

    // errors
    greater_than_error_error: "1/0>1/0" => errors(DivideByZero@1,DivideByZero@5),
    greater_or_equal_error_error: "1/0>=1/0" => errors(DivideByZero@1,DivideByZero@6),
    less_than_error_error: "1/0<1/0" => errors(DivideByZero@1,DivideByZero@5),
    less_or_equal_error_error: "1/0<=1/0" => errors(DivideByZero@1,DivideByZero@6),
}