# Type Checking

## Array

In general JSON, Arrays can contain various values.

```json
// In JSON, this is OK.
[1, 3.14, ["cumin"]]
```

But this is very strange and buggy.
In cumin, Arrays should contain values with same type.

```rust,no_run,noplayground
// In cumin, this is NG.
[1, 3.14, ["cumin"]]
```

This occurs and error

```
Error: Cannot infer type of Array([Nat(1), Float(3.14), Array(String, [Str("cumin")])]); Hint: Array cannot contain values with different types.
```

## Any Type

`Any` is the top type.

```rust,no_run,noplayground
let x: Any = 100;
```

This is ok. And don't worry. `cuminc` knows that `x` is `Nat` (because `100` is `Nat`).
`Any` is convenient in some cases.
This example is trivial, and it is equivalent to `let` statement without type annotation.

For example, `Array<Any>` is a type for __Something Arrays__.

```rust,no_run,noplayground
let xs: Array<Any> = [
    1, 2, 3
];
```

In this example,

1. The elements are all `Nat`.
2. `xs` is a something array `Array<_>`.
3. These facts conclude that `xs` is `Array<Nat>`.

Because `_` is an alias for `Any`, you can write

```rust,no_run,noplayground
let xs: Array<_> = [
    1, 2, 3
];
```

It is **unsafe** that struct fields are declared as `Any`.

```rust,no_run,noplayground
struct Data {
    data: Any,
}
```

Since `data` can be any values, followings are all valid.

```rust,no_run,noplayground
let x = Data {
    data = 1,  // Nat
};

let y = Data {
    data = 3.14,  // Float
};

let z = Data {
    data = ["cumin"]  // Array<String>
};
```

And `cuminc` knows only that `x`, `y` and `z` are just `Data`, and ignores the type of `data`.
So they have all same type!


### Hack: Array with various data.

In cumin v0.9.7, This is ok.

```rust,no_run,noplayground
struct Data {
    data: Any,
}

let x = Data {
    data = 1,  // Nat
};

let y = Data {
    data = 3.14,  // Float
};

let z = Data {
    data = ["cumin"]  // Array<String>
};

[x, y, z]
```

Because the last data is just `Array<Data>`.
__NOTE__: No warrantry to support this hack.
