# Option

If you want `null`-able values, `Option` is very suitable.
For any type `T`, `Option<T>` is valid type and it is null-able.
Null value can be denoted as `None`.

```rust,noplaypen
let name: Option<String> = None;
```

In other hand, not null values for null-able are denoted with `Some(_)`.

```rust,noplaypen
let name: Option<String> = Some("MGR");
```

`Some` is considered as a natural transformation `T -> Option<T>` representing **not** null values.
