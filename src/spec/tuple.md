# Tuple

Tuples are structures which can have different type values.

```rust,no_run,noplayground
let x = (1, "str");  // (Nat, String)
```

`x` is a pair of `Nat` and `String`.
The `(Nat, String)` is the type of `x`.

If you aren't familiar to tuples, this is considered as a kind of struct,
which has no field names.

```rust,no_run,noplayground
struct Anonymous {
    __field_0: Int,
    __field_1: String,
}
let x = Anonymous(1, "str");
```
