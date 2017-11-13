#[macro_use]
pub mod compiler_test;

compiler_tests! {
    add0_0: "0+0" => type(0),
    add0_1: "0+1" => type(1),
    add1_0: "1+0" => type(1),
    add1_1: "1+1" => type(2),
    add123_123: "123+123" => type(246),
    add1_2_3: "1+2+3" => type(6),

    sub0_0: "0-0" => type(0),
    sub1_1: "1-1" => type(0),
    sub2_1: "2-1" => type(1),
    sub1_2: "1-2" => type(-1),
    sub1_0: "1-0" => type(1),
    sub369_123: "369-123" => type(246),
    sub6_2_1: "6-2-1" => type(3),

    mul0_0: "0*0" => type(0),
    mul0_1: "0*1" => type(0),
    mul1_0: "1*0" => type(0),
    mul1_1: "1*1" => type(1),
    mul11_11: "11*11" => type(121),
    mul2_3_4: "2*3*4" => type(24),

    div0_1: "0/1" => type(0),
    div12_1: "12/1" => type(12),
    div12_3: "12/3" => type(4),
    div11_11: "11/11" => type(1),
    div24_3_4: "24/3/4" => type(2),
    div1_2: "1/2" => type(BigRational::new(1.into(), 2.into())),
    div15_7: "15/7" => type(BigRational::new(15.into(), 7.into())),
    div45_3_7: "45/3/7" => type(BigRational::new(15.into(), 7.into())),

    div0_0: "0/0" => error(DivideByZero@1) type(error),
    div1_0: "1/0" => error(DivideByZero@1) type(error),

    muladd2_3_4: "2*3+4" => type(10),
    muladd2_3_4_neg: "-2*3+4" => type(-2),
    muladd2_3_4_pos: "+2*3+4" => type(10),

    addmul2_3_4: "2+3*4" => type(14),
    addmul2_3_4_neg: "-2+3*4" => type(10),
    addmul2_3_4_pos: "+2+3*4" => type(14),

    divadd2_3_4: "30/2*3" => type(45),
    divadd2_3_4_neg: "-30/2*3" => type(-45),
    divadd2_3_4_pos: "+30/2*3" => type(45),

    adddiv3_8_2: "3+8/2" => type(7),
    adddiv3_8_2_neg: "-3+8/2" => type(1),
    adddiv3_8_2_pos: "+3+8/2" => type(7),

    addsub0_0_0: "0+0-0" => type(0),
    addsub0_0_0_neg: "-0+0-0" => type(0),
    addsub0_0_0_pos: "+0+0-0" => type(0),
    addsub1_2_3: "1+2-3" => type(0),
    addsub1_2_3_neg: "-1+2-3" => type(-2),
    addsub1_2_3_pos: "+1+2-3" => type(0),

    subadd0_0_0: "0-0+0" => type(0),
    subadd0_0_0_neg: "-0-0+0" => type(0),
    subadd0_0_0_pos: "+0-0+0" => type(0),
    subadd1_2_3: "1-2+3" => type(2),
    subadd1_2_3_neg: "-1-2+3" => type(0),
    subadd1_2_3_pos: "+1-2+3" => type(2),

    neg_0: "-0" => type(0),
    neg_1: "-1" => type(-1),
    pos_0: "+0" => type(0),
    pos_1: "+1" => type(1),

    plusneg_5_2: "5 + -2" => type(3),
    timesneg_5_2: "5 * -2" => type(-10),
}
