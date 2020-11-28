from qvd import reader

df = reader.read('test_qvd.qvd')
print(df)

dict = reader.read_to_dict('test_qvd_null.qvd')
print(dict)
