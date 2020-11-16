# Overview

Cumin data are composed of Statements and Expressions.
A cumin data must start with zero or more statements and exact one expression data.

```
// cumin data
(Statement)
    :
(Statement)
(Expression)
```

As statement, there are `struct`, `enum` and `let`.
`struct` and `enum` define new types.
`let` gives names for data, which are _variables_ we called.

An expression represents a value,
which can contain already defined types and variables.
For examples, number literals are values and expressions.
Arithmetic expressions are expressions (e.g. `(1 + x) / 2`).
