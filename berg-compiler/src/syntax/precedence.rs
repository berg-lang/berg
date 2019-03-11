use crate::syntax::{IdentifierIndex, OperatorToken};
use Precedence::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Precedence {
    Dot,
    TimesDivide,
    PlusMinus,
    Comparison,
    And,
    Or,
    CommaSequence,
    Assign,
    ColonDeclaration,
    Apply,
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
        use crate::syntax::identifiers::*;
        match from {
            DOT => Dot,
            STAR | SLASH => TimesDivide,
            PLUS | DASH => PlusMinus,
            EQUAL_TO | NOT_EQUAL_TO | GREATER_THAN | GREATER_EQUAL | LESS_THAN | LESS_EQUAL => {
                Comparison
            }
            AND_AND => And,
            OR_OR => Or,
            COMMA => CommaSequence,
            COLON => ColonDeclaration,
            APPLY => Apply,
            SEMICOLON => SemicolonSequence,
            NEWLINE => NewlineSequence,
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
            CommaSequence => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or => true,
                _ => false,
            },
            Assign => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence => true,
                _ => false,
            },
            ColonDeclaration => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence | Assign => {
                    true
                }
                _ => false,
            },
            Apply => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence | Assign
                | ColonDeclaration => true,
                _ => false,
            },
            SemicolonSequence => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence | Assign
                | ColonDeclaration | Apply => true,
                _ => false,
            },
            NewlineSequence => match right {
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence | Assign
                | ColonDeclaration | SemicolonSequence | Apply => true,
                _ => false,
            },
        }
    }
}

impl From<OperatorToken> for Precedence {
    fn from(from: OperatorToken) -> Precedence {
        use OperatorToken::*;
        match from {
            InfixOperator(operator) => operator.into(),
            InfixAssignment(_) => Precedence::Assign,
            // Should only ever be called for infix
            _ => unreachable!(),
        }
    }
}
