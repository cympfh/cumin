# Types

## Primitive Types

There are following types in prior.

```rust,noplaypen
Nat
Int
Float
String
Array<_>
Option<_>
```

where `Nat` is for Natural Numbers.
`Array` and `Option` have type parameter.
In actual code, it should be filled `<_>` with some type.
For example, `Array<Int>` is an array of Int Values.
Type parameters can be nested.
`Array<Array<Option<Int>>>`
is an array of an array of option of Int Values.

## Custom Types

After you declared `struct`s and `enum`s, the names are new types.

```rust,noplaypen
struct X {}

// `X` is a type.

let x: X = X();
```
