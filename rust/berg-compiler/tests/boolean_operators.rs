#[macro_use]
pub mod compiler_test;

compiler_tests! {
    and_true_true: "true&&true"     => type(true),
    and_true_false: "true&&false"   => type(false),
    and_false_true: "false&&true"   => type(false),
    and_false_false: "false&&false" => type(false),

    or_true_true: "true||true"     => type(true),
    or_true_false: "true||false"   => type(true),
    or_false_true: "false||true"   => type(true),
    or_false_false: "false||false" => type(false),

    not_true: "!true"   => type(false),
    not_false: "!false" => type(true),

    and_equal_equal: "1==1&&2==2"   => type(true),
    and_ne_ne: "1!=2&&3!=4"         => type(true),
    and_greater_greater: "4>3&&2>1" => type(true),
    and_less_less: "1<2&&3<4"       => type(true),
    and_ge_ge: "4>=3&&2>=1"         => type(true),
    and_le_le: "1<=2&&3<=5"         => type(true),

    or_equal_equal: "1==2||2==2"   => type(true),
    or_ne_ne: "1!=1||3!=4"         => type(true),
    or_greater_greater: "4>5||2>1" => type(true),
    or_less_less: "3<2||3<4"       => type(true),
    or_ge_ge: "4>=5||2>=1"         => type(true),
    or_le_le: "4<=5||2<=1"         => type(true),

    and_or_ge_add_mul_true: "false||true&&7<=1+2*3" => type(true),
    and_or_ge_add_mul_false: "false||true&&8<=1+2*3" => type(false),
}
