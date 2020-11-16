# Introduction

**cumin** is a Configuration Language like JSON, YAML or TOML,
but this is Mini-Programmable, Structured and Typed.

cumin has [Rust](https://www.rust-lang.org/)-like syntax.

```rust,noplaypen
/// example.cumin

struct User {
    id: Int,
    name: String,
}

let names = [
    User(1, "cympfh"),
    User(2, "Taro"),
    User(3, "John"),
];

names
```

The compiler **cuminc** generates JSON from cumin.

```bash
$ cuminc ./example.cumin
[{"id":1,"name":"cympfh"},{"id":2,"name":"Taro"},{"id":3,"name":"John"}]
```
