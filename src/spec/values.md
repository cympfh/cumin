# Primitive Values

__Values__ are Expressions.
In particular, primitive Values can be used in any context.

## Numbers

Natural numbers and integers are represented as decimal literal.

For examples,

```rust,no_run,noplayground
123
```

```rust,no_run,noplayground
-100000
```

Zero or positive numbers are Natural numbers by default,
and negative ones are Integers.

Floating numbers are denoted with period (`.`), like

```rust,no_run,noplayground
1.234
```

```rust,no_run,noplayground
-0.1
```

_NOTE_:
Following literals are invalid in v0.9.3: `.1`, `1.`.
You should write `0.1` and `1.0`.

## Strings

Strings are denoted by quoting double-quotation (`"`).

```rust,no_run,noplayground
"Hello, World"
```

### Escape

Escape with `\`.

```rust,no_run,noplayground
"\n\r\t\""
```

## Booleans

There are `true` and `false` as Booleans.
