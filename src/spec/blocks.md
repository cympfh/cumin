# blocks

`{}` makes a new scope block.
In blocks, you can write whole cumin data.

```
<block> ::= `{` <statements> <expression> `}`
<statements> ::= <statement> | <statement> <statements>
```

For example, following code is a valid cumin data.

```rust,noplaypen
let x = 1;
x + 1
```

So, you can write

```rust,noplaypen
let z = {
    let x = 1;
    x + 1
};
z
```

The `z` has value `2`, and the `x` is invisible from outer.
This notation can make private variables.
