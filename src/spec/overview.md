# Overview

Cumin data are composed of Statements and Expressions.
A cumin data must start with zero or more Statements and end with exact one Expression data.

Frankly, it is expressed like

```
(Statement)
    :
(Statement)
(Expression)
```

or denoted as `<cumin>` in the following (pseudo-)BNF.

```
<cumin> :: = <statements> <expression> | <expression>
<statements> ::= <statement> | <statement> <statements>
    where
        <statement> ::= (Statement)
        <expression> ::= (Expression)
```

As Statement, there are `struct`, `enum` and `let`.
`struct` and `enum` define new types.
`let` gives names for data, which are _variables_ we called.

An Expression represents a Value,
which can contain already defined types and variables.
For examples, number literals are Values and Expressions.
Arithmetic Expressions are Expressions (e.g. `(1 + x) / 2`).
