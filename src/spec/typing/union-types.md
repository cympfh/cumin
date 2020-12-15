# Union Types

## `type` statement

Union Types are declared with `type` keyword and variants are described one or more types separated by `|`.

```rust,no_run,noplayground
type T = Int | String;
```

Using this `T`,
`Int` values and `String` values can be same type values.
This is convenient to construct an array of `Int` or `String` (i.e. `Array<T>`).

You can represent more complicated types with `struct`.

```rust,no_run,noplayground
struct A { a: Nat }
struct B { b: String }
struct C { c: Array<String> }
type T = A | B | C;
```

### NOTE

Union Types is like union-sets, and **not** Sum Types (or disjoint union).
For example,

```rust,no_run,noplayground
type T = Int | Int;
```

this equals to just `Int` exactly,
because nobody distinguish left `Int` from right one.

### NOTE

Union Types define subtypes.
In primitive, cumin defines the following subtype system:

```
Nat <: Int <: Float
```

and induces the following implicit type-cast:

```
Nat -> Int -> Float
```

And, when `type T = A | B;`,

```
A <: T,
B <: T
```

holds on.
So, you can cast `A -> T` and `B -> T`, but we don't cast them implicitly.
In next section, we will show how to cast them.

## Casting for Union Types

The names of union types can be **casting functions** (or injection).

```rust,no_run,noplayground
type T = Int | String;

let x: Int = 2;
let t = T(x);  // Int -> T

let s = T("hello");  // String -> T
```

```rust,no_run,noplayground
struct A { a: Nat }
struct B { b: String }
struct C { c: Array<String> }
type T = A | B | C;

let t = T(A(1));  // A -> T
let u = T(C { c = [] });  // C -> T
```

If you mind of the mount of parenthesis, we prepared a syntax-sugar.

## Composite Applying Syntax

`A.B(x)` will be `A(B(x))`.
`A.B{x = y}` will be `A(B{x=y})`.

```rust,no_run,noplayground
struct A { a: Nat }
struct B { b: String }
struct C { c: Array<String> }
type T = A | B | C;

let t = T.A(1);  // T(A(1))
let u = T.C{c=[]};  // T(C { c = [] })
```

Yay!

