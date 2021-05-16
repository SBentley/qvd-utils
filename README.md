# Read Qlik Sense .qvd files
![Rust](https://github.com/SBentley/qvd-utils/workflows/Rust/badge.svg) ![Python package](https://github.com/SBentley/qvd-utils/workflows/Python%20package/badge.svg)
A library for reading Qlik Sense .qvd file format from Python, written in Rust. Files can be read to DataFrame or dictionary.

## Install
Install from PyPi https://pypi.org/project/qvd/

```pip install qvd```

## Usage
```
from qvd import reader

df = reader.read('test.qvd')
print(df)
```
![example](https://raw.githubusercontent.com/SBentley/qvd-utils/master/example.png)

### Developing
Create a virtual env https://docs.python-guide.org/dev/virtualenvs/ and activate it.

Install pandas via pip
Install maturin via pip

run ```maturin develop --release``` to install the generated python lib to the virtual env.

## Test
Run ```cargo test``` to run rust unit tests.
Run ```pytest test_qvd_reader.py``` to test python lib.
