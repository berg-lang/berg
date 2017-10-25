use parser::IdentifierIndex;
use parser::LiteralIndex;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Token {
    IntegerLiteral(LiteralIndex),
    Infix(IdentifierIndex),
    Prefix(IdentifierIndex),
    Postfix(IdentifierIndex),
    MissingInfix,
    MissingTerm,
    Nothing,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
}

impl Token {
    pub fn fixity(&self) -> Fixity {
        use Token::*;
        match *self {
            IntegerLiteral(_)|MissingTerm|Nothing => Fixity::Term,
            Infix(_)|MissingInfix => Fixity::Infix,
            Prefix(_) => Fixity::Prefix,
            Postfix(_) => Fixity::Postfix,
        }
    }
    pub fn num_operands(&self) -> u8 { self.fixity().num_operands() }
    pub fn has_left_operand(&self) -> bool { self.fixity().has_left_operand() }
    pub fn has_right_operand(&self) -> bool { self.fixity().has_right_operand() }
}

impl Fixity {
    pub fn num_operands(&self) -> u8 {
        use parser::token::Fixity::*;
        match *self {
            Term => 0,
            Prefix|Postfix => 1,
            Infix => 2,
        }
    }
    pub fn has_left_operand(&self) -> bool {
        use parser::token::Fixity::*;
        match *self {
            Term|Prefix => false,
            Infix|Postfix => true,
        }
    }
    pub fn has_right_operand(&self) -> bool {
        use parser::token::Fixity::*;
        match *self {
            Term|Postfix => false,
            Infix|Prefix => true,
        }
    }
}
