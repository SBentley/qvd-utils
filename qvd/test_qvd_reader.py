from qvd import qvd_reader
import pandas as pd
import numpy


class TestQvdReader():
    def test_read_shape(self):
        qvd = qvd_reader.read('test_files/AAPL.qvd')
        csv = pd.read_csv('test_files/AAPL.csv', float_precision='round_trip')
        assert qvd.shape == csv.shape

    def test_read_size(self):
        qvd = qvd_reader.read('test_files/AAPL.qvd')
        csv = pd.read_csv('test_files/AAPL.csv', float_precision='round_trip')
        print(qvd.columns)
        print(csv.columns)
        assert numpy.array_equal(qvd.columns, csv.columns)
