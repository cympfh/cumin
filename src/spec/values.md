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
You can omit leading zeros.
For example, `.1` is `0.1`, `1.` is `1.0`.

### Number Types

There are 3 types for numbers: `Nat`, `Int` and `Float`.

`Nat` is Natural Numbers. It is zero or positive integers.
`Int` is Integer Numbers.
`Float` is Floating Numbers (pseudo-Real Numbers).

## Strings

Strings are denoted by quoting double-quotation (`"`).

```rust,no_run,noplayground
"Hello, World"
```

### Type

`String` is the type for Strings.

### Escape

Escape with `\`.

```rust,no_run,noplayground
"\n\r\t\""
```

## Booleans

There are `true` and `false` as Boolean Values.
No other values doesn't exist.

### Type

`Bool` is the type for Booleans.
