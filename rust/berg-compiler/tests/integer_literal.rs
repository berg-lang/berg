pub mod compiler_test;
use crate::compiler_test::*;

#[test]
fn zero() { expect( "0"                                             ).to_yield(0) }
#[test]
fn one()  { expect( "1"                                             ).to_yield(1) }
#[test]
fn huge() { expect( "999999999999999999999999999999999999999999999" ).to_yield(BigRational::from_str("999999999999999999999999999999999999999999999").unwrap()) }
