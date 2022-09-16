from qvd import qvd_reader
import os
import pandas as pd
import numpy as np


class TestQvdReader():
    def test_read_shape(self):
        qvd = qvd_reader.read(f'{os.path.dirname(__file__)}/test_files/AAPL.qvd')
        csv = pd.read_csv(f'{os.path.dirname(__file__)}/test_files/AAPL.csv', float_precision='round_trip')
        assert qvd.shape == csv.shape

    def test_read_size(self):
        qvd = qvd_reader.read(f'{os.path.dirname(__file__)}/test_files/AAPL.qvd')
        csv = pd.read_csv(f'{os.path.dirname(__file__)}/test_files/AAPL.csv', float_precision='round_trip')
        assert np.array_equal(np.sort(qvd.columns, axis=0), np.sort(csv.columns, axis=0))
