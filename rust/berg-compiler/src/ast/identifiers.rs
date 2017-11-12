use ast::IdentifierIndex;
use ast::intern_pool::*;

const ALL_OPERATORS: [(IdentifierIndex,&str);9] = [
    (STAR,"*"),
    (SLASH,"/"),
    (PLUS,"+"),
    (DASH,"-"),
    (OPEN_PAREN,"("),
    (CLOSE_PAREN,")"),
    (NOTHING,"nothing"),
    (TRUE,"true"),
    (FALSE,"false"),
];
pub const STAR: IdentifierIndex = IdentifierIndex(0);
pub const SLASH: IdentifierIndex = IdentifierIndex(1);
pub const PLUS: IdentifierIndex = IdentifierIndex(2);
pub const DASH: IdentifierIndex = IdentifierIndex(3);
pub const OPEN_PAREN: IdentifierIndex = IdentifierIndex(4);
pub const CLOSE_PAREN: IdentifierIndex = IdentifierIndex(5);
pub const NOTHING: IdentifierIndex = IdentifierIndex(6);
pub const TRUE: IdentifierIndex = IdentifierIndex(7);
pub const FALSE: IdentifierIndex = IdentifierIndex(8);

pub(crate) fn intern_all() -> InternPool<IdentifierIndex> {
    let mut identifiers = InternPool::default();
    for operator in &ALL_OPERATORS {
        let (operator,string) = *operator;
        let actual_identifier = identifiers.add(string);
        assert_eq!(actual_identifier, operator);
    }
    assert_eq!(identifiers.len(), ALL_OPERATORS.len());
    identifiers
}
