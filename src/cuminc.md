# cuminc

**cuminc** is a compiler for cumin.

## Installation

To build from source code, **cargo** is required.
It is recommended to use [rustup](https://www.rust-lang.org/tools/install) to install cargo.

```bash
$ git clone git@github.com:cympfh/cumin.git
$ make install
$ export PATH=$PATH:$HOME/.cargo/bin/
$ which cuminc
```

## Usage

**cuminc** compiles cumin data and then generates JSON data.

```bash
$ cuminc <file.cumin>
$ cat <file.cumin> | cuminc
```

### Example

```bash
$ echo '{{three = 1 + 2}}' | cuminc
{"three":3}
```
