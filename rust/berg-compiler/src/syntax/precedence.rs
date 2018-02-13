use syntax::{IdentifierIndex,Token};
use syntax::Precedence::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Precedence {
    Dot,
    TimesDivide,
    PlusMinus,
    Comparison,
    And,
    Or,
    Assign,
    ColonDeclaration,
    SemicolonSequence,
    NewlineSequence,
}

const DEFAULT_PRECEDENCE: Precedence = Precedence::PlusMinus;

impl Default for Precedence {
    fn default() -> Precedence {
        DEFAULT_PRECEDENCE
    }
}

impl From<IdentifierIndex> for Precedence {
    fn from(from: IdentifierIndex) -> Self {
        use syntax::identifiers::*;
        match from {
            DOT => Dot,
            STAR | SLASH => TimesDivide,
            PLUS | DASH => PlusMinus,
            EQUAL_TO | NOT_EQUAL_TO | GREATER_THAN | GREATER_EQUAL | LESS_THAN | LESS_EQUAL => {
                Comparison
            }
            AND_AND => And,
            OR_OR => Or,
            COLON => ColonDeclaration,
            SEMICOLON => SemicolonSequence,
            _ => DEFAULT_PRECEDENCE,
        }
    }
}

impl Precedence {
    pub(crate) fn takes_right_child(self, right: Precedence) -> bool {
        match self {
            Dot => false,
            TimesDivide => match right {
                Dot => true,
                _ => false,
            },
            PlusMinus => match right {
                Dot | TimesDivide => true,
                _ => false,
            },
            Comparison => match right {
                Dot | TimesDivide | PlusMinus => true,
                _ => false,
            },
            And => match right {
                Dot | TimesDivide | PlusMinus | Comparison => true,
                _ => false,
            },
            Or => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And => true,
                _ => false,
            },
            Assign => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or => true,
                _ => false,
            },
            ColonDeclaration => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | Assign => true,
                _ => false
            }
            SemicolonSequence => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | Assign | ColonDeclaration => true,
                _ => false
            }
            NewlineSequence => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | Assign | ColonDeclaration | SemicolonSequence => {
                    true
                }
                _ => false,
            },
        }
    }

    pub(crate) fn takes_left_child(self, left: Precedence) -> bool {
        !left.takes_right_child(self)
    }
}

impl From<Token> for Precedence {
    fn from(from: Token) -> Precedence {
        use syntax::Precedence::*;
        match from {
            Token::InfixOperator(operator) => operator.into(),
            Token::InfixAssignment(_) => Assign,
            Token::NewlineSequence => Precedence::NewlineSequence,
            Token::Apply => Precedence::default(),
            // Should only ever be called for infix
            _ => unreachable!(),
        }
    }
}
