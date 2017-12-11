#[macro_use]
pub mod compiler_test;

compiler_tests! {
    no_such_field: "blah" => error(NoSuchField@[0-3]) type(error),
    field_starts_with_number: "1bla" => error(IdentifierStartsWithNumber@[0-3]) type(error),
    field_starts_with_underscore: "_bla" => error(NoSuchField@[0-3]) type(error),
    field_with_all_characters: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_" => error(NoSuchField@[0-62]) type(error),
    one_character_field: "b" => error(NoSuchField@0) type(error),
    underscore_only: "_" => error(NoSuchField@0) type(error),
}
