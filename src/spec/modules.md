# Modules

## Module file

Module files must contain only statements.

```
<module> ::= <statements>
<statements> ::= <statement> | <statement> <statements>
```

## Importing modules

```rust,no_run,noplayground
use "./path/to/module.cumin";
```

The path will be read as

1. Absolute path
2. Relative path from the current directory
3. Relative path from the file
