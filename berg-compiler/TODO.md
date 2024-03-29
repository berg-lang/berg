<!-- markdownlint-disable MD041 Multiple top-level headings in same document -->
<!-- markdownlint-disable MD024 Multiple headings with same title -->
BERG THE FIRST: SEE NO EVIL
===========================

Berg 0.1 is the syntax and structure of Berg. It is a full version of the Berg language with the important exceptions of a compiler, full error system, system interaction and a packaging system. Its purpose is to give a solid read on what Berg *looks* like, to give a foundation to build libraries and other language features on, and to allow others to give feedback on the language and contribute.

1. Base
    [X] UTF-8
    [X] I/O
    [X] CompilerError Reporting
    [X] Tests
    [X] Command Line
2. Expressions
    [X] Integers
    [X] Operators
    [X] Parentheses
3. Objects
    [X] Fields
    [ ] Blocks
    [ ] Fields
    [ ] Extension
4. Control Flow
    [ ] Lazy Evaluation
    [ ] Function Calls
    [ ] Conditionals
    [ ] Recursion
    [ ] Loops
5. Modules
6. Visual Studio Code Extension

BERG 0.2: HEAR NO EVIL
======================

Berg 0.2 completes the control flow features: effects, and host interaction (FFI).

1. Errors
    [ ] CompilerError Objects
    [ ] CompilerError Codes With Properties (MissingOperand, Side=Left/Right/Both)
    [ ] CompilerError Localization
    [ ] CompilerError Propagation
2. Effects
    [ ] I/O
    [ ]
3. FFI
    [ ] FFI

BERG 0.3: SPEAK NO EVIL
=======================

Berg 0.3 introduces compilation, matchers and types to Berg, moving from its interpreted shape to a statically compiled one. 0.1 and 0.2 were designed with this in mind, but this will be the first true exercise of it.

1. Compilation
2. Compilation
    [ ]

Round 1: Integers
=================

Integer
-------

[X] Integer literal
[X] CompilerError: Integer starts with zero

Invalid UTF-8
-------------

[X] CompilerError: Invalid UTF-8
[X] CompilerError: Unsupported Character
[X] CompilerError: SourceRef Not Found
[X] CompilerError: I/O open error
[X] CompilerError: I/O read error
[X] CompilerError: I/O directory join error for relative path

Tools
-----

[X] Unit Test Project
[X] Command Line

Round 2: Expressions
====================

Runtime
-------

[X] TypeChecker: Type, Number, IntegerLiteral.StackValue() -> Number
[X] TypeRuntime { Run(Type) -> Print Result }
[X] Integer tests

Math
----

[X] Add/Subtract Operators
[X] Multiply/Divide Operators
[X] Precedence
[X] Negative/Positive Operators
[X] CompilerError: Divide By Zero
[X] CompilerError: Unrecognized Operator

Round 3: Boolean Logic
======================

Boolean
-------

[X] true, false
[X] CompilerError: cannot use true/false in math operator

Boolean Operators
-----------------

[X] &&, ||, !
[X] anything other than false is true; && and || return determining argument, not necessarily "true"

Comparison Operators
--------------------

[X] ==, !=
[X] >, <, <=, >=
[X] CompilerError: Non-number in comparison operator

Round 4: Expression Syntax
==========================

Parens
------

[X] Expression Operator "(", ")"
[X] Parse CompilerError: Unclosed Paren
[X] Parse CompilerError: Unopened Paren

Nothing
-------

[X] Type::Nothing
[X] Empty source file -> nothing
[X] Empty parentheses -> nothing
[X] CompilerError: cannot apply operator [/*+-] to "nothing" (either left or right side or both)

Space
-----

[X] Whitespace, Tabs
[X] Compound Term Grouping
[X] Newlines; Record Line / Column Data

Round 5: ScopeRef
==============

Statements
----------

[X] Statement Separator ";"
[X] Trailing Semicolon Acceptable
[X] Final Expression In BlockRef is Return StackValue
[X] Newline Statement Separation
[X] Newline Statement Continuation

Fields
---------

*Fields* are accessible in the scope they were first defined and nowhere else.
[X] Field Assignment (a = b)
[X] Field Reassignment (a = b)
[X] Field Reference (a)
[X] CompilerError: No Such Field
[ ] CompilerError: Unused Field Definition
[ ] CompilerError: Reference Before Definition

Round 6: Blocks
===============

Blocks
------

[ ] {} creates a new block
[ ] CompilerError: ) where expected }
[ ] CompilerError: } where expected )

BlockRef ScopeRef
-----------

[ ] Fields in parent block are accessible and assignable
[ ] Fields declared *after* block, in parent scope, are inaccessible
[ ] Fields declared in sibling scopes are inaccessible to each other
[ ] Fields declared in child scope are inaccessible to parents

BlockRef Laziness
--------------

[ ] "output" function so we can test evaluation
[ ] Unused blocks do not evaluate
[ ] Blocks evaluate on first use
[ ] Blocks evaluate no more than once

Short Circuiting
----------------

[ ] && and || do not evaluate the second argument unless needed

Round 7: Objects
================

Fields
------

*Fields* are fields that have been made public, and are thus accessible outside their scope.
[X] Expose field value (:a = b)
[X] Field usable in expression
[ ] "Unused field error" does not apply to field
[ ] "Unused field error" does not apply to field

Field Access
------------

[ ] Field access (a.b)
[ ] CompilerError: no such field
[ ] CompilerError: field not accessible in scope
[ ] CompilerError: field not

Self
----

Return Self
-----------

Extend
------

Round 5: Conditionals
=====================

if
--

else
----

loop
----

break
-----

Round 5: Functions
==================

APPLY
----

Define and run functions

Round 6: Compilation
====================

Round 5: CompilerError Propagation
==========================

CompilerError Properties
----------------

[ ] CompilerError is Berg object
[ ] Errors have arbitrary number of properties

CompilerError Trail
-----------

[ ] CompilerError given opportunity to pick up source or other information at each use

Round 5: Modules
================

Round 7: System Calls
=====================

Round 8: I/O
============

Round 8: Editing Experience
===========================

Round 9: Packaging / Dependencies
=================================

Round 10: Website
=================

Round 11: 0.1 Release
=====================

Space
-----

[ ] Parse CompilerError: Line Too Long (4K characters)
[ ] Single-Line Comment
[ ] Errors: Invalid UTF-8, Comment Too Long. Denormalized OK. Unsupported Characters OK.

Visual Studio Code Extension
----------------------------

[ ] Syntax highlighting
[ ] CompilerError reporting

Command Line
------------

[ ] Display error context
[ ] Display context with possible fixes

Fields
------

[ ] Field Definition (=)
[ ] Field Modification (+=, ++, etc.)
[X] Field Declaration
[X] "Missing" StackValue
[ ] Parse CompilerError: Identifier Too Large
[X] Parse CompilerError: Identifier Starts With Number
[ ] Parse CompilerError: Identifier Must Be Immediately After ":"
[X] Parse CompilerError: Identifier Required In Declaration
[ ] Object Extend On ";" and "\n" (combine / overwrite properties)

Field Reference
------------------

[X] Field Reference (Identifier)
[ ] "nothing"
[ ] Object.GetField(Identifier) -> Object
[ ] CompilerError: Field not declared
[ ] CompilerError: Field not declared - check for misspelled, give suggestion
[ ] Out Of Order Declaration ("[:]A: B; :B: 2")
[ ] Parse CompilerError: Missing : in front of declaration

Field Assignment
-------------------

[ ] Field Assignment (=)
[ ] Parse CompilerError: Missing Operand

Apply
-----

[ ] Apply Operator (Extend With BlockRef) "F <+ A: 1" or "F <+ { A: 1; B: 2 }"
[ ] Because of multiple reasons! Figure out error dedup strategies here ...

Functions
=========

List
------

[ ] List Operator (,)
[ ] Trailing Comma Acceptable

Function Calls
--------------

[ ] APPLY Operator Overload: ("APPLY :: { :Arguments, ... }")
[ ] Function APPLY: "F Arguments"
[ ] Function Declaration Syntax: ":F(:A,:B,:C)"
[ ] Inline Function Declaration Syntax

Indented Function Calls
-----------------------

[ ] Child BlockRef Function Arguments
[ ] Parse CompilerError: Inconsistent Indent Characters (space vs. tab)
[ ] Parse CompilerError: Multiple Undent

Conditionals
============

Booleans
--------

[ ] "true", "false"
[ ] StackValue::True, StackValue::False
[ ] CompilerError: Appropriate Operator Errors

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
[ ] Parse CompilerError: Unclosed Curly Brace
[ ] Parse CompilerError: Unopened Curly Brace

[ ] Empty "{}" -> Nothing

Child Access
------------

[ ] Field Access (Dot) Operator
[ ] Parse CompilerError: Identifier Required For Field Access

[ ] Nested BlockRef Declarations

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
[ ] Parse CompilerError: Unclosed String
[ ] Parse CompilerError: Invalid UTF-8 In String

[ ] Hex Byte
[ ] Parse CompilerError: Invalid UTF-8 Across Escape Sequence And String (probably same error)

[ ] Hex Byte Sequence

[ ] String Escapes

INTERPOLATED STRING
-------------------

[ ] Interpolation
[ ] Unclosed Interpolation CompilerError

UNICODE IDENTIFIERS
-------------------

[ ] Unicode Identifiers (XID)

[ ] Incomplete Grapheme CompilerError

[ ] Denormalized Identifier CompilerError

UNICODE STRING ESCAPES
----------------------

[ ] Unicode Escape Character
[ ] Unicode Escape Sequence
[ ] Unicode Character Name

FUNCTION
--------

[ ] Function APPLY Apply "Operator"

FLOW
----

[ ]

EXPLICIT FLOW
-------------

[ ] Statement Sequence Extend "Operator"

[ ] Curly Brace Operator
[ ] Better Unclosed Parenthesis CompilerError
[ ] Better Unclosed Paren CompilerError

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
[ ] Parse CompilerError: Number Starts With Zero
[ ] Parse CompilerError: Hexadecimal With Uppercase X
[ ] Parse CompilerError: Octal With Uppercase O
[ ] Parse CompilerError: Binary With Uppercase B

FLOAT
-----

[ ] Decimal Point Operator

[ ] Exponent Operator
[ ] Exponent Sign Operator
[ ] Missing Exponent CompilerError

IMAGINARY
---------

[ ] Imaginary Operator
[ ] Imaginary With Uppercase I CompilerError
