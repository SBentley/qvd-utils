use bitvec::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::{prelude::*, types::PyDict};
use quick_xml::de::from_str;
use qvd_structure::{QvdFieldHeader, QvdTableHeader};
use std::io::SeekFrom;
use std::io::{self, Read};
use std::path::Path;
use std::str;
use std::{collections::HashMap, fs::File};
use std::{convert::TryInto, io::prelude::*};
pub mod qvd_structure;

#[pymodule]
fn qvd(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_qvd, m)?)?;

    Ok(())
}

#[pyfunction]
fn read_qvd<'a>(py: Python, file_name: String) -> PyResult<Py<PyDict>> {
    let xml: String = get_xml_data(&file_name).expect("Error reading file");
    let dict = PyDict::new(py);
    let binary_section_offset = xml.as_bytes().len();

    let qvd_structure: QvdTableHeader = from_str(&xml).unwrap();
    let mut symbol_map: HashMap<String, Vec<Option<String>>> = HashMap::new();

    if let Ok(mut f) = File::open(&file_name) {
        // Seek to the end of the XML section
        f.seek(SeekFrom::Start(binary_section_offset as u64))
            .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        let rows_start = qvd_structure.offset;
        let rows_end = buf.len();
        let rows_section = &buf[rows_start..rows_end];
        let record_byte_size = qvd_structure.record_byte_size;

        for field in qvd_structure.fields.headers {
            symbol_map.insert(
                field.field_name.clone(),
                get_symbols_as_strings(&buf, &field),
            );
            let symbol_indexes = get_row_indexes(&rows_section, &field, record_byte_size);
            let column =
                match_symbols_with_indexes(&symbol_map[&field.field_name], &symbol_indexes);
            dict.set_item(field.field_name, column).unwrap();
        }
    }
    Ok(dict.into())
}

fn match_symbols_with_indexes(
    symbols: &Vec<Option<String>>,
    pointers: &Vec<i64>,
) -> Vec<Option<String>> {
    let mut cols: Vec<Option<String>> = Vec::new();
    for pointer in pointers.iter() {
        if symbols.len() == 0 {
            continue;
        } else if *pointer < 0 {
            cols.push(None);
        } else {
            cols.push(symbols[*pointer as usize].clone());
        }
    }
    return cols;
}

fn get_symbols_as_strings(buf: &[u8], field: &QvdFieldHeader) -> Vec<Option<String>> {
    let start = field.offset;
    let end = start + field.length;
    let mut current_string = String::new();
    let mut strings: Vec<Option<String>> = Vec::new();

    let mut i = start;
    while i < end {
        let byte = &buf[i];
        // Check first byte of symbol. This is not part of the symbol but tells us what type of data to read.
        match byte {
            // Strings are null terminated
            0 => {
                strings.push(Some(current_string.clone()));
                current_string.clear();
                i += 1;
            }
            1 => {
                // 4 byte integer
                let target_bytes = buf[i+1..i + 5].to_vec();
                let byte_array: [u8; 4] = target_bytes.try_into().unwrap();
                let numeric_value = i32::from_le_bytes(byte_array);
                strings.push(Some(numeric_value.to_string()));
                i += 5;
            }
            2 => {
                // 4 byte double
                let target_bytes = buf[i + 1..i + 9].to_vec();
                let byte_array: [u8; 8] = target_bytes.try_into().unwrap();
                let numeric_value = f64::from_le_bytes(byte_array);
                strings.push(Some(numeric_value.to_string()));
                i += 9;
            }
            4 | b'\r' | b'\n' => {
                i += 1;
            } // Beginning of a null terminated string
            5 => {
                // 4 bytes of unknown followed by null terminated string
                // Skip the 4 bytes before string
                i += 5;
                continue;
            }
            6 => {
                // 8 bytes of unknown followed by null terminated string
                // Skip the 8 bytes before string
                i += 9;
                continue;
            }
            _ => {
                // Part of a string
                let c = *byte as char;
                current_string.push(c);
                i += 1;
            }
        }
    }
    strings
}

// Retrieve bit stuffed data. Each row has index to value from symbol map.
fn get_row_indexes(buf: &[u8], field: &QvdFieldHeader, record_byte_size: usize) -> Vec<i64> {
    let mut cloned_buf = buf.clone().to_owned();
    let chunks = cloned_buf.chunks_mut(record_byte_size);
    let mut indexes: Vec<i64> = Vec::new();
    for chunk in chunks {
        // Reverse the bytes in the record
        chunk.reverse();
        let bits = BitSlice::<Msb0, _>::from_slice(&chunk[..]).unwrap();
        let start = bits.len() - field.bit_offset;
        let end = bits.len() - field.bit_offset - field.bit_width;
        let binary = bitslice_to_vec(&bits[end..start]);
        let index = binary_to_u32(binary);
        indexes.push((index as i32 + field.bias) as i64);
    }
    indexes
}

// Slow
fn binary_to_u32(binary: Vec<u8>) -> u32 {
    let mut sum: u32 = 0;
    for bit in binary {
        sum <<= 1;
        sum += bit as u32;
    }
    sum
}

// Slow
fn bitslice_to_vec(bitslice: &BitSlice<Msb0, u8>) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    for bit in bitslice {
        let val = match bit {
            true => 1,
            false => 0,
        };
        v.push(val);
    }
    v
}

fn get_xml_data(file_name: &String) -> Result<String, io::Error> {
    match read_file(&file_name) {
        Ok(mut reader) => {
            let mut buffer = Vec::new();
            // There is a line break, carriage return and a null terminator between the XMl and data
            // Find the null terminator
            reader
                .read_until(0, &mut buffer)
                .expect("Failed to read file");
            let xml_string =
                str::from_utf8(&buffer[..]).expect("xml section contains invalid UTF-8 chars");
            Ok(xml_string.to_owned())
        }
        Err(e) => Err(e),
    }
}

fn read_file<P>(filename: P) -> io::Result<io::BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double() {
        let buf: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x7a, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x50, 0x7a, 0x40,
        ];
        let field = QvdFieldHeader {
            length: buf.len(),
            offset: 0,
            field_name: String::new(),
            bias: 0,
            bit_offset: 0,
            bit_width: 0,
        };
        let res = get_symbols_as_strings(&buf, &field);
        let expected: Vec<Option<String>> = vec![Some(420.0.to_string()), Some(421.0.to_string())];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_int() {
        let buf: Vec<u8> = vec![0x01, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x14, 0x00, 0x00, 0x00];
        let field = QvdFieldHeader {
            length: buf.len(),
            offset: 0,
            field_name: String::new(),
            bias: 0,
            bit_offset: 0,
            bit_width: 0,
        };
        let res = get_symbols_as_strings(&buf, &field);
        let expected = vec![Some(10.0.to_string()), Some(20.0.to_string())];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_mixed_numbers() {
        let buf: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x7a, 0x40, 
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x7a, 0x40,
            0x01, 0x01, 0x00, 0x00, 0x00, 
            0x01, 0x02, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x37, 0x30, 0x30, 0x30, 0x00,
            0x06, 0x00,0x00,0x00, 0x00,0x00,0x00,0x00,0x00, 0x38, 0x36, 0x35, 0x2e, 0x32, 0x00
        ];
        let field = QvdFieldHeader {
            length: buf.len(),
            offset: 0,
            field_name: String::new(),
            bias: 0,
            bit_offset: 0,
            bit_width: 0,
        };
        let res = get_symbols_as_strings(&buf, &field);
        let expected: Vec<Option<String>> = vec![
            Some(420.to_string()),
            Some(421.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(7000.to_string()),
            Some(865.2.to_string())
        ];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_string() {
        let buf: Vec<u8> = vec![
            4, 101, 120, 97, 109, 112, 108, 101, 32, 116, 101, 120, 116, 0, 4, 114, 117, 115, 116,
            0,
        ];
        let field = QvdFieldHeader {
            length: buf.len(),
            offset: 0,
            field_name: String::new(),
            bias: 0,
            bit_offset: 0,
            bit_width: 0,
        };
        let res = get_symbols_as_strings(&buf, &field);
        let expected = vec![Some("example text".into()), Some("rust".into())];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_mixed_string() {
        let buf: Vec<u8> = vec![
            4, 101, 120, 97, 109, 112, 108, 101, 32, 116, 101, 120, 116, 0, 4, 114, 117, 115, 116,
            0, 5, 42, 65, 80, 1, 49, 50, 51, 52, 0, 6, 1, 1, 1, 1, 1, 1, 1, 1, 100, 111, 117, 98,
            108, 101, 0,
        ];
        let field = QvdFieldHeader {
            length: buf.len(),
            offset: 0,
            field_name: String::new(),
            bias: 0,
            bit_offset: 0,
            bit_width: 0,
        };
        let res = get_symbols_as_strings(&buf, &field);
        let expected = vec![
            Some("example text".into()),
            Some("rust".into()),
            Some("1234".into()),
            Some("double".into()),
        ];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_bitslice_to_vec() {
        let mut x: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x11, 0x01, 0x22, 0x02, 0x33, 0x13, 0x34, 0x14, 0x35,
        ];
        let bits = BitSlice::<Msb0, _>::from_slice(&mut x[..]).unwrap();
        let target = &bits[27..32];
        let binary_vec = bitslice_to_vec(&target);

        let mut sum: u32 = 0;
        for bit in binary_vec {
            sum <<= 1;
            sum += bit as u32;
        }
        assert_eq!(17, sum);
    }

    #[test]
    fn test_get_row_indexes() {
        let buf: Vec<u8> = vec![
            0x00, 0x14, 0x00, 0x11, 0x01, 0x22, 0x02, 0x33, 0x13, 0x34, 0x24, 0x35,
        ];
        let field = QvdFieldHeader {
            field_name: String::from("name"),
            offset: 0,
            length: 0,
            bit_offset: 10,
            bit_width: 3,
            bias: 0,
        };
        let record_byte_size = buf.len();
        let res = get_row_indexes(&buf, &field, record_byte_size);
        let expected: Vec<i64> = vec![5];
        assert_eq!(expected, res);
    }
}
