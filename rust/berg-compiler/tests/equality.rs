#![recursion_limit = "256"]
#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    // booleans
    equal_true_true: "true==true" => value(true),
    equal_true_false: "true==false" => value(false),
    equal_false_true: "false==true" => value(false),
    equal_false_false: "false==false" => value(true),

    not_equal_true_true: "true!=true" => value(false),
    not_equal_true_false: "true!=false" => value(true),
    not_equal_false_true: "false!=true" => value(true),
    not_equal_false_false: "false!=false" => value(false),

    // numbers
    equal_0_0: "0==0" => value(true),
    equal_1_1: "1==1" => value(true),
    equal_1_0: "1==0" => value(false),
    equal_0_1: "0==1" => value(false),
    equal_1_2: "1==2" => value(false),
    equal_big_big: "1982648372164176312796419487198==1982648372164176312796419487198" => value(true),
    equal_big_big2: "1982648372164176312796419487198==99127934712648732164276347216429663" => value(false),
    equal_big2_big: "99127934712648732164276347216429663==1982648372164176312796419487198" => value(false),

    not_equal_0_0: "0!=0" => value(false),
    not_equal_1_1: "1!=1" => value(false),
    not_equal_1_0: "1!=0" => value(true),
    not_equal_0_1: "0!=1" => value(true),
    not_equal_1_2: "1!=2" => value(true),
    not_equal_big_big: "1982648372164176312796419487198!=1982648372164176312796419487198" => value(false),
    not_equal_big_big2: "1982648372164176312796419487198!=99127934712648732164276347216429663" => value(true),
    not_equal_big2_big: "99127934712648732164276347216429663!=1982648372164176312796419487198" => value(true),

    // nothing
    equal_nothing_nothing: "()==()" => value(true),
    not_equal_nothing_nothing: "()!=()" => value(false),

    // errors
    equal_error_error: "1/0==1/0" => error(DivideByZero@2),
    not_equal_error_error: "1/0!=1/0" => error(DivideByZero@2),

    // nothing/boolean
    equal_nothing_true: "()==true" => value(false),
    equal_nothing_false: "()==false" => value(false),
    equal_true_nothing: "true==()" => value(false),
    equal_false_nothing: "false==()" => value(false),
    not_equal_nothing_true: "()!=true" => value(true),
    not_equal_nothing_false: "()!=false" => value(true),
    not_equal_true_nothing: "true!=()" => value(true),
    not_equal_false_nothing: "false!=()" => value(true),

    // nothing/number
    equal_nothing_0: "()==0" => value(false),
    equal_nothing_1: "()==1" => value(false),
    equal_0_nothing: "0==()" => value(false),
    equal_1_nothing: "1==()" => value(false),
    not_equal_nothing_0: "()!=0" => value(true),
    not_equal_nothing_1: "()!=1" => value(true),
    not_equal_0_nothing: "0!=()" => value(true),
    not_equal_1_nothing: "1!=()" => value(true),

    // error/boolean
    equal_error_true: "1/0==true" => error(DivideByZero@2),
    equal_error_false: "1/0==false" => error(DivideByZero@2),
    equal_true_error: "true==1/0" => error(DivideByZero@8),
    equal_false_error: "false==1/0" => error(DivideByZero@9),
    not_equal_error_true: "1/0!=true" => error(DivideByZero@2),
    not_equal_error_false: "1/0!=false" => error(DivideByZero@2),
    not_equal_true_error: "true!=1/0" => error(DivideByZero@8),
    not_equal_false_error: "false!=1/0" => error(DivideByZero@9),

    // error/number
    equal_error_0: "1/0==0" => error(DivideByZero@2),
    equal_error_1: "1/0==1" => error(DivideByZero@2),
    equal_0_error: "0==1/0" => error(DivideByZero@5),
    equal_1_error: "1==1/0" => error(DivideByZero@5),
    not_equal_error_0: "1/0!=0" => error(DivideByZero@2),
    not_equal_error_1: "1/0!=1" => error(DivideByZero@2),
    not_equal_0_error: "0!=1/0" => error(DivideByZero@5),
    not_equal_1_error: "1!=1/0" => error(DivideByZero@5),

    // number/boolean
    equal_true_1: "true==1" => value(false),
    equal_true_0: "true==0" => value(false),
    equal_false_1: "false==1" => value(false),
    equal_false_0: "false==0" => value(false),
    equal_1_true: "1==true" => value(false),
    equal_0_true: "0==true" => value(false),
    equal_1_false: "1==false" => value(false),
    equal_0_false: "0==false" => value(false),
    not_equal_true_1: "true!=1" => value(true),
    not_equal_true_0: "true!=0" => value(true),
    not_equal_false_1: "false!=1" => value(true),
    not_equal_false_0: "false!=0" => value(true),
    not_equal_1_true: "1!=true" => value(true),
    not_equal_0_true: "0!=true" => value(true),
    not_equal_1_false: "1!=false" => value(true),
    not_equal_0_false: "0!=false" => value(true),
}
