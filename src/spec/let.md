# let

The `let` statement gives names to data.

```rust,noplaypen
let x = 1 + 2;
```

This statement computes the expression `1 + 2` (the result is `3` of course),
and we call it `x`.
Frankly, this is variable.
After `x` is binded, you can use `x` as a value.

The last `;` is required.
This statement can be denoted by following BNF;

```
<let> ::= `let` <id> `=` <expression> `;`
        | `let` <id> `:` <type> `=` <expression> `;`
```
