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

This is convenience but not type-safe.
If two or more dictionaries have same fields with same types,
it is chance to create struct.