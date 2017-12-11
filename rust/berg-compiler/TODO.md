Roadmap:

BERG THE FIRST: SEE NO EVIL
===========================

Berg 0.1 is the syntax and structure of Berg. It is a full version of the Berg language with the important exceptions of a compiler, full error system, system interaction and a packaging system. Its purpose is to give a solid read on what Berg *looks* like, to give a foundation to build libraries and other language features on, and to allow others to give feedback on the language and contribute.

1. Base
    [X] UTF-8
    [X] I/O
    [X] Error Reporting
    [X] Tests
    [X] Command Line
2. Expressions
    [X] Integers
    [X] Operators
    [X] Parentheses
3. Objects
    [X] Variables
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

2. Errors
    [ ] Error Objects
    [ ] Error Codes With Properties (MissingOperand, Side=Left/Right/Both)
    [ ] Error Localization
    [ ] Error Propagation
3. Effects
    [ ] I/O
    [ ]
4. FFI
    [ ] FFI

BERG 0.3: SPEAK NO EVIL
=======================

Berg 0.3 introduces compilation, matchers and types to Berg, moving from its interpreted shape to a statically compiled one. 0.1 and 0.2 were designed with this in mind, but this will be the first true exercise of it.

1. Compilation
3. Compilation
    [ ] 


Round 1: Integers
=================

Integer
-------
[X] Integer literal
[X] Error: Integer starts with zero


Invalid UTF-8
-------------
[X] Error: Invalid UTF-8
[X] Error: Unsupported Character
[X] Error: Source Not Found
[X] Error: I/O open error
[X] Error: I/O read error
[X] Error: I/O directory join error for relative path

Tools
-----
[X] Unit Test Project
[X] Command Line

Round 2: Expressions
====================

Runtime
-------
[X] TypeChecker: Type, Number, IntegerLiteral.Value() -> Number
[X] TypeRuntime { Run(Type) -> Print Result }
[X] Integer tests

Math
----
[X] Add/Subtract Operators
[X] Multiply/Divide Operators
[X] Precedence
[X] Negative/Positive Operators
[X] Error: Divide By Zero
[X] Error: Unrecognized Operator

Round 3: Boolean Logic
======================

Boolean
-------
[X] true, false
[X] Error: cannot use true/false in math operator

Boolean Operators
----------------- 
[X] &&, ||, !
[X] anything other than false is true; && and || return determining argument, not necessarily "true"

Comparison Operators
--------------------
[X] ==, !=
[X] >, <, <=, >=
[X] Error: Non-number in comparison operator

Round 4: Expression Syntax
==========================

Parens
------
[X] Expression Operator "(", ")"
[X] Parse Error: Unclosed Paren
[X] Parse Error: Unopened Paren

Nothing
-------
[X] Type::Nothing
[X] Empty source file -> nothing
[X] Empty parentheses -> nothing
[X] Error: cannot apply operator [/*+-] to "nothing" (either left or right side or both)

Space
-----
[X] Whitespace, Tabs
[X] Compound Term Grouping
[X] Newlines; Record Line / Column Data

Round 5: Scope
==============

Statements
----------
[X] Statement Separator ";"
[X] Trailing Semicolon Acceptable
[X] Final Expression In Block is Return Value
[X] Newline Statement Separation
[X] Newline Statement Continuation

Variables
---------
*Variables* are accessible in the scope they were first defined and nowhere else.
[X] Variable Assignment (a = b)
[X] Variable Reassignment (a = b)
[X] Variable Reference (a)
[X] Error: No Such Variable
[ ] Error: Unused Variable Definition
[ ] Error: Reference Before Definition

Round 6: Blocks
===============

Blocks
------
[ ] {} creates a new block
[ ] Error: ) where expected }
[ ] Error: } where expected )

Block Scope
-----------
[ ] Variables in parent block are accessible and assignable
[ ] Variables declared *after* block, in parent scope, are inaccessible
[ ] Variables declared in sibling scopes are inaccessible to each other
[ ] Variables declared in child scope are inaccessible to parents

Block Laziness
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
*Fields* are variables that have been made public, and are thus accessible outside their scope.
[X] Expose field value (:a = b)
[X] Field usable in expression
[ ] "Unused variable error" does not apply to field
[ ] "Unused variable error" does not apply to field

Field Access
------------
[ ] Field access (a.b)
[ ] Error: no such field
[ ] Error: field not accessible in scope
[ ] Error: field not 

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

Call
----


Define and run functions

Round 6: Compilation
====================

Round 5: Error Propagation
==========================

Error Properties
----------------
[ ] Error is Berg object
[ ] Errors have arbitrary number of properties

Error Trail
-----------
[ ] Error given opportunity to pick up source or other information at each use

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
[ ] Parse Error: Line Too Long (4K characters)
[ ] Single-Line Comment
[ ] Errors: Invalid UTF-8, Comment Too Long. Denormalized OK. Unsupported Characters OK.

Visual Studio Code Extension
----------------------------
[ ] Syntax highlighting
[ ] Error reporting

Command Line
------------
[ ] Display error context
[ ] Display context with possible fixes

Fields
------
[ ] Field Definition (=)
[ ] Field Modification (+=, ++, etc.)
[X] Field Declaration
[X] "Missing" Value
[ ] Parse Error: Identifier Too Large
[X] Parse Error: Identifier Starts With Number
[ ] Parse Error: Identifier Must Be Immediately After ":"
[X] Parse Error: Identifier Required In Declaration
[ ] Object Extend On ";" and "\n" (combine / overwrite properties)

Field Reference
------------------
[X] Field Reference (Identifier)
[ ] "nothing"
[ ] Object.GetField(Identifier) -> Object
[ ] Error: Field not declared
[ ] Error: Field not declared - check for misspelled, give suggestion
[ ] Out Of Order Declaration ("[:]A: B; :B: 2")
[ ] Parse Error: Missing : in front of declaration

Field Assignment
-------------------
[ ] Field Assignment (=)
[ ] Parse Error: Missing Operand

Apply
-----
[ ] Apply Operator (Extend With Block) "F <+ A: 1" or "F <+ { A: 1; B: 2 }"
[ ] Because of multiple reasons! Figure out error dedup strategies here ...

Functions
=========

List
------
[ ] List Operator (,)
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
[ ] Field Access (Dot) Operator
[ ] Parse Error: Identifier Required For Field Access

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

