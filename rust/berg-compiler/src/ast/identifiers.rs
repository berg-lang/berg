use ast::IdentifierIndex;
use ast::intern_pool::*;

pub(crate) const ALL_IDENTIFIERS: [(IdentifierIndex,&str);25] = [
    (EMPTY_STRING,""),
    (STAR,"*"),
    (SLASH,"/"),
    (PLUS,"+"),
    (DASH,"-"),
    (OPEN_PAREN,"("),
    (CLOSE_PAREN,")"),
    (OPEN_CURLY,"{"),
    (CLOSE_CURLY,"}"),
    (SEMICOLON, ";"),
    (AND_AND,"&&"),
    (OR_OR,"||"),
    (EXCLAMATION_POINT,"!"),
    (DOUBLE_EXCLAMATION_POINT,"!!"),
    (EQUAL_TO,"=="),
    (NOT_EQUAL_TO,"!="),
    (GREATER_THAN,">"),
    (LESS_THAN,"<"),
    (GREATER_EQUAL,">="),
    (LESS_EQUAL,"<="),
    (PLUS_PLUS,"++"),
    (DASH_DASH,"--"),
    (COLON,":"),
    (TRUE,"true"),
    (FALSE,"false"),
];
pub(crate) const EMPTY_STRING: IdentifierIndex = IdentifierIndex(0);
pub(crate) const STAR: IdentifierIndex = IdentifierIndex(1);
pub(crate) const SLASH: IdentifierIndex = IdentifierIndex(2);
pub(crate) const PLUS: IdentifierIndex = IdentifierIndex(3);
pub(crate) const DASH: IdentifierIndex = IdentifierIndex(4);
pub(crate) const OPEN_PAREN: IdentifierIndex = IdentifierIndex(5);
pub(crate) const CLOSE_PAREN: IdentifierIndex = IdentifierIndex(6);
pub(crate) const OPEN_CURLY: IdentifierIndex = IdentifierIndex(7);
pub(crate) const CLOSE_CURLY: IdentifierIndex = IdentifierIndex(8);
pub(crate) const SEMICOLON: IdentifierIndex = IdentifierIndex(9);
pub(crate) const AND_AND: IdentifierIndex = IdentifierIndex(10);
pub(crate) const OR_OR: IdentifierIndex = IdentifierIndex(11);
pub(crate) const EXCLAMATION_POINT: IdentifierIndex = IdentifierIndex(12);
pub(crate) const DOUBLE_EXCLAMATION_POINT: IdentifierIndex = IdentifierIndex(13);
pub(crate) const EQUAL_TO: IdentifierIndex = IdentifierIndex(14);
pub(crate) const NOT_EQUAL_TO: IdentifierIndex = IdentifierIndex(15);
pub(crate) const GREATER_THAN: IdentifierIndex = IdentifierIndex(16);
pub(crate) const LESS_THAN: IdentifierIndex = IdentifierIndex(17);
pub(crate) const GREATER_EQUAL: IdentifierIndex = IdentifierIndex(18);
pub(crate) const LESS_EQUAL: IdentifierIndex = IdentifierIndex(19);
pub(crate) const PLUS_PLUS: IdentifierIndex = IdentifierIndex(20);
pub(crate) const DASH_DASH: IdentifierIndex = IdentifierIndex(21);
pub(crate) const COLON: IdentifierIndex = IdentifierIndex(22);
pub(crate) const TRUE: IdentifierIndex = IdentifierIndex(23);
pub(crate) const FALSE: IdentifierIndex = IdentifierIndex(24);

pub(crate) fn intern_all() -> InternPool<IdentifierIndex> {
    let mut identifiers = InternPool::default();
    for operator in &ALL_IDENTIFIERS {
        let (operator,string) = *operator;
        let actual_identifier = identifiers.add(string);
        assert_eq!(actual_identifier, operator);
    }
    assert_eq!(identifiers.len(), ALL_IDENTIFIERS.len());
    identifiers
}

pub(crate) fn identifier_string(identifier: IdentifierIndex) -> &'static str {
    ALL_IDENTIFIERS[identifier.0 as usize].1
}