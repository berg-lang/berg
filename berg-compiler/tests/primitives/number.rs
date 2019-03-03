mod literals {
    use crate::*;

    #[test]
    fn zero() {
        expect("0").to_yield(0)
    }
    #[test]
    fn one() {
        expect("1").to_yield(1)
    }
    #[test]
    fn huge() {
        expect("999999999999999999999999999999999999999999999").to_yield(
            BigRational::from_str("999999999999999999999999999999999999999999999").unwrap(),
        )
    }
}

mod operators_comparison {
    use crate::*;

    #[test]
    fn equal_0_0() {
        expect("0==0").to_yield(true)
    }

    #[test]
    fn equal_0_1() {
        expect("0==1").to_yield(false)
    }

    #[test]
    fn equal_0_error() {
        expect("0==1/0").to_error(DivideByZero, 5)
    }

    #[test]
    fn equal_0_false() {
        expect("0==false").to_yield(false)
    }

    #[test]
    fn equal_0_empty() {
        expect("0==()").to_yield(false)
    }

    #[test]
    fn equal_0_true() {
        expect("0==true").to_yield(false)
    }

    #[test]
    fn equal_1_0() {
        expect("1==0").to_yield(false)
    }

    #[test]
    fn equal_1_1() {
        expect("1==1").to_yield(true)
    }

    #[test]
    fn equal_1_2() {
        expect("1==2").to_yield(false)
    }

    #[test]
    fn equal_1_error() {
        expect("1==1/0").to_error(DivideByZero, 5)
    }

    #[test]
    fn equal_1_false() {
        expect("1==false").to_yield(false)
    }

    #[test]
    fn equal_1_empty() {
        expect("1==()").to_yield(false)
    }

    #[test]
    fn equal_1_true() {
        expect("1==true").to_yield(false)
    }

    #[test]
    fn equal_big_big() {
        expect("1982648372164176312796419487198==1982648372164176312796419487198").to_yield(true)
    }

    #[test]
    fn equal_big_big2() {
        expect("1982648372164176312796419487198==99127934712648732164276347216429663")
            .to_yield(false)
    }

    #[test]
    fn equal_big2_big() {
        expect("99127934712648732164276347216429663==1982648372164176312796419487198")
            .to_yield(false)
    }

    #[test]
    fn not_equal_0_0() {
        expect("0!=0").to_yield(false)
    }

    #[test]
    fn not_equal_0_1() {
        expect("0!=1").to_yield(true)
    }

    #[test]
    fn not_equal_0_error() {
        expect("0!=1/0").to_error(DivideByZero, 5)
    }

    #[test]
    fn not_equal_0_false() {
        expect("0!=false").to_yield(true)
    }

    #[test]
    fn not_equal_0_empty() {
        expect("0!=()").to_yield(true)
    }

    #[test]
    fn not_equal_0_true() {
        expect("0!=true").to_yield(true)
    }

    #[test]
    fn not_equal_1_0() {
        expect("1!=0").to_yield(true)
    }

    #[test]
    fn not_equal_1_1() {
        expect("1!=1").to_yield(false)
    }

    #[test]
    fn not_equal_1_2() {
        expect("1!=2").to_yield(true)
    }

    #[test]
    fn not_equal_1_error() {
        expect("1!=1/0").to_error(DivideByZero, 5)
    }

    #[test]
    fn not_equal_1_false() {
        expect("1!=false").to_yield(true)
    }

    #[test]
    fn not_equal_1_empty() {
        expect("1!=()").to_yield(true)
    }

    #[test]
    fn not_equal_1_true() {
        expect("1!=true").to_yield(true)
    }

    #[test]
    fn not_equal_big_big() {
        expect("1982648372164176312796419487198!=1982648372164176312796419487198").to_yield(false)
    }

    #[test]
    fn not_equal_big_big2() {
        expect("1982648372164176312796419487198!=99127934712648732164276347216429663")
            .to_yield(true)
    }

    #[test]
    fn not_equal_big2_big() {
        expect("99127934712648732164276347216429663!=1982648372164176312796419487198")
            .to_yield(true)
    }

    #[test]
    fn greater_or_equal_0_0() {
        expect("0>=0").to_yield(true)
    }

    #[test]
    fn greater_or_equal_0_1() {
        expect("0>=1").to_yield(false)
    }

    #[test]
    fn greater_or_equal_0_error() {
        expect("0>=1/0").to_error(DivideByZero, 5)
    }

    #[test]
    fn greater_or_equal_0_false() {
        expect("0>=false").to_error(BadOperandType, 3..=7)
    }

    #[test]
    fn greater_or_equal_0_empty() {
        expect("0>=()").to_error(BadOperandType, 3..=4)
    }

    #[test]
    fn greater_or_equal_0_true() {
        expect("0>=true").to_error(BadOperandType, 3..=6)
    }

    #[test]
    fn greater_or_equal_1_0() {
        expect("1>=0").to_yield(true)
    }

    #[test]
    fn greater_or_equal_1_1() {
        expect("1>=1").to_yield(true)
    }

    #[test]
    fn greater_or_equal_1_2() {
        expect("1>=2").to_yield(false)
    }

    #[test]
    fn greater_or_equal_1_false() {
        expect("1>=false").to_error(BadOperandType, 3..=7)
    }

    #[test]
    fn greater_or_equal_1_true() {
        expect("1>=true").to_error(BadOperandType, 3..=6)
    }

    #[test]
    fn greater_or_equal_big_big() {
        expect("1982648372164176312796419487198>=1982648372164176312796419487198").to_yield(true)
    }

    #[test]
    fn greater_or_equal_big_big2() {
        expect("1982648372164176312796419487198>=99127934712648732164276347216429663")
            .to_yield(false)
    }

    #[test]
    fn greater_or_equal_big2_big() {
        expect("99127934712648732164276347216429663>=1982648372164176312796419487198")
            .to_yield(true)
    }

    #[test]
    fn greater_than_0_0() {
        expect("0>0").to_yield(false)
    }

    #[test]
    fn greater_than_0_1() {
        expect("0>1").to_yield(false)
    }

    #[test]
    fn greater_than_0_error() {
        expect("0>1/0").to_error(DivideByZero, 4)
    }

    #[test]
    fn greater_than_0_false() {
        expect("0>false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn greater_than_0_empty() {
        expect("0>()").to_error(BadOperandType, 2..=3)
    }

    #[test]
    fn greater_than_0_true() {
        expect("0>true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn greater_than_1_0() {
        expect("1>0").to_yield(true)
    }

    #[test]
    fn greater_than_1_1() {
        expect("1>1").to_yield(false)
    }

    #[test]
    fn greater_than_1_2() {
        expect("1>2").to_yield(false)
    }

    #[test]
    fn greater_than_1_false() {
        expect("1>false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn greater_than_1_true() {
        expect("1>true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn greater_than_big_big() {
        expect("1982648372164176312796419487198>1982648372164176312796419487198").to_yield(false)
    }

    #[test]
    fn greater_than_big_big2() {
        expect("1982648372164176312796419487198>99127934712648732164276347216429663")
            .to_yield(false)
    }

    #[test]
    fn greater_than_big2_big() {
        expect("99127934712648732164276347216429663>1982648372164176312796419487198").to_yield(true)
    }

    #[test]
    fn less_or_equal_0_0() {
        expect("0<=0").to_yield(true)
    }

    #[test]
    fn less_or_equal_0_1() {
        expect("0<=1").to_yield(true)
    }

    #[test]
    fn less_or_equal_0_error() {
        expect("0<=1/0").to_error(DivideByZero, 5)
    }

    #[test]
    fn less_or_equal_0_false() {
        expect("0<=false").to_error(BadOperandType, 3..=7)
    }

    #[test]
    fn less_or_equal_0_empty() {
        expect("0<=()").to_error(BadOperandType, 3..=4)
    }

    #[test]
    fn less_or_equal_0_true() {
        expect("0<=true").to_error(BadOperandType, 3..=6)
    }

    #[test]
    fn less_or_equal_1_0() {
        expect("1<=0").to_yield(false)
    }

    #[test]
    fn less_or_equal_1_1() {
        expect("1<=1").to_yield(true)
    }

    #[test]
    fn less_or_equal_1_2() {
        expect("1<=2").to_yield(true)
    }

    #[test]
    fn less_or_equal_1_false() {
        expect("1<=false").to_error(BadOperandType, 3..=7)
    }

    #[test]
    fn less_or_equal_1_true() {
        expect("1<=true").to_error(BadOperandType, 3..=6)
    }

    #[test]
    fn less_or_equal_big_big() {
        expect("1982648372164176312796419487198<=1982648372164176312796419487198").to_yield(true)
    }

    #[test]
    fn less_or_equal_big_big2() {
        expect("1982648372164176312796419487198<=99127934712648732164276347216429663")
            .to_yield(true)
    }

    #[test]
    fn less_or_equal_big2_big() {
        expect("99127934712648732164276347216429663<=1982648372164176312796419487198")
            .to_yield(false)
    }

    #[test]
    fn less_than_0_0() {
        expect("0<0").to_yield(false)
    }

    #[test]
    fn less_than_0_1() {
        expect("0<1").to_yield(true)
    }

    #[test]
    fn less_than_0_error() {
        expect("0<1/0").to_error(DivideByZero, 4)
    }

    #[test]
    fn less_than_0_false() {
        expect("0<false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn less_than_0_empty() {
        expect("0<()").to_error(BadOperandType, 2..=3)
    }

    #[test]
    fn less_than_0_true() {
        expect("0<true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn less_than_1_0() {
        expect("1<0").to_yield(false)
    }

    #[test]
    fn less_than_1_1() {
        expect("1<1").to_yield(false)
    }

    #[test]
    fn less_than_1_2() {
        expect("1<2").to_yield(true)
    }

    #[test]
    fn less_than_1_false() {
        expect("1<false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn less_than_1_true() {
        expect("1<true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn less_than_big_big() {
        expect("1982648372164176312796419487198<1982648372164176312796419487198").to_yield(false)
    }

    #[test]
    fn less_than_big_big2() {
        expect("1982648372164176312796419487198<99127934712648732164276347216429663").to_yield(true)
    }

    #[test]
    fn less_than_big2_big() {
        expect("99127934712648732164276347216429663<1982648372164176312796419487198")
            .to_yield(false)
    }
}

mod operators_logical {
    use crate::*;

    #[test]
    fn and_1_1() {
        expect("1&&1").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn and_1_error() {
        expect("1&&(1/0)").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn and_1_false() {
        expect("1&&false").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn and_1_empty() {
        expect("1&&()").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn and_1_true() {
        expect("1&&true").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn or_1_1() {
        expect("1||1").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn or_1_error() {
        expect("1||(1/0)").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn or_1_false() {
        expect("1||false").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn or_1_empty() {
        expect("1||()").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn or_1_true() {
        expect("1||true").to_error(UnsupportedOperator, 1..=2)
    }

    #[test]
    fn not_1() {
        expect("!1").to_error(UnsupportedOperator, 0)
    }
}

mod operators_math {
    use crate::*;

    #[test]
    fn add0_0() {
        expect("0+0").to_yield(0)
    }

    #[test]
    fn add0_1() {
        expect("0+1").to_yield(1)
    }

    #[test]
    fn add1_0() {
        expect("1+0").to_yield(1)
    }

    #[test]
    fn add1_1() {
        expect("1+1").to_yield(2)
    }

    #[test]
    fn add_1_error() {
        expect("1+1/0").to_error(DivideByZero, 4)
    }

    #[test]
    fn add_1_false() {
        expect("1+false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn add_1_empty() {
        expect("1+()").to_error(BadOperandType, 2..=3)
    }

    #[test]
    fn add_1_true() {
        expect("1+true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn add123_123() {
        expect("123+123").to_yield(246)
    }

    #[test]
    fn sub0_0() {
        expect("0-0").to_yield(0)
    }

    #[test]
    fn sub1_0() {
        expect("1-0").to_yield(1)
    }

    #[test]
    fn sub1_1() {
        expect("1-1").to_yield(0)
    }

    #[test]
    fn sub1_2() {
        expect("1-2").to_yield(-1)
    }

    #[test]
    fn sub_1_error() {
        expect("1-1/0").to_error(DivideByZero, 4)
    }

    #[test]
    fn sub_1_false() {
        expect("1-false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn sub_1_empty() {
        expect("1-()").to_error(BadOperandType, 2..=3)
    }

    #[test]
    fn sub_1_true() {
        expect("1-true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn sub2_1() {
        expect("2-1").to_yield(1)
    }

    #[test]
    fn sub369_123() {
        expect("369-123").to_yield(246)
    }

    #[test]
    fn mul0_0() {
        expect("0*0").to_yield(0)
    }

    #[test]
    fn mul0_1() {
        expect("0*1").to_yield(0)
    }

    #[test]
    fn mul1_0() {
        expect("1*0").to_yield(0)
    }

    #[test]
    fn mul1_1() {
        expect("1*1").to_yield(1)
    }

    #[test]
    fn mul_1_error() {
        expect("1*(1/0)").to_error(DivideByZero, 5)
    }

    #[test]
    fn mul_1_false() {
        expect("1*false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn mul_1_empty() {
        expect("1*()").to_error(BadOperandType, 2..=3)
    }

    #[test]
    fn mul_1_true() {
        expect("1*true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn mul11_11() {
        expect("11*11").to_yield(121)
    }

    #[test]
    fn div0_0() {
        expect("0/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn div0_1() {
        expect("0/1").to_yield(0)
    }

    #[test]
    fn div1_0() {
        expect("1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn div1_2() {
        expect("1/2").to_yield(BigRational::new(1.into(), 2.into()))
    }

    #[test]
    fn div_1_error() {
        expect("1/(1/0)").to_error(DivideByZero, 5)
    }

    #[test]
    fn div_1_false() {
        expect("1/false").to_error(BadOperandType, 2..=6)
    }

    #[test]
    fn div_1_empty() {
        expect("1/()").to_error(BadOperandType, 2..=3)
    }

    #[test]
    fn div_1_true() {
        expect("1/true").to_error(BadOperandType, 2..=5)
    }

    #[test]
    fn div11_11() {
        expect("11/11").to_yield(1)
    }

    #[test]
    fn div12_1() {
        expect("12/1").to_yield(12)
    }

    #[test]
    fn div12_3() {
        expect("12/3").to_yield(4)
    }

    #[test]
    fn div15_7() {
        expect("15/7").to_yield(BigRational::new(15.into(), 7.into()))
    }

    #[test]
    fn neg_0() {
        expect("-0").to_yield(0)
    }

    #[test]
    fn neg_1() {
        expect("-1").to_yield(-1)
    }

    #[test]
    fn pos_0() {
        expect("+0").to_yield(0)
    }

    #[test]
    fn pos_1() {
        expect("+1").to_yield(1)
    }

    #[test]
    fn addsub0_0_0() {
        expect("0+0-0").to_yield(0)
    }

    #[test]
    fn subadd0_0_0() {
        expect("0-0+0").to_yield(0)
    }

    #[test]
    fn sub6_2_1() {
        expect("6-2-1").to_yield(3)
    }

    #[test]
    fn addsub1_2_3() {
        expect("1+2-3").to_yield(0)
    }

    #[test]
    fn subadd1_2_3() {
        expect("1-2+3").to_yield(2)
    }

    #[test]
    fn add1_2_3() {
        expect("1+2+3").to_yield(6)
    }

    #[test]
    fn div24_3_4() {
        expect("24/3/4").to_yield(2)
    }

    #[test]
    fn muladd2_3_4() {
        expect("2*3+4").to_yield(10)
    }

    #[test]
    fn addmul2_3_4() {
        expect("2+3*4").to_yield(14)
    }

    #[test]
    fn mul2_3_4() {
        expect("2*3*4").to_yield(24)
    }

    #[test]
    fn divadd2_3_4() {
        expect("30/2*3").to_yield(45)
    }

    #[test]
    fn div45_3_7() {
        expect("45/3/7").to_yield(BigRational::new(15.into(), 7.into()))
    }

    #[test]
    fn timesneg_5_2() {
        expect("5 * -2").to_yield(-10)
    }

    #[test]
    fn plusneg_5_2() {
        expect("5 + -2").to_yield(3)
    }

    #[test]
    fn adddiv3_8_2() {
        expect("3+8/2").to_yield(7)
    }
}
