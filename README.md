# cumin

Structured Typed Configuration Language

## Features

- Rust-like Syntax
- Structured
    - struct, enum
- Typed
    - Validated Data
- Mini-Programmable

## Example

```rust
// struct is a Fixed schema
struct Person {
    name: String,
    sex: Sex = Sex::Other,  // can have default value
    age: Nat,
}

// Enum Value
// Exact One of "Male", "Female" or "Other"
enum Sex {
    Male,
    Female,
    Other,
}

// Exporting the last data
// Here {{ ... }} is a Just Dictionary[fields => Any data]
{{

    // list of Person
    names = [
        Person("John", Sex::Male, 17),
        Person { name="Xohn", age=1 },  // Default Value used for Sex
    ],

    // Cumin by
    author = Person {
        name = "cympfh",
        sex = Sex::Male,
        age = 0,
    },

}}
```

### Convert to JSON

Cumin Compoler `cuminc` converts `cumin` to `JSON`.

```bash
# bash
$ cuminc ./examples/names.cumin
{
    "names": [
        {"name": "John", "sex": "Male", "age": 17},
        {"name": "Xohn", "sex": "Other", "age": 1}
    ],
    "author": {"name": "cympfh", "sex": "Male", "age": 0}
}
```

### Query Command (like jq, Feature Work)

```bash
# bash
$ cq '.names[0]' ./examples/names.cumin
Person("John", Sex::Male, 17)

$ cq -r '.author.name' ./examples/names.cumin
cympfh
```

## For Vim Users

```vim
Plugin 'rust-lang/rust.vim'
au BufRead,BufNewFile *.cumin set filetype=cumin
au FileType cumin set syntax=rust
```

