Syntactic Structure: Expressions
================================

Berg expressions are similar to most conventional languages: you can have prefix, postfix and infix operators.

From [Blocks](visual_structure_blocks.md):

- **Block**: rectangle in the code. May be indented. Contains one or more statements.
- **Statement**: complete expression taking up one or more lines in a block. Contains one or more phrases. Multiple statements in the same block *extend* each other, first to last. May be *continued* by another line or have a *child block* as an operand.
- **Phrase**: complete expression within a statement that may include multiple terms and operators. Multiple phrases in the same block are *applied* to each other, left to right.
- **Term**: Complete expressions within a phrase with no spaces between them. Contains atomics optionally separated by operators. Multiple atomics in the same term are *applied* to each other, left to right. Operations within a term are always higher precedence than operators between two terms.
- **Atomic**: single expression, either a numeric or string literal, a delimited expression (`{...}`, `(...)` or `[...]`)

Fixity
------

Different operators can have different variants: for example, `+` has infix, postfix and prefix variants. Variants are determined by whether they are next to an atomic or not. If an operator is immediately followed by an atomic on one side and not on the other, it is prefix or postfix depending on which side:

* `a + b`: infix (many b's)
* `a+b`: infix (many b's)
* `a+`: postfix (many b's) 
* `+b`: prefix (positive a)

Atomics
-------

Atomic terms are the *nouns* of Berg expressions. Numeric literals (`21.4`), string literals (`"hi"`), property references (`Foo`), inline declarations (`:Foo`) and delimited expressions (`(x+2)`) are all atomics. 

Operators
---------

*Operators* are math, logic and other symbols that indicate an operation to perform, and which expression to the left or right (or both) it should be performed in.

| Category        | Operators
|-----------------|-------------------------------------------------------------------
| Property Access | . .?
| Subscript       | [] (left)
| Assignment      | Assignment (left)
| Math            | * / + - &* &+ &- <Math>? &- (pre) - (pre) + (pre)
| Range           | [<]..[>]
| Comparison      | == != < > <= >= <=> <Comparison>?
| Match           | ! (post) ? (post) + (post) * (post)
| Boolean         | ! (pre) || && ?? ?: (left)
| Assignment      | : (in+post) = <Math>= <Math>?= <Boolean>= [&]++[?] [&]--[?]
| Ternary         | ?: (right)
| List            | , (in+post)
| Apply           | <apply>
| Statement       | ; (in+post)
| Delimited       | <extend> <apply block> () [] {} ?: (middle) "..." /.../


