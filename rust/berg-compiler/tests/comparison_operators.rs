pub mod compiler_test;
use crate::compiler_test::*;

// numbers
#[test]
fn greater_than_0_0()          { expect( "0>0"                                                                  ).to_yield(false) }
#[test]
fn greater_than_1_1()          { expect( "1>1"                                                                  ).to_yield(false) }
#[test]
fn greater_than_1_0()          { expect( "1>0"                                                                  ).to_yield(true) }
#[test]
fn greater_than_0_1()          { expect( "0>1"                                                                  ).to_yield(false) }
#[test]
fn greater_than_1_2()          { expect( "1>2"                                                                  ).to_yield(false) }
#[test]
fn greater_than_big_big()      { expect( "1982648372164176312796419487198>1982648372164176312796419487198"      ).to_yield(false) }
#[test]
fn greater_than_big_big2()     { expect( "1982648372164176312796419487198>99127934712648732164276347216429663"  ).to_yield(false) }
#[test]
fn greater_than_big2_big()     { expect( "99127934712648732164276347216429663>1982648372164176312796419487198"  ).to_yield(true) }

#[test]
fn greater_or_equal_0_0()      { expect( "0>=0"                                                                 ).to_yield(true) }
#[test]
fn greater_or_equal_1_1()      { expect( "1>=1"                                                                 ).to_yield(true) }
#[test]
fn greater_or_equal_1_0()      { expect( "1>=0"                                                                 ).to_yield(true) }
#[test]
fn greater_or_equal_0_1()      { expect( "0>=1"                                                                 ).to_yield(false) }
#[test]
fn greater_or_equal_1_2()      { expect( "1>=2"                                                                 ).to_yield(false) }
#[test]
fn greater_or_equal_big_big()  { expect( "1982648372164176312796419487198>=1982648372164176312796419487198"     ).to_yield(true) }
#[test]
fn greater_or_equal_big_big2() { expect( "1982648372164176312796419487198>=99127934712648732164276347216429663" ).to_yield(false) }
#[test]
fn greater_or_equal_big2_big() { expect( "99127934712648732164276347216429663>=1982648372164176312796419487198" ).to_yield(true) }

#[test]
fn less_than_0_0()             { expect( "0<0"                                                                  ).to_yield(false) }
#[test]
fn less_than_1_1()             { expect( "1<1"                                                                  ).to_yield(false) }
#[test]
fn less_than_1_0()             { expect( "1<0"                                                                  ).to_yield(false) }
#[test]
fn less_than_0_1()             { expect( "0<1"                                                                  ).to_yield(true) }
#[test]
fn less_than_1_2()             { expect( "1<2"                                                                  ).to_yield(true) }
#[test]
fn less_than_big_big()         { expect( "1982648372164176312796419487198<1982648372164176312796419487198"      ).to_yield(false) }
#[test]
fn less_than_big_big2()        { expect( "1982648372164176312796419487198<99127934712648732164276347216429663"  ).to_yield(true) }
#[test]
fn less_than_big2_big()        { expect( "99127934712648732164276347216429663<1982648372164176312796419487198"  ).to_yield(false) }

#[test]
fn less_or_equal_0_0()         { expect( "0<=0"                                                                 ).to_yield(true) }
#[test]
fn less_or_equal_1_1()         { expect( "1<=1"                                                                 ).to_yield(true) }
#[test]
fn less_or_equal_1_0()         { expect( "1<=0"                                                                 ).to_yield(false) }
#[test]
fn less_or_equal_0_1()         { expect( "0<=1"                                                                 ).to_yield(true) }
#[test]
fn less_or_equal_1_2()         { expect( "1<=2"                                                                 ).to_yield(true) }
#[test]
fn less_or_equal_big_big()     { expect( "1982648372164176312796419487198<=1982648372164176312796419487198"     ).to_yield(true) }
#[test]
fn less_or_equal_big_big2()    { expect( "1982648372164176312796419487198<=99127934712648732164276347216429663" ).to_yield(true) }
#[test]
fn less_or_equal_big2_big()    { expect( "99127934712648732164276347216429663<=1982648372164176312796419487198" ).to_yield(false) }
