from .qvd import read_qvd, read_qvd_from_buffer
import pandas as pd
import io


def read(file):
    data_dict = read_to_dict(file)
    df = pd.DataFrame.from_dict(data_dict)
    return df


def read_to_dict(file):
    if (isinstance(file, io.TextIOBase)
        or isinstance(file, io.BufferedIOBase)
        or isinstance(file, io.RawIOBase)
        or isinstance(file, io.IOBase)):
        try:
            unpacked_data = file.read()
        except UnicodeDecodeError as e:
            raise Exception("Supply a raw file access. Use mode \"rb\" instead of mode \"r\"")
    elif isinstance(file, bytes):
        unpacked_data = file
    elif isinstance(file, str):
        return read_qvd(file)
    else:
        raise Exception("Please supply a raw string or a file")
    result_data = read_qvd_from_buffer(unpacked_data)
    return result_data

