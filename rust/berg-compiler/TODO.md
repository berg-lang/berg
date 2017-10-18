Integer Math
============

Tests
-----
[X] Unit Test Project
[X] Invalid UTF-8 Error Test
[X] Unsupported Character Error Test
[X] Both Errors (Alternating) Test

Runtime
-------
[X] TypeChecker: Type, Number, IntegerLiteral.Value() -> Number
[X] TypeRuntime { Run(Type) -> Print Result }
[X] Integer tests

Math
----
[X] Add/Subtract Operators

[X] Multiply/Divide Operators
[ ] Precedence

[ ] ProducesBecause { Values(): ValueSet, BecauseAll(): Iterator<(Expression, ValueSet)> }
[ ] Expression { Produces() -> Iterator<ProducesBecause> }
[ ] Type.Contains(value)
[X] Type::Single(Value)
[ ] Expression as Producer
[X] Type Error: Divide By Zero

[X] Negative/Positive Operators

Parens
------
[ ] Expression Operator "(", ")"
[ ] Parse Error: Unclosed Paren
[ ] Parse Error: Unopened Paren Error

Space
-----
[ ] Whitespace, Tabs

[ ] Compound Term Grouping

[ ] Newlines; Record Line / Column Data
[ ] Parse Error: Line Too Long (4K characters)

[ ] Single-Line Comment
[ ] Errors: Invalid UTF-8, Comment Too Long. Denormalized OK. Unsupported Characters OK.

Errors
------
[ ] Display error context

[ ] Display context with possible fixes

[X] Parse Error: Improve Unsupported Character Error (many characters)

Objects
=======

Nothing
-------
[ ] "nothing"
[X] Type::Nothing

[X] Empty source file -> nothing

[ ] Empty parentheses -> nothing

[ ] Error: cannot apply operator [/*+-] to "nothing" (either left or right side or both)

Sequences
---------
[ ] Sequence Operator ";"
[ ] Trailing Semicolon Acceptable
[ ] Final Operator In Sequence is Return Value

Newline Sequences
-----------------
[ ] Newline Statement Separation
[ ] Newline Statement Continuation

Properties
----------
[ ] Property Declaration
[ ] "Missing" Value
[ ] Parse Error: Identifier Too Large
[ ] Parse Error: Identifier Starts With Number
[ ] Parse Error: Identifier Must Be Immediately After ":"
[ ] Parse Error: Identifier Required In Declaration

[ ] Object Extend On ";" and "\n" (combine / overwrite properties)

[ ] Property Assignment (=)
[ ] Parse Error: Missing Operand

[ ] Property Reference (Identifier)
[ ] Object.GetProperty(Identifier) -> Object
[ ] Error: Property not declared

[ ] Error: Property not declared - check for misspelled, give suggestion

Lazy Declaration
----------------
[ ] Out Of Order Declaration ("[:]A: B; :B: 2")
[ ] Parse Error: Missing : in front of declaration

Apply
-----
[ ] Apply Operator (Extend With Block) "F <+ A: 1" or "F <+ { A: 1; B: 2 }"
[ ] BecauseOf multiple reasons! Figure out error dedup strategies here ...

Functions
=========

Series
------
[ ] Series Operator (,)

[ ] Trailing Comma Acceptable

Function Calls
--------------

[ ] Call Operator Overload: ("Call :: { :Arguments, ... }")

[ ] Function Call: "F Arguments"

[ ] Function Declaration Syntax: ":F(:A,:B,:C)"

[ ] Inline Function Declaration Syntax

Indented Function Calls
-----------------------
[ ] Child Block Function Arguments
[ ] Parse Error: Inconsistent Indent Characters (space vs. tab)
[ ] Parse Error: Multiple Undent

Conditionals
============

Booleans
--------
[ ] "true", "false"
[ ] Value::True, Value::False
[ ] Error: Appropriate Operator Errors

If/Else
-------

Comparison Operators
--------------------

Boolean Operators
-----------------


Structure
=========

Child Objects
-------------
[ ] Object Operator "{", "}"
[ ] Parse Error: Unclosed Curly Brace
[ ] Parse Error: Unopened Curly Brace

[ ] Empty "{}" -> Nothing

Child Access
------------
[ ] Property Access (Dot) Operator
[ ] Parse Error: Identifier Required For Property Access

[ ] Nested Block Declarations

Includes
--------



COMPARISON
----------
[ ] Equal/Not Equal Operators

[ ] Greater Than/Less Than/Greater Than Or Equal To/Less Than Or Equal To Operators

[ ] Comparison Operator ("<=>")

BOOLEAN
-------
[ ] And/Or/Not Operators

CONDITIONAL
-----------
[ ] If: if X Y else Z

[ ] Else: if X Y else Z


CONTEXT
-------
[ ] ::FileContext - file level input properties

STRING
------
[ ] Raw String
[ ] Parse Error: Unclosed String
[ ] Parse Error: Invalid UTF-8 In String

[ ] Hex Byte
[ ] Parse Error: Invalid UTF-8 Across Escape Sequence And String (probably same error)

[ ] Hex Byte Sequence

[ ] String Escapes

INTERPOLATED STRING
-------------------
[ ] Interpolation
[ ] Unclosed Interpolation Error

UNICODE IDENTIFIERS
-------------------
[ ] Unicode Identifiers (XID)

[ ] Incomplete Grapheme Error

[ ] Denormalized Identifier Error

UNICODE STRING ESCAPES
----------------------
[ ] Unicode Escape Character
[ ] Unicode Escape Sequence
[ ] Unicode Character Name

FUNCTION
--------
[ ] Function Call Apply "Operator"

FLOW
----
[ ] 

EXPLICIT FLOW
-------------
[ ] Statement Sequence Extend "Operator"

[ ] Curly Brace Operator
[ ] Better Unclosed Parenthesis Error
[ ] Better Unclosed Paren Error

ARRAYS
------
[ ] Index Operator
[ ] 

HASHES
------
[ ] Pair Operator

HEX/OCT/BIN
-----------
[ ] Hexadecimal Number
[ ] Octal Number
[ ] Binary Number
[ ] Parse Error: Number Starts With Zero
[ ] Parse Error: Hexadecimal With Uppercase X
[ ] Parse Error: Octal With Uppercase O
[ ] Parse Error: Binary With Uppercase B

FLOAT
-----
[ ] Decimal Point Operator

[ ] Exponent Operator
[ ] Exponent Sign Operator
[ ] Missing Exponent Error

IMAGINARY
---------
[ ] Imaginary Operator
[ ] Imaginary With Uppercase I Error

