#[macro_use]
pub mod compiler_test;

compiler_tests! {
    // booleans
    equal_true_true: "true==true" => type(true),
    equal_true_false: "true==false" => type(false),
    equal_false_true: "false==true" => type(false),
    equal_false_false: "false==false" => type(true),

    not_equal_true_true: "true!=true" => type(false),
    not_equal_true_false: "true!=false" => type(true),
    not_equal_false_true: "false!=true" => type(true),
    not_equal_false_false: "false!=false" => type(false),

    // numbers
    equal_0_0: "0==0" => type(true),
    equal_1_1: "1==1" => type(true),
    equal_1_0: "1==0" => type(false),
    equal_0_1: "0==1" => type(false),
    equal_1_2: "1==2" => type(false),
    equal_big_big: "1982648372164176312796419487198==1982648372164176312796419487198" => type(true),
    equal_big_big2: "1982648372164176312796419487198==99127934712648732164276347216429663" => type(false),
    equal_big2_big: "99127934712648732164276347216429663==1982648372164176312796419487198" => type(false),

    not_equal_0_0: "0!=0" => type(false),
    not_equal_1_1: "1!=1" => type(false),
    not_equal_1_0: "1!=0" => type(true),
    not_equal_0_1: "0!=1" => type(true),
    not_equal_1_2: "1!=2" => type(true),
    not_equal_big_big: "1982648372164176312796419487198!=1982648372164176312796419487198" => type(false),
    not_equal_big_big2: "1982648372164176312796419487198!=99127934712648732164276347216429663" => type(true),
    not_equal_big2_big: "99127934712648732164276347216429663!=1982648372164176312796419487198" => type(true),

    // nothing
    equal_nothing_nothing: "()==()" => type(true),
    not_equal_nothing_nothing: "()!=()" => type(false),

    // errors
    equal_error_error: "1/0==1/0" => type(error),
    not_equal_error_error: "1/0!=1/0" => type(error),

    // nothing/boolean
    equal_nothing_true: "()==true" => type(false),
    equal_nothing_false: "()==false" => type(false),
    equal_true_nothing: "true==()" => type(false),
    equal_false_nothing: "false==()" => type(false),
    not_equal_nothing_true: "()!=true" => type(true),
    not_equal_nothing_false: "()!=false" => type(true),
    not_equal_true_nothing: "true!=()" => type(true),
    not_equal_false_nothing: "false!=()" => type(true),

    // nothing/number
    equal_nothing_0: "()==0" => type(false),
    equal_nothing_1: "()==1" => type(false),
    equal_0_nothing: "0==()" => type(false),
    equal_1_nothing: "1==()" => type(false),
    not_equal_nothing_0: "()!=0" => type(true),
    not_equal_nothing_1: "()!=1" => type(true),
    not_equal_0_nothing: "0!=()" => type(true),
    not_equal_1_nothing: "1!=()" => type(true),

    // error/boolean
    equal_error_true: "1/0==true" => type(error),
    equal_error_false: "1/0==false" => type(error),
    equal_true_error: "true==1/0" => type(error),
    equal_false_error: "false==1/0" => type(error),
    not_equal_error_true: "1/0!=true" => type(error),
    not_equal_error_false: "1/0!=false" => type(error),
    not_equal_true_error: "true!=1/0" => type(error),
    not_equal_false_error: "false!=1/0" => type(error),

    // error/number
    equal_error_0: "1/0==0" => type(error),
    equal_error_1: "1/0==1" => type(error),
    equal_0_error: "0==1/0" => type(error),
    equal_1_error: "1==1/0" => type(error),
    not_equal_error_0: "1/0!=0" => type(error),
    not_equal_error_1: "1/0!=1" => type(error),
    not_equal_0_error: "0!=1/0" => type(error),
    not_equal_1_error: "1!=1/0" => type(error),

    // number/boolean
    equal_true_1: "true==1" => type(false),
    equal_true_0: "true==0" => type(false),
    equal_false_1: "false==1" => type(false),
    equal_false_0: "false==0" => type(false),
    equal_1_true: "1==true" => type(false),
    equal_0_true: "0==true" => type(false),
    equal_1_false: "1==false" => type(false),
    equal_0_false: "0==false" => type(false),
    not_equal_true_1: "true!=1" => type(true),
    not_equal_true_0: "true!=0" => type(true),
    not_equal_false_1: "false!=1" => type(true),
    not_equal_false_0: "false!=0" => type(true),
    not_equal_1_true: "1!=true" => type(true),
    not_equal_0_true: "0!=true" => type(true),
    not_equal_1_false: "1!=false" => type(true),
    not_equal_0_false: "0!=false" => type(true),
}
