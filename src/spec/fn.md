# Function

## Declaration

Functions are declared with `fn` keyword or `let` keyword.

```
<function> ::=
      `fn` <name> `(` <args> `)` `=` <expr> `;`
    | `let` <name> `(` <args> `)` `=` <expr> `;`

<name> ::= <identifier>

<args> ::=
      <empty>
    | <var> `:` <type>
    | <var> `:` <type> `,` <args>
```

### Examples

```rust,no_run,noplayground
struct S {
    x: Int,
}

fn f(x: Int) = S { x = x + 1 };
let g(x: Int) = S(x-1);

[f(2), g(2)]  // S{x=3}, S{x=1}
```

## Lexical Scopes

```rust,no_run,noplayground
let z = 1;

fn one() = z;  // this `z` is 1.

// `one` can be referred from here.

let z: String = "2";

// `two` cannot be used yet.
let x = two();
// ↑ ERROR!

fn two() = z;  // this `z` is "2" now.

{{
    a = one(),  // 1
    b = two(),  // "2"
}}
```
