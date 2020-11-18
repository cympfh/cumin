# operators

## Number Operators

| Name      | cumin | Example       | Math        |
|:---------:|:-----:|:-------------:|:-----------:|
| Addition  |  `+`  | `x + y`       | \\( x+y \\) |
| Subtract  |  `-`  | `x - y`       | \\( x-y \\) |
| Minus     |  `-`  | `-x`          | \\( -x \\)  |
| Multiply  |  `*`  | `x * y`       | \\( xy \\)  |
| Division  |  `/`  | `x / y`       | \\( x/y \\) |
| Power     |  `**` | `x ** y`      | \\( x^y \\) |
| Priority  | `()`  | `(x + y) * z` | \\( (x+y)\times z \\) |

There are three types for Numbers:
`Nat`, `Int` and `Float`.
The operations do implicit type casting.
For example, the result of `Nat + Float` has `Float`.
It is one-way `Nat -> Int -> Float`.

## Bool Operators

| Name      | cumin | Example       | Math               |
|:---------:|:-----:|:-------------:|:------------------:|
| And       |  `*`  | `a * b`       | \\( a \land b \\)  |
| Or        |  `+`  | `a + b`       | \\( a \lor b \\)   |
| Xor       |  `**` | `a ** b`      | \\( a \oplus b \\) |
| Not       |  `-`  | `-a`          | \\( \lnot a \\)    |

