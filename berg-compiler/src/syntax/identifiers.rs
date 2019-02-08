use crate::syntax::IdentifierIndex;
use crate::util::intern_pool::*;

macro_rules! identifiers {
    { $($name:ident = $str:expr),* } => {
        const ALL_IDENTIFIERS: [(IdentifierIndex, &str); IdentifierIndices::LEN as usize] = [
            $( ($name, $str) ),*
        ];
        // We use this enum to determine the indices
        #[allow(non_camel_case_types)]
        enum IdentifierIndices {
            $($name),*,
            LEN
        }
        $(
            pub const $name: IdentifierIndex = IdentifierIndex(IdentifierIndices::$name as u32);
        )*
        pub const LEN: usize = IdentifierIndices::LEN as usize;
    }
}
identifiers! {
    NOT_AN_IDENTIFIER = "<not an identifier>",
    EMPTY_STRING = "",
    STAR = "*",
    SLASH = "/",
    PLUS = "+",
    DASH = "-",
    OPEN_PAREN = "(",
    CLOSE_PAREN = ")",
    OPEN_CURLY = "{",
    CLOSE_CURLY = "}",
    SEMICOLON = ";",
    AND_AND = "&&",
    OR_OR = "||",
    EXCLAMATION_POINT = "!",
    DOUBLE_EXCLAMATION_POINT = "!!",
    EQUAL_TO = "==",
    NOT_EQUAL_TO = "!=",
    GREATER_THAN = ">",
    LESS_THAN = "<",
    GREATER_EQUAL = ">=",
    LESS_EQUAL = "<=",
    PLUS_PLUS = "++",
    DASH_DASH = "--",
    COLON = ":",
    TRUE = "true",
    FALSE = "false",
    FIELDS = "Fields",
    OPERATORS = "Operators",
    PREFIX_OPERATORS = "PrefixOperators",
    SUFFIX_OPERATORS = "SuffixOperators",
    APPLY = "Call",
    NEWLINE = "\n",
    NOTHING = "nothing",
    NEXT = "Next",
    DOT = ".",
    SPACE = " ",
    COMMA = ","
}

pub(crate) fn intern_all() -> InternPool<IdentifierIndex> {
    let mut identifiers = InternPool::default();
    for operator in ALL_IDENTIFIERS.iter() {
        let (operator, string) = *operator;
        let actual_identifier = identifiers.add(string);
        assert_eq!(actual_identifier, operator);
    }
    assert_eq!(identifiers.len(), ALL_IDENTIFIERS.len());
    identifiers
}

pub fn identifier_string(identifier: IdentifierIndex) -> &'static str {
    ALL_IDENTIFIERS[identifier.0 as usize].1
}
