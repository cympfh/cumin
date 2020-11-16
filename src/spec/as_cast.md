# as-cast

With `as` keyword, values are casted to another types.

```rust,noplaypen
// Inter Number Types
let x = 1.0 as Int; // Float -> Int
let y = 2 as Float; // Nat -> Float

// Stringify
let x = 1.0 as String; // "1.0"
let y = -2 as String; // "-2"

// Parse String
let x = "1.0" as Float; // 1.0
let y = "-2" as Int; // -2
```

__NOTE__:
Type casting is a coercion procedure.
It sometimes occurs runtime errors.
