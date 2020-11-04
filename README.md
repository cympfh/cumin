# cumin

Mini-Programmable Configuration Language

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
]

// Cumin by
let author = Person {
    name = "cympfh",
    sex = Sex::Male,
    age = 0,
}
```

```bash
# bash
$ cq '.names[0]' ./examples/name_list.cumin
Person("John", Sex::Male, 17)

$ cq -r '.author.name' ./examples/name_list.cumin
cympfh
```

```python
# python
import cumin

conf = cumin.load('./examples/name_list.cumin')
for person in conf['names']:
    if person['age'] > 20:
        print(person['name'])
```
