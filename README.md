# cumin 🌿

is a Structured, Typed and Mini-Programmable Configuration Language.

## Documents

- [cympfh.cc/cumin, English](https://cympfh.cc/cumin)
- [v0.9.10 マニュアル, 日本語](https://zenn.dev/cympfh/books/cumin-book-v0910)

## Features

- Rust-like Syntax
- Structured
    - struct, enum
- Typed
    - Validated Data
- Mini-Programmable

## Example

```rust
struct UserRecord {
    id: Int,
    name: Option<String> = None,
    region: Region = Region::Unknown,
}

enum Region {
    Unknown,
    East,
    West,
}

[
    UserRecord(1, "cympfh", Region::East),
    UserRecord { id = 2, name = "Alan", region = Region::West, },
    UserRecord { id = 3, name = "Bob" },
    UserRecord { id = 4, region = Region::East },
]
```

## Compiler

Cumin Compiler `cuminc` converts to JSON from Cumin.

```bash
$ cuminc ./examples/names.cumin
[
  {
    "id": 1,
    "name": "cympfh",
    "region": "East"
  },
  {
    "id": 2,
    "name": "Alan",
    "region": "West"
  },
  {
    "id": 3,
    "name": "Bob",
    "region": "Unknown"
  },
  {
    "id": 4,
    "name": null,
    "region": "East"
  }
]
```

## For Vim Users

```vim
Plugin 'rust-lang/rust.vim'
au BufRead,BufNewFile *.cumin set filetype=cumin
au FileType cumin set syntax=rust
```

