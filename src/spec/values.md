# Primitive Values

__Values__ are Expressions.
In particular, primitive Values can be used in any context.

## Numbers

Natural numbers and integers are represented as decimal literal.

For examples,

```rust,noplaypen
123
```

```rust,noplaypen
-100000
```

Zero or positive numbers are Natural numbers by default,
and negative ones are Integers.

Floating numbers are denoted with period (`.`), like

```rust,noplaypen
1.234
```

```rust,noplaypen
-0.1
```

_NOTE_:
Following literals are invalid in v0.9.3: `.1`, `1.`.
You should write `0.1` and `1.0`.

## Strings

Strings are denoted by quoting double-quotation (`"`).

```rust,noplaypen
"Hello, World"
```

### Escape

Escape with `\`.

```rust,noplaypen
"\n\r\t\""
```

