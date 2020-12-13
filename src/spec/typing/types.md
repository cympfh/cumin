# Types

## Primitive Types

There are following types in prior.

```rust,no_run,noplayground
Any
Nat
Int
Float
Bool
String
Array<_>
Option<_>
```

`Any` is the top type for any values.
This is convenient for gradual typing.
`_` is alias for `Any`.

`Nat` is for Natural Numbers (0 or positive integers), and `Int` is for Integers.

`Array` and `Option` have type parameter.
`<_>` is the placeholder.
In actual code, it should be filled `<_>` with some type.
For example, `Array<Int>` is an array of Int Values.
Type parameters can be nested.
`Array<Array<Option<Int>>>`
is an array of an array of option of Int Values.

## Custom Types

After you declared `struct`-s and `enum`-s, the names are new types.
The names will be the names of types.

```rust,no_run,noplayground
struct X {}

// `X` is a type now.

let x: X = X();
```
