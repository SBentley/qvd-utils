from qvd import reader
import sys

args = sys.argv[1:]

for file in args:
    df = reader.read(file)
    print(df)

    dict = reader.read_to_dict(file)
    print(f'\n {dict}')
