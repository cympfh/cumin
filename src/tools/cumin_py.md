# cumin-py

**cumin-py** is a Python binding for cumin.

## Installation

Firstly **cargo** is required.

### From PyPI

```bash
pip install cumin-py
```

### From Github

```bash
$ git clone git@github.com:cympfh/cumin-py.git
$ pip install .
```

## Usage

From Python code,

```python
import cumin

data = cumin.loads("{{three = 1 + 2}}")
# {'three': 3}

data = cumin.load("./data.cumin")
# import file `data.cumin`
```
