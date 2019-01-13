pub mod compiler_test;
use compiler_test::*;

#[test] fn no_such_field()                { expect( "blah"                                                            ).to_error(NoSuchField,0..=3) }
#[test] fn field_starts_with_number()     { expect( "1bla"                                                            ).to_error(IdentifierStartsWithNumber,0..=3) }
#[test] fn field_starts_with_underscore() { expect( "_bla"                                                            ).to_error(NoSuchField,0..=3) }
#[test] fn field_with_all_characters()    { expect( "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_" ).to_error(NoSuchField,0..=62) }
#[test] fn one_character_field()          { expect( "b"                                                               ).to_error(NoSuchField,0) }
#[test] fn underscore_only()              { expect( "_"                                                               ).to_error(NoSuchField,0) }
