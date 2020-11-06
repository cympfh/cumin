# cumin

Mini-Programmable Typed Configuration Language

## Example

```rust
struct Person {
    name: String,
    sex: Sex,
    age: Nat,
}

enum Sex {
    Male,
    Female,
    Other,
}

// list of Person
let names = [
    Person("John", Sex::Male, 17),
    Person { name="Xohn", sex=Sex::Other, age=1 },
];

// Cumin by
let author = Person {
    name = "cympfh",
    sex = Sex::Male,
    age = 0,
};
```

### Query Command (like jq)

```bash
# bash
$ cq '.names[0]' ./examples/name_list.cumin
Person("John", Sex::Male, 17)

$ cq -r '.author.name' ./examples/name_list.cumin
cympfh
```

### Convert to JSON

```bash
# bash
$ cuminc -T json ./examples/name_list.cumin
{
    "names": [
        {"name": "John", "sex": "Male", "age": 17},
        {"name": "Xohn", "sex": "Other", "age": 1}
    ],
    "author": {"name": "cympfh", "sex": "Male", "age": 0}
}
```
