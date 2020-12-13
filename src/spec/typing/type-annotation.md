# Type Annotation

## Typed Let

`let` statement may have type annotations.

```rust,no_run,noplayground
let x: Int = 100;
```

`cuminc` evaluates this in the following steps:

1. eval `100`
    - the type infered as `Nat` because it is a (non-negative) natural number.
2. natural cast
    - `x` is annotated as `Int`.
    - `Nat` can be casted to `Int` naturally.
    - get `100` as `Int`.
3. name it `x`

In the natural cast, `cuminc` doesn't coerce forcibly.
For example, `String` to `Int`, `Int` to `Nat`.
__NOTE__: If you need, `as`-cast coerce to other types.

The type annotation is optional.
If it is omitted, the step 2 will be skipped.

```rust,no_run,noplayground
let x = 100;
```

In this example, `x` is `Nat`.

## Typed Struct

In structs, all fields should be type annotated.

```rust,no_run,noplayground
struct S {
    x: Nat,
    y: Int,
    z: Array<String>,
}
```

When constructing struct values (applying), `cuminc` checks the types of applied values.

```rust,no_run,noplayground
S {
    x = 1,
    y = -2,
    z = ["cumin"],
}
```
