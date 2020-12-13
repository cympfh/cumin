# let

## let Statement

The `let` Statement gives names to data.

```rust,no_run,noplayground
let x: Int = 1 + 2;
```

Type annotation is freely optional.

```rust,no_run,noplayground
let x = 1 + 2;
```

This Statement computes the expression `1 + 2` (the result is `3` of course),
and we call it `x`.
Frankly, this is a variable,
and you can use `x` as a Value.

The last `;` is required.
This Statement can be denoted by following BNF;

```
<let> ::= `let` <id> `=` <expression> `;`
        | `let` <id> `:` <type> `=` <expression> `;`
```

## shadowing

When some variables are defined already,
you can declare the same names with `let`.
New data shadows old data.

```rust,no_run,noplayground
let x = 1;
// Here, x is Nat 1.

let x = "hoge";
// Here, x is String "hoge".
```

Once variables are shadowed, they cannot be used.
