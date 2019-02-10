use crate::eval::ScopeRef;
use crate::syntax::identifiers::{
    APPLY, COLON, COMMA, DASH_DASH, DOT, EMPTY_STRING, NEWLINE, PLUS_PLUS, SEMICOLON,
};
use crate::syntax::{
    Expression, ExpressionBoundaryError, ExpressionRef, FieldIndex, IdentifierIndex, Token,
};
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::{
    BergError, BergResult, BergVal, BergValue, EvalError, EvalResult, NextVal, TakeError, Tuple,
};
use num::BigRational;
use std::fmt;
use std::str::FromStr;

pub type ExpressionEvaluator<'p, 'a> = Expression<'p, 'a, &'p ScopeRef<'a>>;

impl<'p, 'a: 'p> From<ExpressionEvaluator<'p, 'a>> for ExpressionRef<'a> {
    fn from(from: ExpressionEvaluator<'p, 'a>) -> Self {
        ExpressionRef::new(from.scope().ast(), from.index())
    }
}
impl<'p, 'a: 'p> fmt::Display for ExpressionEvaluator<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'p, 'a: 'p> ExpressionEvaluator<'p, 'a> {
    fn scope(self) -> &'p ScopeRef<'a> {
        self.context()
    }
    pub fn evaluate(self) -> BergResult<'a> {
        let indent = "  ".repeat(self.depth());
        println!("{}Evaluating {} ...", indent, self);
        use crate::syntax::ExpressionBoundaryError::*;
        use crate::syntax::Token::*;
        use crate::value::ErrorCode::*;
        let result = match self.token() {
            //
            // Nouns (operands)
            //

            // 1234
            IntegerLiteral(literal) => {
                let parsed = BigRational::from_str(self.ast().literal_string(literal)).unwrap();
                Ok(BergVal::BigRational(parsed))
            }
            // VariableName
            FieldReference(field) => self.scope().local_field(field, self.ast()).take_error(self),
            // VariableName
            RawIdentifier(name) => Ok(name.into()),
            // Empty parens or empty block
            // () {}
            MissingExpression => BergError::MissingExpression.take_error(self),

            //
            // Infix operators
            //

            // A; B
            InfixOperator(SEMICOLON) => self.evaluate_semicolon(),
            // Field: Value
            InfixOperator(COLON) => self.evaluate_colon(),
            // A, B, C, D
            InfixOperator(COMMA) => self.evaluate_comma(),
            // A <op> B
            InfixOperator(operator) => self.evaluate_infix(operator),
            // A <op>= B
            InfixAssignment(operator) => self.evaluate_infix_assign(operator),
            // Multiline sequence:
            // A
            // B
            NewlineSequence => self.evaluate_infix(NEWLINE),
            // F Arg
            Apply => self.evaluate_infix(APPLY),

            //
            // Prefix operators
            //

            // A++
            PrefixOperator(PLUS_PLUS) => self.evaluate_prefix_assign(PLUS_PLUS),
            // A--
            PrefixOperator(DASH_DASH) => self.evaluate_prefix_assign(DASH_DASH),
            // A:
            PrefixOperator(COLON) => self.evaluate_declare(),
            // A<op>
            PrefixOperator(operator) => self.evaluate_prefix(operator),

            PostfixOperator(PLUS_PLUS) => self.evaluate_postfix_assign(PLUS_PLUS),
            PostfixOperator(DASH_DASH) => self.evaluate_postfix_assign(DASH_DASH),
            PostfixOperator(operator) => self.evaluate_postfix(operator),

            //
            // Syntax errors
            //
            ErrorTerm(IdentifierStartsWithNumber, literal) => {
                BergError::IdentifierStartsWithNumber(literal).take_error(self)
            }
            ErrorTerm(UnsupportedCharacters, literal) => {
                BergError::UnsupportedCharacters(literal).take_error(self)
            }
            RawErrorTerm(InvalidUtf8, raw_literal) => {
                BergError::InvalidUtf8(raw_literal).take_error(self)
            }
            // ( and { syntax errors
            Open {
                error: OpenError, ..
            }
            | OpenBlock {
                error: OpenError, ..
            } => self.ast().open_error().clone().take_error(self),
            OpenBlock {
                error: ExpressionBoundaryError::OpenWithoutClose,
                ..
            }
            | Open {
                error: ExpressionBoundaryError::OpenWithoutClose,
                ..
            } => BergError::OpenWithoutClose.take_error(self),
            OpenBlock {
                error: ExpressionBoundaryError::CloseWithoutOpen,
                ..
            }
            | Open {
                error: ExpressionBoundaryError::CloseWithoutOpen,
                ..
            } => BergError::CloseWithoutOpen.take_error(self),

            //
            // Groupings
            //

            // (...)
            Open { error: None, .. } => self.evaluate_open(),

            // {...}, indent group
            OpenBlock {
                error: None, index, ..
            } => Ok(self
                .scope()
                .create_child_block(self.inner_expression().index(), index)
                .into()),

            // Tokens that should have been handled elsewhere in the stack
            Close { .. } | CloseBlock { .. } | ErrorTerm(..) | RawErrorTerm(..) => unreachable!(),
        };
        println!(
            "{}Evaluated to {}",
            indent,
            match &result {
                Ok(value) => format!("{}", value),
                Err(error) => format!("{}", error),
            }
        );
        result
    }

    fn evaluate_open(self) -> BergResult<'a> {
        let inner = self.inner_expression();
        if let Token::MissingExpression = inner.token() {
            Ok(BergVal::empty_tuple())
        } else {
            inner.evaluate()
        }
    }

    //
    // a; b; c
    //
    // a   b
    //  \ /
    //   ;   c
    //    \ /
    //     ;
    //     ^ you are here
    //
    // We evaluate all semicolons together, since terminating semocolons
    // are treated slightly differently: 1; ;2 yields an error, while 1;2;  does not.
    //                                     ^                              ^
    fn evaluate_semicolon(self) -> BergResult<'a> {
        let left_operand = self.left_operand()?;
        let left = match left_operand.token() {
            Token::InfixOperator(SEMICOLON) => left_operand.evaluate_semicolon_left()?,
            _ => left_operand.evaluate()?,
        };
        let right = self.right_expression();
        match right.token() {
            // If the rightmost semicolon has no operand, we treat it as
            // returning the empty tuple
            Token::MissingExpression => Ok(BergVal::empty_tuple()),
            // Check for malformed tree
            Token::InfixOperator(SEMICOLON) => panic!("semicolon on the right hand side of a semicolon is unexpected: right hand side of {}!", self),
            _ => left.infix(SEMICOLON, right).take_error(self),
        }
    }

    //
    // 1,2,3
    //
    // 1   2
    //  \ /
    //   ,   3
    //    \ /
    //     ,
    //     ^ you are here
    //
    // We evaluate all commas together because the right side of inner expressions
    // are treated slightly differently: 1, ,2 yields an error, while 1,2,  does not.
    //                                     ^                              ^
    // Additionally, we want to store them as a Vec, and it's performant if we
    // can create the Vec all at once.
    //
    // Finally, a note: Tuples are stored in reverse order. So "1,2,3" is stored
    // internally as "3,2,1." This is so that "next" is easy to do on the Tuple:
    // you just pop the value off the end. No muss, no fuss.
    //
    fn evaluate_comma(self) -> BergResult<'a> {
        //
        // The goal is to get a vector in reverse order, because that's how Tuples
        // are stored (for efficiency reasons).
        //
        let left = self.left_operand()?;
        let right = self.right_expression();
        let vec = match right.token() {
            // If the rightmost comma has no operand, we ignore it: 1,2, == 1,2
            Token::MissingExpression => left.evaluate_comma_left(0)?,
            // Check for malformed tree
            Token::InfixOperator(COMMA) => panic!(
                "comma on the right hand side of a comma is unexpected: right hand side of {}!",
                self
            ),
            // Grab the left hand side of the vec and then insert this value.
            _ => {
                let mut vec = left.evaluate_comma_left(1)?;
                vec[0] = right.evaluate()?;
                vec
            }
        };
        Ok(BergVal::Tuple(Tuple::from_reversed_vec(vec)))
    }

    fn evaluate_infix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let left = self.left_operand()?;
        let right = self.right_operand()?;
        left.infix(operator, right).take_error(self)
    }

    fn evaluate_infix_assign(self, operator: IdentifierIndex) -> BergResult<'a> {
        let mut target = self.left_operand()?.into_assignment_target()?;
        let right = self.right_operand()?;
        let value = match operator {
            EMPTY_STRING => {
                target.initialize()?;
                right.evaluate()
            }
            _ => target.get()?.infix(operator, right).take_error(self),
        };
        target.set(value)
    }

    fn evaluate_colon(self) -> BergResult<'a> {
        // Declare the variable so it can self-reference if needed.
        let mut target = self
            .left_operand()?
            .into_assignment_target()?
            .in_declaration();
        target.declare()?;

        // Because the right operand of colon is *always* a block, the MissingExpression will be inside it (if any).
        let right = self.right_operand()?;
        assert!(match right.token() {
            Token::OpenBlock { .. } => true,
            _ => false,
        });
        if let Token::MissingExpression = right.inner_expression().token() {
            return BergError::MissingExpression.take_error(self);
        }

        // Now just evaluate and assign!
        let value = right.evaluate();
        target.set(value)
    }

    fn evaluate_prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.prefix_operand()?.prefix(operator).take_error(self)
    }

    fn evaluate_postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let operand = self.postfix_operand()?;
        operand.postfix(operator).take_error(self)
    }

    fn evaluate_prefix_assign(self, operator: IdentifierIndex) -> BergResult<'a> {
        let mut target = self.prefix_operand()?.into_assignment_target()?;
        target.initialize()?;
        let value = target.get()?.prefix(operator).take_error(self);
        target.set(value)
    }

    fn evaluate_postfix_assign(self, operator: IdentifierIndex) -> BergResult<'a> {
        let mut target = self.postfix_operand()?.into_assignment_target()?;
        target.initialize()?;
        let value = target.get()?.prefix(operator).take_error(self);
        target.set(value)
    }

    fn evaluate_declare(self) -> BergResult<'a> {
        let mut target = self.into_assignment_target()?;
        target.declare()?;
        target.get()
    }

    fn into_assignment_target(self) -> BergResult<'a, AssignmentTarget<'p, 'a>> {
        use crate::syntax::Token::*;
        match self.token() {
            FieldReference(field) => Ok(AssignmentTarget::Local(self, field)),
            PrefixOperator(COLON) => {
                let colon_operand = self.prefix_operand()?;
                match colon_operand.token() {
                    FieldReference(field) => {
                        Ok(AssignmentTarget::DeclareLocal(colon_operand, field))
                    }
                    _ => BergError::AssignmentTargetMustBeIdentifier.take_error(colon_operand),
                }
            }
            Open {
                error: ExpressionBoundaryError::None,
                ..
            } => self.inner_expression().into_assignment_target(),
            InfixOperator(DOT) => {
                let right = self.right_operand()?;
                match right.token() {
                    RawIdentifier(name) => {
                        let object = self.left_operand()?.evaluate()?;
                        Ok(AssignmentTarget::Object(self, object, name))
                    }
                    _ => BergError::AssignmentTargetMustBeIdentifier.take_error(right),
                }
            }
            _ => BergError::AssignmentTargetMustBeIdentifier.take_error(self),
        }
    }

    fn evaluate_comma_left(self, index: usize) -> BergResult<'a, Vec<BergVal<'a>>> {
        let vec = match self.token() {
            // Get the vector with the left hand side already filled in, then add the right hand side.
            Token::InfixOperator(COMMA) => {
                let mut vec = self.left_operand()?.evaluate_comma_left(index + 1)?;
                let right = self.right_operand()?;
                assert_ne!(
                    right.token(),
                    Token::InfixOperator(COMMA),
                    "comma on the right hand side of a comma is unexpected: right hand side of {}!",
                    self.to_string()
                );
                vec[index] = right.evaluate()?;
                vec
            }
            // Create the vector with just enough to put our left and right operands in.
            _ => {
                let mut vec = vec![BergVal::Nothing; index + 1];
                vec[index] = self.evaluate()?;
                vec
            }
        };
        Ok(vec)
    }
    fn evaluate_semicolon_left(self) -> BergResult<'a> {
        let left = self.left_operand()?.evaluate()?;
        let right = self.right_operand()?;
        match right.token() {
            // Check for malformed tree
            Token::InfixOperator(SEMICOLON) => panic!("semicolon on the right hand side of a semicolon is unexpected: right hand side of {}!", self),
            _ => left.infix(SEMICOLON, right).take_error(self)
        }
    }

    pub fn left_operand(self) -> BergResult<'a, Self>
    {
        let operand = self.left_expression();
        match operand.token() {
            Token::MissingExpression => BergError::MissingExpression.take_error(self),
            _ => Ok(operand),
        }
    }
    pub fn right_operand(self) -> BergResult<'a, Self>
    {
        let operand = self.right_expression();
        match operand.token() {
            Token::MissingExpression => BergError::MissingExpression.take_error(self),
            _ => Ok(operand),
        }
    }
    pub fn prefix_operand(self) -> BergResult<'a, Self>
    {
        let operand = self.right_expression();
        match operand.token() {
            Token::MissingExpression => BergError::MissingExpression.take_error(self),
            _ => Ok(operand),
        }
    }
    pub fn postfix_operand(self) -> BergResult<'a, Self>
    {
        let operand = self.left_expression();
        match operand.token() {
            Token::MissingExpression => BergError::MissingExpression.take_error(self),
            _ => Ok(operand),
        }
    }
}

impl<'p, 'a: 'p> BergValue<'a> for ExpressionEvaluator<'p, 'a> {
    fn infix<T: BergValue<'a>>(self, operator: IdentifierIndex, right: T) -> EvalResult<'a> {
        self.evaluate()?.infix(operator, right)
    }
    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        self.evaluate()?.postfix(operator)
    }
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        self.evaluate()?.prefix(operator)
    }
    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        self.evaluate()?.field(name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        self.evaluate()?.set_field(name, value)
    }
    fn into_val(self) -> BergResult<'a> {
        self.evaluate()
    }
    fn next_val(self) -> BergResult<'a, NextVal<'a>> {
        self.evaluate()?.next_val()
    }
    fn into_native<T: TypeName + TryFrom<BergVal<'a>>>(self) -> BergResult<'a, EvalResult<'a, T>>
    where
        <T as TryFrom<BergVal<'a>>>::Error: Into<BergVal<'a>>,
    {
        match self.evaluate()?.into_native() {
            Ok(Err(EvalError::Raw(BergError::BadType(value, expected_type)))) => {
                Ok(Err(EvalError::Raw(BergError::BadOperandType(
                    self.operand_position(),
                    value,
                    expected_type,
                ))))
            }
            result => result,
        }
    }
}

#[derive(Debug)]
enum AssignmentTarget<'p, 'a: 'p> {
    Local(ExpressionEvaluator<'p, 'a>, FieldIndex),
    DeclareLocal(ExpressionEvaluator<'p, 'a>, FieldIndex),
    Object(ExpressionEvaluator<'p, 'a>, BergVal<'a>, IdentifierIndex),
}

impl<'p, 'a: 'p> AssignmentTarget<'p, 'a> {
    fn in_declaration(self) -> Self {
        use AssignmentTarget::*;
        match self {
            Local(expression, field) => DeclareLocal(expression, field),
            value => value,
        }
    }

    fn initialize(&self) -> BergResult<'a, ()> {
        use AssignmentTarget::*;
        match self {
            Local(expression, field) | DeclareLocal(expression, field) => expression
                .scope()
                .bring_local_field_into_scope(*field, expression.ast())
                .take_error(*expression),
            Object(..) => Ok(()),
        }
    }

    fn get(&mut self) -> BergResult<'a> {
        use crate::syntax::identifiers::DOT;
        use AssignmentTarget::*;
        self.initialize()?;
        match self {
            Local(expression, field) | DeclareLocal(expression, field) => expression
                .scope()
                .local_field(*field, expression.ast())
                .take_error(*expression),
            // Infix consumes values, but we still need the object around, so we clone the obj (it's cheap at the moment, a reference or primitive)
            Object(expression, object, _) => {
                let right = expression.right_operand()?;
                object.clone().infix(DOT, right).take_error(*expression)
            }
        }
    }

    fn set(&mut self, value: BergResult<'a>) -> BergResult<'a> {
        use AssignmentTarget::*;
        match self {
            Local(expression, field) | DeclareLocal(expression, field) => expression
                .scope()
                .set_local_field(*field, value, expression.ast())
                .take_error(*expression)?,
            Object(expression, object, name) => {
                object.set_field(*name, value).take_error(*expression)?
            }
        }
        // If it's a declaration, declare it public now that it's been set.
        self.declare()?;
        Ok(BergVal::empty_tuple())
    }

    fn declare(&mut self) -> BergResult<'a> {
        use AssignmentTarget::*;
        match self {
            DeclareLocal(expression, field) => expression
                .scope()
                .declare_field(*field, expression.ast())
                .take_error(*expression)?,
            Local(..) | Object(..) => {}
        }
        Ok(BergVal::empty_tuple())
    }
}
