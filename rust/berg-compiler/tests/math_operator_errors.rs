pub mod compiler_test;
use compiler_test::*;

#[test]
fn addmul_missing_operator_precedence() { expect( "1 * + 3"     ).to_error(MissingOperand,2) }
#[test]
fn muladd_missing_operator_precedence() { expect( "1 + * 3"     ).to_error(MissingOperand,4) }
#[test]
fn addparen_missing_operator_precedence() { expect( "(1 + )"      ).to_error(MissingOperand,3) }
#[test]
fn parenadd_missing_operator_precedence() { expect( "( + 1)"      ).to_error(MissingOperand,2) }

#[test]
fn div0_0()              { expect( "0/0"         ).to_error(DivideByZero,2) }
#[test]
fn div1_0()              { expect( "1/0"         ).to_error(DivideByZero,2) }

#[test]
fn trailing_neg()        { expect( "0-"          ).to_error(UnsupportedOperator,1) }
#[test]
fn trailing_pos()        { expect( "0+"          ).to_error(UnsupportedOperator,1) }
#[test]
fn sub_only()            { expect( "-"           ).to_error(MissingOperand,0) }
#[test]
fn add_only()            { expect( "+"           ).to_error(MissingOperand,0) }
#[test]
fn plus_minus()          { expect( "1+-2"        ).to_error(UnsupportedOperator,1..=2) }

// + errors
#[test]
fn add_true_1()          { expect( "true+1"      ).to_error(UnsupportedOperator,4) }
#[test]
fn add_1_true()          { expect( "1+true"      ).to_error(BadType,2..=5) }
#[test]
fn add_false_1()         { expect( "false+1"     ).to_error(UnsupportedOperator,5) }
#[test]
fn add_1_false()         { expect( "1+false"     ).to_error(BadType,2..=6) }
#[test]
fn add_nothing_1()       { expect( "()+1"        ).to_error(UnsupportedOperator,2) }
#[test]
fn add_1_nothing()       { expect( "1+()"        ).to_error(BadType,2..=3) }
#[test]
fn add_error_1()         { expect( "1/0+1"       ).to_error(DivideByZero,2) }
#[test]
fn add_1_error()         { expect( "1+1/0"       ).to_error(DivideByZero,4) }

#[test]
fn add_true_true()       { expect( "true+true"   ).to_error(UnsupportedOperator,4) }
#[test]
fn add_false_true()      { expect( "false+true"  ).to_error(UnsupportedOperator,5) }
#[test]
fn add_true_false()      { expect( "true+false"  ).to_error(UnsupportedOperator,4) }
#[test]
fn add_false_false()     { expect( "false+false" ).to_error(UnsupportedOperator,5) }
#[test]
fn add_true_error()      { expect( "true+1/0"    ).to_error(UnsupportedOperator,4) }
#[test]
fn add_error_true()      { expect( "1/0+true"    ).to_error(DivideByZero,2) }
#[test]
fn add_false_error()     { expect( "false+1/0"   ).to_error(UnsupportedOperator,5) }
#[test]
fn add_error_false()     { expect( "1/0+false"   ).to_error(DivideByZero,2) }
#[test]
fn add_true_nothing()    { expect( "true+()"     ).to_error(UnsupportedOperator,4) }
#[test]
fn add_nothing_true()    { expect( "()+true"     ).to_error(UnsupportedOperator,2) }
#[test]
fn add_false_nothing()   { expect( "false+()"    ).to_error(UnsupportedOperator,5) }
#[test]
fn add_nothing_false()   { expect( "()+false"    ).to_error(UnsupportedOperator,2) }

#[test]
fn add_error_error()     { expect( "1/0+1/0"     ).to_error(DivideByZero,2) }
#[test]
fn add_error_nothing()   { expect( "1/0+()"      ).to_error(DivideByZero,2) }
#[test]
fn add_nothing_error()   { expect( "()+1/0"      ).to_error(UnsupportedOperator,2) }
#[test]
fn add_nothing_nothing() { expect( "()+()"       ).to_error(UnsupportedOperator,2) }

// - errors
#[test]
fn sub_true_1()          { expect( "true-1"      ).to_error(UnsupportedOperator,4) }
#[test]
fn sub_1_true()          { expect( "1-true"      ).to_error(BadType,2..=5) }
#[test]
fn sub_false_1()         { expect( "false-1"     ).to_error(UnsupportedOperator,5) }
#[test]
fn sub_1_false()         { expect( "1-false"     ).to_error(BadType,2..=6) }
#[test]
fn sub_nothing_1()       { expect( "()-1"        ).to_error(UnsupportedOperator,2) }
#[test]
fn sub_1_nothing()       { expect( "1-()"        ).to_error(BadType,2..=3) }
#[test]
fn sub_error_1()         { expect( "1/0-1"       ).to_error(DivideByZero,2) }
#[test]
fn sub_1_error()         { expect( "1-1/0"       ).to_error(DivideByZero,4) }

#[test]
fn sub_true_true()       { expect( "true-true"   ).to_error(UnsupportedOperator,4) }
#[test]
fn sub_false_true()      { expect( "false-true"  ).to_error(UnsupportedOperator,5) }
#[test]
fn sub_true_false()      { expect( "true-false"  ).to_error(UnsupportedOperator,4) }
#[test]
fn sub_false_false()     { expect( "false-false" ).to_error(UnsupportedOperator,5) }
#[test]
fn sub_true_error()      { expect( "true-1/0"    ).to_error(UnsupportedOperator,4) }
#[test]
fn sub_error_true()      { expect( "1/0-true"    ).to_error(DivideByZero,2) }
#[test]
fn sub_false_error()     { expect( "false-1/0"   ).to_error(UnsupportedOperator,5) }
#[test]
fn sub_error_false()     { expect( "1/0-false"   ).to_error(DivideByZero,2) }
#[test]
fn sub_true_nothing()    { expect( "true-()"     ).to_error(UnsupportedOperator,4) }
#[test]
fn sub_nothing_true()    { expect( "()-true"     ).to_error(UnsupportedOperator,2) }
#[test]
fn sub_false_nothing()   { expect( "false-()"    ).to_error(UnsupportedOperator,5) }
#[test]
fn sub_nothing_false()   { expect( "()-false"    ).to_error(UnsupportedOperator,2) }

#[test]
fn sub_error_error()     { expect( "1/0-1/0"     ).to_error(DivideByZero,2) }
#[test]
fn sub_error_nothing()   { expect( "1/0-()"      ).to_error(DivideByZero,2) }
#[test]
fn sub_nothing_error()   { expect( "()-1/0"      ).to_error(UnsupportedOperator,2) }
#[test]
fn sub_nothing_nothing() { expect( "()-()"       ).to_error(UnsupportedOperator,2) }


// * errors
#[test]
fn mul_true_1()          { expect( "true*1"      ).to_error(UnsupportedOperator,4) }
#[test]
fn mul_1_true()          { expect( "1*true"      ).to_error(BadType,2..=5) }
#[test]
fn mul_false_1()         { expect( "false*1"     ).to_error(UnsupportedOperator,5) }
#[test]
fn mul_1_false()         { expect( "1*false"     ).to_error(BadType,2..=6) }
#[test]
fn mul_nothing_1()       { expect( "()*1"        ).to_error(UnsupportedOperator,2) }
#[test]
fn mul_1_nothing()       { expect( "1*()"        ).to_error(BadType,2..=3) }
#[test]
fn mul_error_1()         { expect( "(1/0)*1"     ).to_error(DivideByZero,3) }
#[test]
fn mul_1_error()         { expect( "1*(1/0)"     ).to_error(DivideByZero,5) }

#[test]
fn mul_true_true()       { expect( "true*true"   ).to_error(UnsupportedOperator,4) }
#[test]
fn mul_false_true()      { expect( "false*true"  ).to_error(UnsupportedOperator,5) }
#[test]
fn mul_true_false()      { expect( "true*false"  ).to_error(UnsupportedOperator,4) }
#[test]
fn mul_false_false()     { expect( "false*false" ).to_error(UnsupportedOperator,5) }
#[test]
fn mul_true_error()      { expect( "true*(1/0)"  ).to_error(UnsupportedOperator,4) }
#[test]
fn mul_error_true()      { expect( "(1/0)*true"  ).to_error(DivideByZero,3) }
#[test]
fn mul_false_error()     { expect( "false*(1/0)" ).to_error(UnsupportedOperator,5) }
#[test]
fn mul_error_false()     { expect( "(1/0)*false" ).to_error(DivideByZero,3) }
#[test]
fn mul_true_nothing()    { expect( "true*()"     ).to_error(UnsupportedOperator,4) }
#[test]
fn mul_nothing_true()    { expect( "()*true"     ).to_error(UnsupportedOperator,2) }
#[test]
fn mul_false_nothing()   { expect( "false*()"    ).to_error(UnsupportedOperator,5) }
#[test]
fn mul_nothing_false()   { expect( "()*false"    ).to_error(UnsupportedOperator,2) }

#[test]
fn mul_error_error()     { expect( "(1/0)*(1/0)" ).to_error(DivideByZero,3) }
#[test]
fn mul_error_nothing()   { expect( "(1/0)*()"    ).to_error(DivideByZero,3) }
#[test]
fn mul_nothing_error()   { expect( "()*(1/0)"    ).to_error(UnsupportedOperator,2) }
#[test]
fn mul_nothing_nothing() { expect( "()*()"       ).to_error(UnsupportedOperator,2) }

// / errors
#[test]
fn div_true_1()          { expect( "true/1"      ).to_error(UnsupportedOperator,4) }
#[test]
fn div_1_true()          { expect( "1/true"      ).to_error(BadType,2..=5) }
#[test]
fn div_false_1()         { expect( "false/1"     ).to_error(UnsupportedOperator,5) }
#[test]
fn div_1_false()         { expect( "1/false"     ).to_error(BadType,2..=6) }
#[test]
fn div_nothing_1()       { expect( "()/1"        ).to_error(UnsupportedOperator,2) }
#[test]
fn div_1_nothing()       { expect( "1/()"        ).to_error(BadType,2..=3) }
#[test]
fn div_error_1()         { expect( "(1/0)/1"     ).to_error(DivideByZero,3) }
#[test]
fn div_1_error()         { expect( "1/(1/0)"     ).to_error(DivideByZero,5) }

#[test]
fn div_true_true()       { expect( "true/true"   ).to_error(UnsupportedOperator,4) }
#[test]
fn div_false_true()      { expect( "false/true"  ).to_error(UnsupportedOperator,5) }
#[test]
fn div_true_false()      { expect( "true/false"  ).to_error(UnsupportedOperator,4) }
#[test]
fn div_false_false()     { expect( "false/false" ).to_error(UnsupportedOperator,5) }
#[test]
fn div_true_error()      { expect( "true/(1/0)"  ).to_error(UnsupportedOperator,4) }
#[test]
fn div_error_true()      { expect( "(1/0)/true"  ).to_error(DivideByZero,3) }
#[test]
fn div_false_error()     { expect( "false/(1/0)" ).to_error(UnsupportedOperator,5) }
#[test]
fn div_error_false()     { expect( "(1/0)/false" ).to_error(DivideByZero,3) }
#[test]
fn div_true_nothing()    { expect( "true/()"     ).to_error(UnsupportedOperator,4) }
#[test]
fn div_nothing_true()    { expect( "()/true"     ).to_error(UnsupportedOperator,2) }
#[test]
fn div_false_nothing()   { expect( "false/()"    ).to_error(UnsupportedOperator,5) }
#[test]
fn div_nothing_false()   { expect( "()/false"    ).to_error(UnsupportedOperator,2) }

#[test]
fn div_error_error()     { expect( "(1/0)/(1/0)" ).to_error(DivideByZero,3) }
#[test]
fn div_error_nothing()   { expect( "(1/0)/()"    ).to_error(DivideByZero,3) }
#[test]
fn div_nothing_error()   { expect( "()/(1/0)"    ).to_error(UnsupportedOperator,2) }
#[test]
fn div_nothing_nothing() { expect( "()/()"       ).to_error(UnsupportedOperator,2) }

// Negative - errors
#[test]
fn neg_true()            { expect( "-true"       ).to_error(UnsupportedOperator,0) }
#[test]
fn neg_false()           { expect( "-false"      ).to_error(UnsupportedOperator,0) }
#[test]
fn neg_nothing()         { expect( "-()"         ).to_error(UnsupportedOperator,0) }
#[test]
fn neg_error()           { expect( "-(1/0)"      ).to_error(DivideByZero,4) }

// Positive + errors
#[test]
fn pos_true()            { expect( "+true"       ).to_error(UnsupportedOperator,0) }
#[test]
fn pos_false()           { expect( "+false"      ).to_error(UnsupportedOperator,0) }
#[test]
fn pos_nothing()         { expect( "+()"         ).to_error(UnsupportedOperator,0) }
#[test]
fn pos_error()           { expect( "+(1/0)"      ).to_error(DivideByZero,4) }
