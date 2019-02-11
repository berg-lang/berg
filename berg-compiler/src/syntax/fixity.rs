use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
    Open,
    Close,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionFixity { 
    Term,
    Prefix,
    Open,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorFixity {
    Infix,
    Postfix,
    Close
}

impl Fixity {
    pub fn num_operands(self) -> u8 {
        use Fixity::*;
        match self {
            Term => 0,
            Prefix | Postfix | Open | Close => 1,
            Infix => 2,
        }
    }
    pub fn has_left_operand(self) -> bool {
        use Fixity::*;
        match self {
            Infix | Postfix | Close => true,
            Term | Prefix | Open => false,
        }
    }
    pub fn has_right_operand(self) -> bool {
        use Fixity::*;
        match self {
            Infix | Prefix | Open => true,
            Term | Postfix | Close => false,
        }
    }
    pub fn takes_right_child(self, right: impl Into<Fixity>) -> bool {
        use Fixity::*;
        match (self, right.into()) {
            // Terms are always OK as a right child
            (_, Term) | (_, Prefix) | (_, Open) => true,
            // Term, postfix and close don't take right children at all.
            (Term, _) | (Postfix, _) | (Close, _)=> false,
            // Prefix doesn't take any operators as right child
            (Prefix, Postfix) | (Prefix, Infix) | (Prefix, Close) => false,
            // Open takes all operators as right child
            (Open, Postfix) | (Open, Infix) | (Open, Close) => true,
            // Infix takes postfix operators, but not infix or close.
            (Infix, Postfix) => true,
            (Infix, Infix) | (Infix, Close) => false,
        }
    }
}

impl fmt::Display for Fixity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Fixity::*;
        let fixity = match *self {
            Term => "term",
            Prefix => "unary",
            Infix => "binary",
            Open => "open",
            Close => "close",
            Postfix => "postfix",
        };
        write!(f, "{}", fixity)
    }
}

impl ExpressionFixity {
    pub fn num_operands(self) -> u8 {
        Fixity::from(self).num_operands()
    }
    pub fn has_left_operand(self) -> bool {
        Fixity::from(self).has_left_operand()
    }
    pub fn has_right_operand(self) -> bool {
        Fixity::from(self).has_right_operand()
    }
    pub fn takes_right_child(self, right: impl Into<Fixity>) -> bool {
        Fixity::from(self).takes_right_child(right)
    }
}

impl From<ExpressionFixity> for Fixity {
    fn from(from: ExpressionFixity) -> Self {
        use ExpressionFixity::*;
        match from {
            Term => Fixity::Term,
            Prefix => Fixity::Prefix,
            Open => Fixity::Open,
        }
    }
}

impl fmt::Display for ExpressionFixity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Fixity::from(*self))
    }
}

impl OperatorFixity {
    pub fn num_operands(self) -> u8 {
        Fixity::from(self).num_operands()
    }
    pub fn has_left_operand(self) -> bool {
        Fixity::from(self).has_left_operand()
    }
    pub fn has_right_operand(self) -> bool {
        Fixity::from(self).has_right_operand()
    }
    pub fn takes_right_child(self, right: impl Into<Fixity>) -> bool {
        Fixity::from(self).takes_right_child(right)
    }
}

impl From<OperatorFixity> for Fixity {
    fn from(from: OperatorFixity) -> Self {
        use OperatorFixity::*;
        match from {
            Infix => Fixity::Infix,
            Postfix => Fixity::Postfix,
            Close => Fixity::Close,
        }
    }
}

impl fmt::Display for OperatorFixity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Fixity::from(*self))
    }
}
