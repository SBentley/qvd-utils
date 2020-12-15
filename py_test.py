from qvd import qvd_reader
import sys

args = sys.argv[1:]

for file in args:
    df = qvd_reader.read(file)
    print(df)

#    print(df['DEAL_PRICE'])
    #dict = reader.read_to_dict(file)
    #print(f'\n {dict}')
