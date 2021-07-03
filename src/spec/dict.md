# Dictionary

Dictionary is a Value data which can have any fields with any types.
It is denoted by quoting with `{{` and `}}`.

```rust,no_run,noplayground
{{
    x = 1,
    y = -2.3,
    s = "yellow",
}}
```

If you need, each fields can have type-annotation optionally.


```rust,no_run,noplayground
{{
    x: Int = 1,
    y = -2.3,
    s: String = "yellow",
}}
```

This is sometimes convenience but not type-safe.
If two or more dictionaries have same fields with same types,
it is chance to define a struct.
