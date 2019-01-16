pub mod compiler_test;
use crate::compiler_test::*;

// booleans
#[test]
fn equal_true_true()           { expect( "true==true"   ).to_yield(true) }
#[test]
fn equal_true_false()          { expect( "true==false"  ).to_yield(false) }
#[test]
fn equal_false_true()          { expect( "false==true"  ).to_yield(false) }
#[test]
fn equal_false_false()         { expect( "false==false" ).to_yield(true) }

#[test]
fn not_equal_true_true()       { expect( "true!=true"   ).to_yield(false) }
#[test]
fn not_equal_true_false()      { expect( "true!=false"  ).to_yield(true) }
#[test]
fn not_equal_false_true()      { expect( "false!=true"  ).to_yield(true) }
#[test]
fn not_equal_false_false()     { expect( "false!=false" ).to_yield(false) }

// numbers
#[test]
fn equal_0_0()                 { expect( "0==0"         ).to_yield(true) }
#[test]
fn equal_1_1()                 { expect( "1==1"         ).to_yield(true) }
#[test]
fn equal_1_0()                 { expect( "1==0"         ).to_yield(false) }
#[test]
fn equal_0_1()                 { expect( "0==1"         ).to_yield(false) }
#[test]
fn equal_1_2()                 { expect( "1==2"         ).to_yield(false) }
#[test]
fn equal_big_big()             { expect( "1982648372164176312796419487198==1982648372164176312796419487198" ).to_yield(true) }
#[test]
fn equal_big_big2()            { expect( "1982648372164176312796419487198==99127934712648732164276347216429663" ).to_yield(false) }
#[test]
fn equal_big2_big()            { expect( "99127934712648732164276347216429663==1982648372164176312796419487198" ).to_yield(false) }

#[test]
fn not_equal_0_0()             { expect( "0!=0"         ).to_yield(false) }
#[test]
fn not_equal_1_1()             { expect( "1!=1"         ).to_yield(false) }
#[test]
fn not_equal_1_0()             { expect( "1!=0"         ).to_yield(true) }
#[test]
fn not_equal_0_1()             { expect( "0!=1"         ).to_yield(true) }
#[test]
fn not_equal_1_2()             { expect( "1!=2"         ).to_yield(true) }
#[test]
fn not_equal_big_big()         { expect( "1982648372164176312796419487198!=1982648372164176312796419487198" ).to_yield(false) }
#[test]
fn not_equal_big_big2()        { expect( "1982648372164176312796419487198!=99127934712648732164276347216429663" ).to_yield(true) }
#[test]
fn not_equal_big2_big()        { expect( "99127934712648732164276347216429663!=1982648372164176312796419487198" ).to_yield(true) }

// nothing
#[test]
fn equal_nothing_nothing()     { expect( "()==()"       ).to_yield(true) }
#[test]
fn not_equal_nothing_nothing() { expect( "()!=()"       ).to_yield(false) }

// errors
#[test]
fn equal_error_error()         { expect( "1/0==1/0"     ).to_error(DivideByZero,2) }
#[test]
fn not_equal_error_error()     { expect( "1/0!=1/0"     ).to_error(DivideByZero,2) }

// nothing/boolean
#[test]
fn equal_nothing_true()        { expect( "()==true"     ).to_yield(false) }
#[test]
fn equal_nothing_false()       { expect( "()==false"    ).to_yield(false) }
#[test]
fn equal_true_nothing()        { expect( "true==()"     ).to_yield(false) }
#[test]
fn equal_false_nothing()       { expect( "false==()"    ).to_yield(false) }
#[test]
fn not_equal_nothing_true()    { expect( "()!=true"     ).to_yield(true) }
#[test]
fn not_equal_nothing_false()   { expect( "()!=false"    ).to_yield(true) }
#[test]
fn not_equal_true_nothing()    { expect( "true!=()"     ).to_yield(true) }
#[test]
fn not_equal_false_nothing()   { expect( "false!=()"    ).to_yield(true) }

// nothing/number
#[test]
fn equal_nothing_0()           { expect( "()==0"        ).to_yield(false) }
#[test]
fn equal_nothing_1()           { expect( "()==1"        ).to_yield(false) }
#[test]
fn equal_0_nothing()           { expect( "0==()"        ).to_yield(false) }
#[test]
fn equal_1_nothing()           { expect( "1==()"        ).to_yield(false) }
#[test]
fn not_equal_nothing_0()       { expect( "()!=0"        ).to_yield(true) }
#[test]
fn not_equal_nothing_1()       { expect( "()!=1"        ).to_yield(true) }
#[test]
fn not_equal_0_nothing()       { expect( "0!=()"        ).to_yield(true) }
#[test]
fn not_equal_1_nothing()       { expect( "1!=()"        ).to_yield(true) }

// error/boolean
#[test]
fn equal_error_true()          { expect( "1/0==true"    ).to_error(DivideByZero,2) }
#[test]
fn equal_error_false()         { expect( "1/0==false"   ).to_error(DivideByZero,2) }
#[test]
fn equal_true_error()          { expect( "true==1/0"    ).to_error(DivideByZero,8) }
#[test]
fn equal_false_error()         { expect( "false==1/0"   ).to_error(DivideByZero,9) }
#[test]
fn not_equal_error_true()      { expect( "1/0!=true"    ).to_error(DivideByZero,2) }
#[test]
fn not_equal_error_false()     { expect( "1/0!=false"   ).to_error(DivideByZero,2) }
#[test]
fn not_equal_true_error()      { expect( "true!=1/0"    ).to_error(DivideByZero,8) }
#[test]
fn not_equal_false_error()     { expect( "false!=1/0"   ).to_error(DivideByZero,9) }

// error/number
#[test]
fn equal_error_0()             { expect( "1/0==0"       ).to_error(DivideByZero,2) }
#[test]
fn equal_error_1()             { expect( "1/0==1"       ).to_error(DivideByZero,2) }
#[test]
fn equal_0_error()             { expect( "0==1/0"       ).to_error(DivideByZero,5) }
#[test]
fn equal_1_error()             { expect( "1==1/0"       ).to_error(DivideByZero,5) }
#[test]
fn not_equal_error_0()         { expect( "1/0!=0"       ).to_error(DivideByZero,2) }
#[test]
fn not_equal_error_1()         { expect( "1/0!=1"       ).to_error(DivideByZero,2) }
#[test]
fn not_equal_0_error()         { expect( "0!=1/0"       ).to_error(DivideByZero,5) }
#[test]
fn not_equal_1_error()         { expect( "1!=1/0"       ).to_error(DivideByZero,5) }

// number/boolean
#[test]
fn equal_true_1()              { expect( "true==1"      ).to_yield(false) }
#[test]
fn equal_true_0()              { expect( "true==0"      ).to_yield(false) }
#[test]
fn equal_false_1()             { expect( "false==1"     ).to_yield(false) }
#[test]
fn equal_false_0()             { expect( "false==0"     ).to_yield(false) }
#[test]
fn equal_1_true()              { expect( "1==true"      ).to_yield(false) }
#[test]
fn equal_0_true()              { expect( "0==true"      ).to_yield(false) }
#[test]
fn equal_1_false()             { expect( "1==false"     ).to_yield(false) }
#[test]
fn equal_0_false()             { expect( "0==false"     ).to_yield(false) }
#[test]
fn not_equal_true_1()          { expect( "true!=1"      ).to_yield(true) }
#[test]
fn not_equal_true_0()          { expect( "true!=0"      ).to_yield(true) }
#[test]
fn not_equal_false_1()         { expect( "false!=1"     ).to_yield(true) }
#[test]
fn not_equal_false_0()         { expect( "false!=0"     ).to_yield(true) }
#[test]
fn not_equal_1_true()          { expect( "1!=true"      ).to_yield(true) }
#[test]
fn not_equal_0_true()          { expect( "0!=true"      ).to_yield(true) }
#[test]
fn not_equal_1_false()         { expect( "1!=false"     ).to_yield(true) }
#[test]
fn not_equal_0_false()         { expect( "0!=false"     ).to_yield(true) }
