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

    mul0_0: "0*0" => result(0),
    mul0_1: "0*1" => result(0),
    mul1_0: "1*0" => result(0),
    mul1_1: "1*1" => result(1),
    mul11_11: "11*11" => result(121),
    mul2_3_4: "2*3*4" => result(24),

    div0_1: "0/1" => result(0),
    div12_1: "12/1" => result(12),
    div12_3: "12/3" => result(4),
    div11_11: "11/11" => result(1),
    div24_3_4: "24/3/4" => result(2),
    div1_2: "1/2" => result(BigRational::new(1.into(), 2.into())),
    div15_7: "15/7" => result(BigRational::new(15.into(), 7.into())),
    div45_3_7: "45/3/7" => result(BigRational::new(15.into(), 7.into())),

    div0_0: "0/0" => error(DivideByZero@1) result(error),
    div1_0: "1/0" => error(DivideByZero@1) result(error),

    muladd2_3_4: "2*3+4" => result(10),
    muladd2_3_4_neg: "-2*3+4" => result(-2),
    muladd2_3_4_pos: "+2*3+4" => result(10),

    divadd2_3_4: "30/2*3" => result(45),
    divadd2_3_4_neg: "-30/2*3" => result(-45),
    divadd2_3_4_pos: "+30/2*3" => result(45),

    addsub0_0_0: "0+0-0" => result(0),
    addsub0_0_0_neg: "-0+0-0" => result(0),
    addsub0_0_0_pos: "+0+0-0" => result(0),
    addsub1_2_3: "1+2-3" => result(0),
    addsub1_2_3_neg: "-1+2-3" => result(-2),
    addsub1_2_3_pos: "+1+2-3" => result(0),

    subadd0_0_0: "0-0+0" => result(0),
    subadd0_0_0_neg: "-0-0+0" => result(0),
    subadd0_0_0_pos: "+0-0+0" => result(0),
    subadd1_2_3: "1-2+3" => result(2),
    subadd1_2_3_neg: "-1-2+3" => result(0),
    subadd1_2_3_pos: "+1-2+3" => result(2),

    neg_0: "-0" => result(0),
    neg_1: "-1" => result(-1),
    pos_0: "+0" => result(0),
    pos_1: "+1" => result(1),

    trailing_neg: "0-" => error(UnrecognizedOperator@1) result(error),
    trailing_pos: "0+" => error(UnrecognizedOperator@1) result(error),
    neg_only:      "-" => error(MissingRightOperand@0) result(error),
    pos_only:      "+" => error(MissingRightOperand@0) result(error),
    plus_minus: "1+-2" => error(UnrecognizedOperator@[1-2]) result(error),

    addmul2_3_4: "2+3*4" => error(OperatorsOutOfPrecedenceOrder@3) result(error),

    adddiv2_3_4: "2+3/4" => error(OperatorsOutOfPrecedenceOrder@3) result(error),
}
