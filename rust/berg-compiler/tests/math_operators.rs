pub mod compiler_test;
use compiler_test::*;

#[test] fn add0_0()          { expect( "0+0"     ).to_yield(0) }
#[test] fn add0_1()          { expect( "0+1"     ).to_yield(1) }
#[test] fn add1_0()          { expect( "1+0"     ).to_yield(1) }
#[test] fn add1_1()          { expect( "1+1"     ).to_yield(2) }
#[test] fn add123_123()      { expect( "123+123" ).to_yield(246) }
#[test] fn add1_2_3()        { expect( "1+2+3"   ).to_yield(6) }

#[test] fn sub0_0()          { expect( "0-0"     ).to_yield(0) }
#[test] fn sub1_1()          { expect( "1-1"     ).to_yield(0) }
#[test] fn sub2_1()          { expect( "2-1"     ).to_yield(1) }
#[test] fn sub1_2()          { expect( "1-2"     ).to_yield(-1) }
#[test] fn sub1_0()          { expect( "1-0"     ).to_yield(1) }
#[test] fn sub369_123()      { expect( "369-123" ).to_yield(246) }
#[test] fn sub6_2_1()        { expect( "6-2-1"   ).to_yield(3) }

#[test] fn mul0_0()          { expect( "0*0"     ).to_yield(0) }
#[test] fn mul0_1()          { expect( "0*1"     ).to_yield(0) }
#[test] fn mul1_0()          { expect( "1*0"     ).to_yield(0) }
#[test] fn mul1_1()          { expect( "1*1"     ).to_yield(1) }
#[test] fn mul11_11()        { expect( "11*11"   ).to_yield(121) }
#[test] fn mul2_3_4()        { expect( "2*3*4"   ).to_yield(24) }

#[test] fn div0_1()          { expect( "0/1"     ).to_yield(0) }
#[test] fn div12_1()         { expect( "12/1"    ).to_yield(12) }
#[test] fn div12_3()         { expect( "12/3"    ).to_yield(4) }
#[test] fn div11_11()        { expect( "11/11"   ).to_yield(1) }
#[test] fn div24_3_4()       { expect( "24/3/4"  ).to_yield(2) }
#[test] fn div1_2()          { expect( "1/2"     ).to_yield(BigRational::new(1.into(), 2.into())) }
#[test] fn div15_7()         { expect( "15/7"    ).to_yield(BigRational::new(15.into(), 7.into())) }
#[test] fn div45_3_7()       { expect( "45/3/7"  ).to_yield(BigRational::new(15.into(), 7.into())) }

#[test] fn div0_0()          { expect( "0/0"     ).to_error(DivideByZero,2) }
#[test] fn div1_0()          { expect( "1/0"     ).to_error(DivideByZero,2) }

#[test] fn muladd2_3_4()     { expect( "2*3+4"   ).to_yield(10) }
#[test] fn muladd2_3_4_neg() { expect( "-2*3+4"  ).to_yield(-2) }
#[test] fn muladd2_3_4_pos() { expect( "+2*3+4"  ).to_yield(10) }

#[test] fn addmul2_3_4()     { expect( "2+3*4"   ).to_yield(14) }
#[test] fn addmul2_3_4_neg() { expect( "-2+3*4"  ).to_yield(10) }
#[test] fn addmul2_3_4_pos() { expect( "+2+3*4"  ).to_yield(14) }

#[test] fn divadd2_3_4()     { expect( "30/2*3"  ).to_yield(45) }
#[test] fn divadd2_3_4_neg() { expect( "-30/2*3" ).to_yield(-45) }
#[test] fn divadd2_3_4_pos() { expect( "+30/2*3" ).to_yield(45) }

#[test] fn adddiv3_8_2()     { expect( "3+8/2"   ).to_yield(7) }
#[test] fn adddiv3_8_2_neg() { expect( "-3+8/2"  ).to_yield(1) }
#[test] fn adddiv3_8_2_pos() { expect( "+3+8/2"  ).to_yield(7) }

#[test] fn addsub0_0_0()     { expect( "0+0-0"   ).to_yield(0) }
#[test] fn addsub0_0_0_neg() { expect( "-0+0-0"  ).to_yield(0) }
#[test] fn addsub0_0_0_pos() { expect( "+0+0-0"  ).to_yield(0) }
#[test] fn addsub1_2_3()     { expect( "1+2-3"   ).to_yield(0) }
#[test] fn addsub1_2_3_neg() { expect( "-1+2-3"  ).to_yield(-2) }
#[test] fn addsub1_2_3_pos() { expect( "+1+2-3"  ).to_yield(0) }

#[test] fn subadd0_0_0()     { expect( "0-0+0"   ).to_yield(0) }
#[test] fn subadd0_0_0_neg() { expect( "-0-0+0"  ).to_yield(0) }
#[test] fn subadd0_0_0_pos() { expect( "+0-0+0"  ).to_yield(0) }
#[test] fn subadd1_2_3()     { expect( "1-2+3"   ).to_yield(2) }
#[test] fn subadd1_2_3_neg() { expect( "-1-2+3"  ).to_yield(0) }
#[test] fn subadd1_2_3_pos() { expect( "+1-2+3"  ).to_yield(2) }

#[test] fn neg_0()           { expect( "-0"      ).to_yield(0) }
#[test] fn neg_1()           { expect( "-1"      ).to_yield(-1) }
#[test] fn pos_0()           { expect( "+0"      ).to_yield(0) }
#[test] fn pos_1()           { expect( "+1"      ).to_yield(1) }

#[test] fn plusneg_5_2()     { expect( "5 + -2"  ).to_yield(3) }
#[test] fn timesneg_5_2()    { expect( "5 * -2"  ).to_yield(-10) }
