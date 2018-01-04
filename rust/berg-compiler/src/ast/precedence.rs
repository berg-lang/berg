use ast::precedence::Precedence::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Precedence {
    TimesDivide,
    PlusMinus,
    Comparison,
    And,
    Or,
    Assign,
    SemicolonSequence,
    NewlineSequence,
}

impl Default for Precedence {
    fn default() -> Precedence {
        PlusMinus
    }
}

impl Precedence {
    pub(crate) fn takes_right_child(self, right: Precedence) -> bool {
        match self {
            TimesDivide => false,
            PlusMinus => match right {
                TimesDivide => true,
                _ => false,
            },
            Comparison => match right {
                TimesDivide | PlusMinus => true,
                _ => false,
            },
            And => match right {
                TimesDivide | PlusMinus | Comparison => true,
                _ => false,
            },
            Or => match right {
                TimesDivide | PlusMinus | Comparison | And => true,
                _ => false,
            },
            Assign => match right {
                TimesDivide | PlusMinus | Comparison | And | Or => true,
                _ => false,
            },
            SemicolonSequence => match right {
                TimesDivide | PlusMinus | Comparison | And | Or | Assign => true,
                _ => false,
            },
            NewlineSequence => match right {
                TimesDivide | PlusMinus | Comparison | And | Or | Assign | SemicolonSequence => {
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
