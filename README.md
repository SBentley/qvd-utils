# Read Qlik Sense .qvd files

## Usage
```
from qvd import reader

df = reader.read('test.qvd')
print(df)
```

### Developing
Create a virtual env https://docs.python-guide.org/dev/virtualenvs/ and activate it.

Install maturin

run ```maturin develop``` to install python lib to the virtual env.
