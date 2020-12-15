# Read Qlik Sense .qvd files
![Rust](https://github.com/SBentley/qvd-utils/workflows/Rust/badge.svg)

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

Install maturin

run ```maturin develop``` to install python lib to the virtual env.
