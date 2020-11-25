#![allow(unused_imports)]
use bitvec::prelude::*;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use quick_xml::de::{from_str, DeError};
use qvd_structure::{Fields, QvdFieldHeader, QvdTableHeader, Symbol};
use serde::Deserialize;
use std::convert::TryInto;
use std::error::Error;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{self, Read};
use std::path::Path;
use std::str;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{collections::HashMap, fs::File};
use std::{env, fs};
pub mod qvd_structure;

fn main() {
    println!("start");
    let now = Instant::now();

    let file_name = env::args().nth(1).expect("No qvd file given in args");
    let xml: String = get_xml_data(&file_name).expect("Unable to locate xml section in qvd file");

    let binary_section_offset = xml.as_bytes().len();

    let qvd_structure: QvdTableHeader = from_str(&xml).unwrap();
    let mut symbol_map: HashMap<String, Symbol> = HashMap::new();
    let mut rows: HashMap<String , Symbol> = HashMap::new();

    if let Ok(mut f) = File::open(&file_name) {
        // Seek to the end of the XML section
        f.seek(SeekFrom::Start(binary_section_offset as u64))
            .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        let rows_start = qvd_structure.offset;
        let rows_end = buf.len();
        let rows_section = &buf[rows_start..rows_end];
        //println!("rows {:?}", rows_section);
        let record_byte_size = qvd_structure.record_byte_size;

        for field_header in qvd_structure.fields.headers {
            symbol_map.insert(
                field_header.field_name.clone(),
                get_symbols(&buf, &field_header),
            );
            //println!("{:?}", symbol_map[&field_header.field_name]);

            let pointers = get_row_indexes(&rows_section, &field_header, record_byte_size);
            let row = match_symbols_with_rows(
                &field_header.field_name,
                &symbol_map[&field_header.field_name],
                &pointers,
            );
            rows.insert(field_header.field_name, row);           
        }
        println!("{:?}" ,rows);
    }
    println!("time {}ms", now.elapsed().as_millis());
}

fn match_symbols_with_rows(field_name: &str, symbol: &Symbol, row_pointers: &Vec<i64>) -> Symbol {
    match symbol {
        Symbol::Strings(symbols) => {
            let mut rows: Vec<String> = Vec::new();
            for pointer in row_pointers {
                if *pointer == -1 {
                    rows.push(String::from("NULL"));
                }
                else {rows.push(symbols[*pointer as usize].clone());}
            }
            return Symbol::Strings(rows);
        }
        Symbol::Numbers(symbols) => {
            let mut rows: Vec<i64> = Vec::new();
            for pointer in row_pointers {
                if *pointer == -1 {
                    rows.push(0);
                }
                else {rows.push(symbols[*pointer as usize]);}
            }
            return Symbol::Numbers(rows);
        }
    }
}

fn get_symbols(buf: &[u8], field: &QvdFieldHeader) -> Symbol {
    let start = field.offset;
    let end = start + field.length;
    match buf[start] {
        4 | 5 | 6 => Symbol::Strings(process_string_symbols(&buf[start..end])),
        1 | 2 => {
            if field.length > 8 {
                Symbol::Numbers(process_number_symbols(&buf[start..end]))
            } else {
                Symbol::Numbers(Vec::new())
            }
        }
        _ => {
            let mut v: Vec<String> = Vec::new();
            //TODO: remove null string
            v.push(String::from("NULL"));
            Symbol::Strings(v)
        },
    }
}

fn get_row_indexes(buf: &[u8], field: &QvdFieldHeader, record_byte_size: usize) -> Vec<i64> {
    let mut cloned_buf = buf.clone().to_owned();
    let chunks = cloned_buf.chunks_mut(record_byte_size);
    let mut indexes: Vec<i64> = Vec::new();    
    // Reverse the bytes
    for chunk in chunks {      
        chunk.reverse();
        let bits = BitSlice::<Msb0, _>::from_slice(&chunk[..]).unwrap();
        let start = bits.len() - field.bit_offset;
        let end = bits.len() - field.bit_offset - field.bit_width;
        let binary = bitslice_to_vec(&bits[end..start]);
        let index = binary_to_u32(binary);
        if index == 0 {
            indexes.push(index as i64);
        } else {
            indexes.push((index as i32 + field.bias) as i64);
        }
    }
    indexes
}

fn binary_to_u32(binary: Vec<u8>) -> u32 {
    let mut sum: u32 = 0;
    for bit in binary {
        sum <<= 1;
        sum += bit as u32;
    }
    sum
}

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
            reader.read_until(0, &mut buffer).unwrap();
            let xml_string =
                str::from_utf8(&buffer[..]).expect("xml section contains invalid UTF-8 chars");
            Ok(xml_string.to_owned())
        }
        Err(e) => Err(e),
    }
}

fn process_string_symbols(buf: &[u8]) -> Vec<String> {
    let mut current_string = String::new();
    let mut strings: Vec<String> = Vec::new();

    let mut i = 0;
    while i < buf.len() {
        let byte = &buf[i];
        match byte {
            0 => {
                strings.push(current_string.clone());
                current_string.clear();
            }
            4 | b'\r' | b'\n' => (),
            5 => {
                // Skip the 4 bytes before string
                i += 5;
                continue;
            }
            6 => {
                // Skip the 8 bytes before string
                i += 9;
                continue;
            }
            _ => {
                let c = *byte as char;
                current_string.push(c);
            }
        }
        i += 1;
    }
    strings
}

// 8 bytes
pub fn process_number_symbols(buf: &[u8]) -> Vec<i64> {
    let mut numbers: Vec<i64> = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        let byte = &buf[i];
        match byte {
            1 => {
                let mut x = &buf[i + 1..i + 5];
                let value = x.read_i32::<BigEndian>().unwrap();
                numbers.push(value as i64);
                i += 5;
            }
            2 => {
                let mut x = &buf[i + 1..i + 9];
                let value = x.read_i64::<BigEndian>().unwrap();
                numbers.push(value);
                i += 9;
            }
            _ => {
                panic!("unexpected char at offset {} double", i);
            }
        }
    }
    numbers
}


// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_file<P>(filename: P) -> io::Result<io::BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;
    use byteorder::ByteOrder;
    use byteorder::LittleEndian;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_double() {
        let buf: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xA4, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0xA5,
        ];
        let res = process_number_symbols(&buf);
        let expected: Vec<i64> = vec![420, 421];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_int() {
        let buf: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00, 0x0A, 0x01, 0x00, 0x00, 0x00, 0x14];
        let res = process_number_symbols(&buf);
        let expected = vec![10, 20];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_mixed_numbers() {
        let buf: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xA4, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0xA5, 0x01, 0x00, 0x00, 0x00, 0x0A, 0x01, 0x00, 0x00, 0x00, 0x14,
        ];
        let res = process_number_symbols(&buf);
        let expected: Vec<i64> = vec![420, 421, 10, 20];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_string() {
        let buf: Vec<u8> = vec![
            4, 101, 120, 97, 109, 112, 108, 101, 32, 116, 101, 120, 116, 0, 4, 114, 117, 115, 116,
            0,
        ];
        let res = process_string_symbols(&buf);
        let expected = vec!["example text", "rust"];
        assert_eq!(expected, res);
    }

    #[test]
    fn test_mixed_string() {
        let buf: Vec<u8> = vec![
            4, 101, 120, 97, 109, 112, 108, 101, 32, 116, 101, 120, 116, 0, 4, 114, 117, 115, 116,
            0, 5, 42, 65, 80, 1, 49, 50, 51, 52, 0, 6, 1, 1, 1, 1, 1, 1, 1, 1, 100, 111, 117, 98,
            108, 101, 0,
        ];
        let res = process_string_symbols(&buf);
        let expected = vec!["example text", "rust", "1234", "double"];
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
    fn test_get_rows() {
        let mut buf: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x11, 0x01, 0x22, 0x02, 0x33, 0x13, 0x34, 0x14, 0x35,
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
