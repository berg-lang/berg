#[macro_use]
pub mod compiler_test;
use compiler_test::berg_compiler::BigRational;

compiler_tests! {
    add0_0: "0+0" => value(0),
    add0_1: "0+1" => value(1),
    add1_0: "1+0" => value(1),
    add1_1: "1+1" => value(2),
    add123_123: "123+123" => value(246),
    add1_2_3: "1+2+3" => value(6),

    sub0_0: "0-0" => value(0),
    sub1_1: "1-1" => value(0),
    sub2_1: "2-1" => value(1),
    sub1_2: "1-2" => value(-1),
    sub1_0: "1-0" => value(1),
    sub369_123: "369-123" => value(246),
    sub6_2_1: "6-2-1" => value(3),

    mul0_0: "0*0" => value(0),
    mul0_1: "0*1" => value(0),
    mul1_0: "1*0" => value(0),
    mul1_1: "1*1" => value(1),
    mul11_11: "11*11" => value(121),
    mul2_3_4: "2*3*4" => value(24),

    div0_1: "0/1" => value(0),
    div12_1: "12/1" => value(12),
    div12_3: "12/3" => value(4),
    div11_11: "11/11" => value(1),
    div24_3_4: "24/3/4" => value(2),
    div1_2: "1/2" => value(BigRational::new(1.into(), 2.into())),
    div15_7: "15/7" => value(BigRational::new(15.into(), 7.into())),
    div45_3_7: "45/3/7" => value(BigRational::new(15.into(), 7.into())),

    div0_0: "0/0" => error(DivideByZero@1),
    div1_0: "1/0" => error(DivideByZero@1),

    muladd2_3_4: "2*3+4" => value(10),
    muladd2_3_4_neg: "-2*3+4" => value(-2),
    muladd2_3_4_pos: "+2*3+4" => value(10),

    addmul2_3_4: "2+3*4" => value(14),
    addmul2_3_4_neg: "-2+3*4" => value(10),
    addmul2_3_4_pos: "+2+3*4" => value(14),

    divadd2_3_4: "30/2*3" => value(45),
    divadd2_3_4_neg: "-30/2*3" => value(-45),
    divadd2_3_4_pos: "+30/2*3" => value(45),

    adddiv3_8_2: "3+8/2" => value(7),
    adddiv3_8_2_neg: "-3+8/2" => value(1),
    adddiv3_8_2_pos: "+3+8/2" => value(7),

    addsub0_0_0: "0+0-0" => value(0),
    addsub0_0_0_neg: "-0+0-0" => value(0),
    addsub0_0_0_pos: "+0+0-0" => value(0),
    addsub1_2_3: "1+2-3" => value(0),
    addsub1_2_3_neg: "-1+2-3" => value(-2),
    addsub1_2_3_pos: "+1+2-3" => value(0),

    subadd0_0_0: "0-0+0" => value(0),
    subadd0_0_0_neg: "-0-0+0" => value(0),
    subadd0_0_0_pos: "+0-0+0" => value(0),
    subadd1_2_3: "1-2+3" => value(2),
    subadd1_2_3_neg: "-1-2+3" => value(0),
    subadd1_2_3_pos: "+1-2+3" => value(2),

    neg_0: "-0" => value(0),
    neg_1: "-1" => value(-1),
    pos_0: "+0" => value(0),
    pos_1: "+1" => value(1),

    plusneg_5_2: "5 + -2" => value(3),
    timesneg_5_2: "5 * -2" => value(-10),
}
