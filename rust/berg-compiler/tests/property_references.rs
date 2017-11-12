#[macro_use]
pub mod compiler_test;

compiler_tests! {
    no_such_property: "blah" => error(NoSuchProperty@[0-3]) type(error),
    property_starts_with_number: "1bla" => error(IdentifierStartsWithNumber@[0-3]) type(error),
    property_starts_with_underscore: "_bla" => error(NoSuchProperty@[0-3]) type(error),
    property_with_all_characters: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_" => error(NoSuchProperty@[0-62]) type(error),
    one_character_property: "b" => error(NoSuchProperty@0) type(error),
    underscore_only: "_" => error(NoSuchProperty@0) type(error),
}
