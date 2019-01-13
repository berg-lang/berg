pub mod compiler_test;
use compiler_test::*;

#[test] fn true_literal()    { expect( "true"  ).to_yield(true) }
#[test] fn false_literal()   { expect( "false" ).to_yield(false) }
#[test] fn uppercase_true()  { expect( "TRUE"  ).to_error(NoSuchField,0..=3) }
#[test] fn uppercase_false() { expect( "FALSE" ).to_error(NoSuchField,0..=4) }
