# cumin

Mini-Programmable Typed Configuration Language

## Example

```rust
// struct is a Fixed schema
struct Person {
    name: String,
    sex: Sex,
    age: Nat,
}

// "Male" or "Female" or "Other"
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
        Person { name="Xohn", sex=Sex::Other, age=1 },
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

```bash
# bash
$ cuminc -T json ./examples/names.cumin
{
    "names": [
        {"name": "John", "sex": "Male", "age": 17},
        {"name": "Xohn", "sex": "Other", "age": 1}
    ],
    "author": {"name": "cympfh", "sex": "Male", "age": 0}
}
```

### Query Command (like jq)

```bash
# bash
$ cq '.names[0]' ./examples/names.cumin
Person("John", Sex::Male, 17)

$ cq -r '.author.name' ./examples/names.cumin
cympfh
```
