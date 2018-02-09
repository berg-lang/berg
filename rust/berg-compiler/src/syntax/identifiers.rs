use syntax::IdentifierIndex;
use util::intern_pool::*;

const ALL_IDENTIFIERS: [(IdentifierIndex, &str); LEN] = [
    (NOT_AN_IDENTIFIER, "<not an identifier>"),
    (EMPTY_STRING, ""),
    (STAR, "*"),
    (SLASH, "/"),
    (PLUS, "+"),
    (DASH, "-"),
    (OPEN_PAREN, "("),
    (CLOSE_PAREN, ")"),
    (OPEN_CURLY, "{"),
    (CLOSE_CURLY, "}"),
    (SEMICOLON, ";"),
    (AND_AND, "&&"),
    (OR_OR, "||"),
    (EXCLAMATION_POINT, "!"),
    (DOUBLE_EXCLAMATION_POINT, "!!"),
    (EQUAL_TO, "=="),
    (NOT_EQUAL_TO, "!="),
    (GREATER_THAN, ">"),
    (LESS_THAN, "<"),
    (GREATER_EQUAL, ">="),
    (LESS_EQUAL, "<="),
    (PLUS_PLUS, "++"),
    (DASH_DASH, "--"),
    (COLON, ":"),
    (TRUE, "true"),
    (FALSE, "false"),
    (FIELDS, "Fields"),
    (OPERATORS, "Operators"),
    (PREFIX_OPERATORS, "PrefixOperators"),
    (SUFFIX_OPERATORS, "SuffixOperators"),
    (CALL, "Call"),
    (NEWLINE, "\n"),
    (NOTHING, "nothing"),
    (DOT, "."),
    (SPACE, " "),
];

pub const NOT_AN_IDENTIFIER: IdentifierIndex = IdentifierIndex(0);
pub const EMPTY_STRING: IdentifierIndex = IdentifierIndex(1);
pub const STAR: IdentifierIndex = IdentifierIndex(2);
pub const SLASH: IdentifierIndex = IdentifierIndex(3);
pub const PLUS: IdentifierIndex = IdentifierIndex(4);
pub const DASH: IdentifierIndex = IdentifierIndex(5);
pub const OPEN_PAREN: IdentifierIndex = IdentifierIndex(6);
pub const CLOSE_PAREN: IdentifierIndex = IdentifierIndex(7);
pub const OPEN_CURLY: IdentifierIndex = IdentifierIndex(8);
pub const CLOSE_CURLY: IdentifierIndex = IdentifierIndex(9);
pub const SEMICOLON: IdentifierIndex = IdentifierIndex(10);
pub const AND_AND: IdentifierIndex = IdentifierIndex(11);
pub const OR_OR: IdentifierIndex = IdentifierIndex(12);
pub const EXCLAMATION_POINT: IdentifierIndex = IdentifierIndex(13);
pub const DOUBLE_EXCLAMATION_POINT: IdentifierIndex = IdentifierIndex(14);
pub const EQUAL_TO: IdentifierIndex = IdentifierIndex(15);
pub const NOT_EQUAL_TO: IdentifierIndex = IdentifierIndex(16);
pub const GREATER_THAN: IdentifierIndex = IdentifierIndex(17);
pub const LESS_THAN: IdentifierIndex = IdentifierIndex(18);
pub const GREATER_EQUAL: IdentifierIndex = IdentifierIndex(19);
pub const LESS_EQUAL: IdentifierIndex = IdentifierIndex(20);
pub const PLUS_PLUS: IdentifierIndex = IdentifierIndex(21);
pub const DASH_DASH: IdentifierIndex = IdentifierIndex(22);
pub const COLON: IdentifierIndex = IdentifierIndex(23);
pub const TRUE: IdentifierIndex = IdentifierIndex(24);
pub const FALSE: IdentifierIndex = IdentifierIndex(25);
pub const FIELDS: IdentifierIndex = IdentifierIndex(26);
pub const OPERATORS: IdentifierIndex = IdentifierIndex(27);
pub const PREFIX_OPERATORS: IdentifierIndex = IdentifierIndex(28);
pub const SUFFIX_OPERATORS: IdentifierIndex = IdentifierIndex(29);
pub const CALL: IdentifierIndex = IdentifierIndex(30);
pub const NEWLINE: IdentifierIndex = IdentifierIndex(31);
pub const NOTHING: IdentifierIndex = IdentifierIndex(32);
pub const DOT: IdentifierIndex = IdentifierIndex(33);
pub const SPACE: IdentifierIndex = IdentifierIndex(34);
pub const LEN: usize = 35;

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
