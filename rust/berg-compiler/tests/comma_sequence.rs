#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    comma_sequence: "1,2" => value([1,2]),
    comma_left_space: "1 ,2" => value([1,2]),
    comma_right_space: "1, 2" => value([1,2]),
    comma_both_space: "1 , 2" => value([1,2]),

    comma_sequence_add: "1+1+1,2+2+2" => value([3,6]),
    comma_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false,false||true&&4>=3+2*1" => value([true,false]),
    comma_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false,false||true&&4<=3+2*1" => value([false,true]),

    right_comma: "1," => error(MissingOperand@2),
    left_comma: ",1" => error(MissingOperand@0),
    both_comma: ",1," => error(MissingOperand@0),

    left_double_comma: ",,1" => error(MissingOperand@0),
    right_double_comma: "1,," => error(MissingOperand@2),
    both_double_comma: ",,1,," => error(MissingOperand@0),
    between_double_comma: "1,,2" => error(MissingOperand@2),

    paren_comma_all_over: ",(,(,),)," => error(MissingOperand@0),
}
