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
    FollowedBy,
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
            FOLLOWED_BY | IMMEDIATELY_FOLLOWED_BY => FollowedBy,
            SEMICOLON => SemicolonSequence,
            NEWLINE_SEQUENCE => NewlineSequence,
            _ => DEFAULT_PRECEDENCE,
        }
    }
}

impl Precedence {
    pub(crate) fn takes_right_child(self, right: Precedence) -> bool {
        match self {
            Dot => false,
            TimesDivide => matches!(right, Dot),
            PlusMinus => matches!(right, Dot | TimesDivide),
            Comparison => matches!(right, Dot | TimesDivide | PlusMinus),
            And => matches!(right, Dot | TimesDivide | PlusMinus | Comparison),
            Or => matches!(right, Dot | TimesDivide | PlusMinus | Comparison | And),
            CommaSequence => matches!(right, Dot | TimesDivide | PlusMinus | Comparison | And | Or),
            Assign => matches!(
                right,
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence
            ),
            ColonDeclaration => matches!(
                right,
                Dot | TimesDivide | PlusMinus | Comparison | And | Or | CommaSequence | Assign
            ),
            FollowedBy => matches!(
                right,
                Dot | TimesDivide
                    | PlusMinus
                    | Comparison
                    | And
                    | Or
                    | CommaSequence
                    | Assign
                    | ColonDeclaration
            ),
            SemicolonSequence => matches!(
                right,
                Dot | TimesDivide
                    | PlusMinus
                    | Comparison
                    | And
                    | Or
                    | CommaSequence
                    | Assign
                    | ColonDeclaration
                    | FollowedBy
            ),
            NewlineSequence => matches!(
                right,
                Dot | TimesDivide
                    | PlusMinus
                    | Comparison
                    | And
                    | Or
                    | CommaSequence
                    | Assign
                    | ColonDeclaration
                    | FollowedBy
                    | SemicolonSequence
            ),
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
