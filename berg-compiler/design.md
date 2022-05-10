Interpreter
-----------

Berg sees each function APPLY as an object (and each object as a function APPLY). Each function manages its own local variables, and functions have pointers to
their parent function (lexically). A function does not exit until its child functions have completed. This means that the stack can be used for both the
local variables, and for function calls themselves.

### Rust Objects

Rust's lifetime system is key to making sure we don't mess this up. Each
evaluation, therefore, lives on the stack, and its children have lifetimes tied
to it. Each operation we do has an object:

- RootRef: contains the root variables everyone shares.
- SourceEvaluator: parses and then evaluates a source text. While this step creates no variables, it does keep the ast around and therefore must be accounted for with lifetimes.
- BlockEvaluator: creates a scope for variables, and runs any expressions inside it.
- ExpressionEvaluator: runs an expression. Does not create scope.

When an expression runs, it returns a value on the stack, with the lifetime of the child itself. The parent may either use the value immediately, get a local copy of it (in the case of bool/string/etc.) or wrap it up with the ExpressionEvaluator in a ScopedValue (in the case that a local copy is impossible--for example, if the value is an un-executed block).

### `Val` and `Value`

`Value` is the primary interface Rust objects implement to interact with Berg. It allows the value to perform operations (infix, postfix, prefix). This will likely change so that infix/postfix/prefix operators are *values* that can be called, but we're trying not to rewrite the *whole* world at once.

`Val` is a concrete value that can hold any Berg value. It has the same operations as Value (just passes them on to children) and stores primitive values efficiently without neglecting pure Rust objects. Values have an explicit lifetime on their data so that Rust can help us know when to transform the. Things you can do with a `Val`:

- `val.downcast::<bool>() -> bool`
- `val.downcast_ref::<bool>() -> &bool`
- `val.downcast_mut::<bool>() -> &mut bool`
- `val.evaluate_lfix_expression(PLUS, &right_hand_side) -> Val`
- `evaluator.evaluate_extended(val: Val<'parent>, to_scope: Val<'scope>) -> Val<'scope>` - this takes the value and wraps it up
