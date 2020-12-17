from qvd import qvd_reader
import pandas as pd
import pytest


class TestQvdReader():
    def test_read_shape(self):
        qvd = qvd_reader.read('AAPL.qvd')
        csv = pd.read_csv('AAPL.csv', float_precision='round_trip')
        assert qvd.shape == csv.shape