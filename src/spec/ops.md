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

| Name      | cumin   | Example     | Math               |
|:---------:|:-------:|:-----------:|:------------------:|
| And       |  `and`  | `x and y`   | \\( a \land b \\)  |
| Or        |  `or`   | `x or y`    | \\( a \lor b \\)   |
| Xor       |  `xor`  | `x xor y`   | \\( a \oplus b \\) |
| Not       |  `not`  | `not x`     | \\( \lnot a \\)    |

