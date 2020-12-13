# Natural Cast

For types `S` and `T`,
we denote `S -> T` describing that `S` can be casted to `T` naturally.

And `S -> T` if and only if the following code is valid when `x` has type `S`:

```rust,no_run,noplayground
let y: T = x;  // when x:S.
```

There are two rules for natural cast.

## Numbers downcast

```rust,no_run,noplayground
Nat -> Int -> Float
```

__NOTE__: `->` is transitive.
The fact that `S -> T`, `T -> U` and `S -> U` are denoted as `S -> T -> U`.

Any natural numbers can be considered as integers or as floating numbers.
But the inverse doesn't hold on.

## All values are `Any`

For any type `S`, `S -> Any`.
