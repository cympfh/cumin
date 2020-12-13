# blocks

`{}` makes a new scope block.
In blocks, you can write whole cumin data.

```
<block> ::= `{` <statements> <expression> `}`
<statements> ::= <statement> | <statement> <statements>
```

For example, following code is a valid cumin data.

```rust,no_run,noplayground
let x = 1;
x + 1
```

So, you can write

```rust,no_run,noplayground
let z = {
    let x = 1;
    x + 1
};
z
```

Here, `z` has Value `2`, and the `x` is invisible from outer.
This notation can make private variables.
