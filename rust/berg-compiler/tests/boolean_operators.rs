#[macro_use]
pub mod compiler_test;

compiler_tests! {
    and_true_true: "true&&true"     => value(true),
    and_true_false: "true&&false"   => value(false),
    and_false_true: "false&&true"   => value(false),
    and_false_false: "false&&false" => value(false),

    or_true_true: "true||true"     => value(true),
    or_true_false: "true||false"   => value(true),
    or_false_true: "false||true"   => value(true),
    or_false_false: "false||false" => value(false),

    not_true: "!true"   => value(false),
    not_false: "!false" => value(true),

    and_equal_equal: "1==1&&2==2"   => value(true),
    and_ne_ne: "1!=2&&3!=4"         => value(true),
    and_greater_greater: "4>3&&2>1" => value(true),
    and_less_less: "1<2&&3<4"       => value(true),
    and_ge_ge: "4>=3&&2>=1"         => value(true),
    and_le_le: "1<=2&&3<=5"         => value(true),

    or_equal_equal: "1==2||2==2"   => value(true),
    or_ne_ne: "1!=1||3!=4"         => value(true),
    or_greater_greater: "4>5||2>1" => value(true),
    or_less_less: "3<2||3<4"       => value(true),
    or_ge_ge: "4>=5||2>=1"         => value(true),
    or_le_le: "4<=5||2<=1"         => value(true),

    and_or_ge_add_mul_true: "false||true&&7<=1+2*3" => value(true),
    and_or_ge_add_mul_false: "false||true&&8<=1+2*3" => value(false),
}
