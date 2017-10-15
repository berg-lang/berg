#[macro_use]
pub mod compiler_test;

compiler_tests! {
    add0_0: "0+0" => result(0),
    add0_1: "0+1" => result(1),
    add1_0: "1+0" => result(1),
    add1_1: "1+1" => result(2),
    add123_123: "123+123" => result(246),
    add1_2_3: "1+2+3" => result(6),

    sub0_0: "0-0" => result(0),
    sub1_1: "1-1" => result(0),
    sub2_1: "2-1" => result(1),
    sub1_2: "1-2" => result(-1),
    sub1_0: "1-0" => result(1),
    sub369_123: "369-123" => result(246),
    sub6_2_1: "6-2-1" => result(3),

    addsub0_0_0: "0+0-0" => result(0),
    addsub0_0_0_other: "0-0+0" => result(0),
    addsub0_0_0_neg: "-0+0-0" => result(0),
    addsub0_0_0_neg_other: "-0-0+0" => result(0),
    addsub0_0_0_pos: "+0+0-0" => result(0),
    addsub0_0_0_pos_other: "+0-0+0" => result(0),
    addsub1_2_3: "1+2-3" => result(0),
    addsub1_2_3_other: "1-2+3" => result(2),
    addsub1_2_3_neg: "-1+2-3" => result(-2),
    addsub1_2_3_neg_other: "-1-2+3" => result(0),
    addsub1_2_3_pos: "+1+2-3" => result(0),
    addsub1_2_3_pos_other: "+1-2+3" => result(2),

    neg_0: "-0" => result(0),
    neg_1: "-1" => result(-1),
    pos_0: "+0" => result(0),
    pos_1: "+1" => result(1),
}
