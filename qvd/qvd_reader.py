from .qvd import read_qvd
import pandas as pd


def read(file_name):
    data = read_qvd(file_name)
    df = pd.DataFrame.from_dict(data)
    return df


def read_to_dict(file_name):
    data = read_qvd(file_name)
    return data
