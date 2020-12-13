# Struct

## Struct Declaration

You can introduce new types with `struct`.

```rust,no_run,noplayground
struct X {
    x: Int,
    n: Nat,
    s: String,
}
```

Each fields can have default Values.

```rust,no_run,noplayground
struct X {
    x: Int,
    n: Nat = 0,
    s: String,
}
```

_NOTE_:
Cumin always allows comma-trailing.
The last `,` is optional completely, but I recommend putting ','.

This Statement syntax can be denoted as `<struct>` in the following BNF.

```
<struct> ::= struct <id> { <fields> }
<fields> ::= <field> | <field> `,` | <field> `,` <fields>
<fields> ::= <id> `:` <type> | <id> `:` <type> `=` <expression>

where
    <id> ::= (identifier)
    <type> ::= (type name)
    <expression> ::= (Expression)
```

## Struct Values

After you declared structs, you can apply them.

For example, the previous struct `X` has three fields `x`, `n` and `s`.
You can create `X` Values by appling three Values.

```rust,no_run,noplayground
X(0, 123, "yellow")
```

Applied Values can be any Expression.

```rust,no_run,noplayground
let n = -3;
X(12 * 3, n, "yel" + "low")
```

This style is similar to the function apply in many Programming Languages.
Cumin has another style.

```rust,no_run,noplayground
X { s = "yellow", n = 123, x = 0 }
```

Because any fields are named, the appling Values can be in any order.
And you can omit the fields having default Values.

```rust,no_run,noplayground
X { s = "yellow", x = 0 }  // n is omitted, and the default Value `0` be applied.
```

