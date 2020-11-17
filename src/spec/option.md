# Option

If you want _null_-able Values, `Option` is very suitable.
For any type `T`, `Option<T>` is valid type and it is _null_-able.
Null Value can be denoted as `None`.

```rust,noplaypen
let name: Option<String> = None;
```

In other hand, not _null_ Values for _null_-able are denoted with `Some(_)`.

```rust,noplaypen
let name: Option<String> = Some("MGR");
```

`Some` is considered as a natural transformation `T -> Option<T>` representing **not** _null_ Values.
