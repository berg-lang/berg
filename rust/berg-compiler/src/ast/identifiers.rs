use ast::IdentifierIndex;
use ast::intern_pool::*;
use ast::precedence::Precedence;
use ast::precedence::Precedence::*;

const ALL_OPERATORS: [(IdentifierIndex,&str);27] = [
    (STAR,"*"),
    (SLASH,"/"),
    (PLUS,"+"),
    (DASH,"-"),
    (OPEN_PAREN,"("),
    (CLOSE_PAREN,")"),
    (SEMICOLON, ";"),
    (AND_AND,"&&"),
    (OR_OR,"||"),
    (NOT,"!"),
    (EQUAL_TO,"=="),
    (NOT_EQUAL_TO,"!="),
    (GREATER_THAN,">"),
    (LESS_THAN,"<"),
    (GREATER_EQUAL,">="),
    (LESS_EQUAL,"<="),
    (ASSIGN,"="),
    (ASSIGN_STAR,"*="),
    (ASSIGN_SLASH,"/="),
    (ASSIGN_PLUS,"+="),
    (ASSIGN_DASH,"-="),
    (ASSIGN_AND_AND,"&&="),
    (ASSIGN_OR_OR,"||="),
    (PLUS_PLUS,"++"),
    (DASH_DASH,"--"),
    (TRUE,"true"),
    (FALSE,"false"),
];
pub const STAR: IdentifierIndex = IdentifierIndex(0);
pub const SLASH: IdentifierIndex = IdentifierIndex(1);
pub const PLUS: IdentifierIndex = IdentifierIndex(2);
pub const DASH: IdentifierIndex = IdentifierIndex(3);
pub const OPEN_PAREN: IdentifierIndex = IdentifierIndex(4);
pub const CLOSE_PAREN: IdentifierIndex = IdentifierIndex(5);
pub const SEMICOLON: IdentifierIndex = IdentifierIndex(6);
pub const AND_AND: IdentifierIndex = IdentifierIndex(7);
pub const OR_OR: IdentifierIndex = IdentifierIndex(8);
pub const NOT: IdentifierIndex = IdentifierIndex(9);
pub const EQUAL_TO: IdentifierIndex = IdentifierIndex(10);
pub const NOT_EQUAL_TO: IdentifierIndex = IdentifierIndex(11);
pub const GREATER_THAN: IdentifierIndex = IdentifierIndex(12);
pub const LESS_THAN: IdentifierIndex = IdentifierIndex(13);
pub const GREATER_EQUAL: IdentifierIndex = IdentifierIndex(14);
pub const LESS_EQUAL: IdentifierIndex = IdentifierIndex(15);
pub const ASSIGN: IdentifierIndex = IdentifierIndex(16);
pub const ASSIGN_STAR: IdentifierIndex = IdentifierIndex(17);
pub const ASSIGN_SLASH: IdentifierIndex = IdentifierIndex(18);
pub const ASSIGN_PLUS: IdentifierIndex = IdentifierIndex(19);
pub const ASSIGN_DASH: IdentifierIndex = IdentifierIndex(20);
pub const ASSIGN_AND_AND: IdentifierIndex = IdentifierIndex(21);
pub const ASSIGN_OR_OR: IdentifierIndex = IdentifierIndex(22);
pub const PLUS_PLUS: IdentifierIndex = IdentifierIndex(23);
pub const DASH_DASH: IdentifierIndex = IdentifierIndex(24);
pub const TRUE: IdentifierIndex = IdentifierIndex(25);
pub const FALSE: IdentifierIndex = IdentifierIndex(26);

impl From<IdentifierIndex> for Precedence {
    fn from(identifier: IdentifierIndex) -> Precedence {
        match identifier {
            STAR|SLASH => TimesDivide,
            EQUAL_TO|NOT_EQUAL_TO|GREATER_THAN|GREATER_EQUAL|LESS_THAN|LESS_EQUAL => Comparison,
            AND_AND => And,
            OR_OR => Or,
            ASSIGN|ASSIGN_STAR|ASSIGN_SLASH|ASSIGN_PLUS|ASSIGN_DASH|ASSIGN_AND_AND|ASSIGN_OR_OR => Assign,
            SEMICOLON => SemicolonSequence,
            _ => Precedence::default(),
        }
    }
}

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
