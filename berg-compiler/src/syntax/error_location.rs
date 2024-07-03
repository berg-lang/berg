use super::{AstIndex, AstRef, ByteRange};

#[derive(Debug, Clone)]
pub enum ErrorLocation<'a> {
    Generic,
    SourceOnly(AstRef<'a>),
    SourceExpression(AstRef<'a>, AstIndex),
    SourceRange(AstRef<'a>, ByteRange),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionErrorPosition {
    Expression,
    Left,
    Right,
    LeftLeft,
    LeftRight,
    RightLeft,
    RightRight,
}

impl ExpressionErrorPosition {
    pub fn relative_to(self, new_position: ExpressionErrorPosition) -> ExpressionErrorPosition {
        use ExpressionErrorPosition::*;
        match (new_position, self) {
            (new_position, Expression) => new_position,
            (Expression, position) => position,
            (Left, Left) => LeftLeft,
            (Left, Right) => LeftRight,
            (Right, Left) => RightLeft,
            (Right, Right) => RightRight,
            (LeftLeft, _)
            | (LeftRight, _)
            | (RightLeft, _)
            | (RightRight, _)
            | (_, LeftLeft)
            | (_, LeftRight)
            | (_, RightLeft)
            | (_, RightRight) => unreachable!(
                "Cannot reposition {:?} on top of {:?}: too deep!",
                self, new_position
            ),
        }
    }
}
