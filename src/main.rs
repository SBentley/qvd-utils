#![feature(seek_convenience)]
#![allow(unused_imports)]
use quick_xml::de::{from_str, DeError};
use serde::Deserialize;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{self, Read};
use std::path::Path;
use std::str;
use std::{collections::HashMap, fs::File};
use std::{env, fs};

extern crate quick_xml;
extern crate serde;

#[derive(Debug, Deserialize)]
struct QvdTableHeader {
    #[serde(rename = "TableName")]
    pub table_name: String,
    #[serde(rename = "CreatorDoc")]
    pub creator_doc: String,
    #[serde(rename = "Fields")]
    pub fields: Fields,
    #[serde(rename = "NoOfRecords")]
    pub no_of_records: u32,
    #[serde(rename = "RecordByteSize")]
    pub record_byte_size: u32,
    #[serde(rename = "Offset")]
    pub bit_offset: u32,
    #[serde(rename = "Length")]
    pub length: u32,
}

#[derive(Debug, Deserialize)]
struct Fields {
    #[serde(rename = "$value", default)]
    pub headers: Vec<QvdFieldHeader>,
}
#[derive(Debug, Deserialize)]
struct QvdFieldHeader {
    #[serde(rename = "FieldName")]
    pub field_name: String,
    #[serde(rename = "Offset")]
    pub offset: u32,
    #[serde(rename = "Length")]
    pub length: u32,
    #[serde(rename = "BitOffset")]
    pub bit_offset: u32,
    #[serde(rename = "BitWidth")]
    pub bit_width: u32,
    #[serde(rename = "Bias")]
    pub bias: i32,
}

enum Symbol {
    Strings(Vec<String>),
    Doubles(Vec<i64>),
    Ints(Vec<i32>),
}

fn main() {
    let mut xml_string = String::new();
    let file_name = env::args().nth(1).expect("No qvd file given in args");
    let source: &mut String = match read_file(&file_name) {
        Ok(mut reader) => {
            loop {
                let mut buffer = [0; 100];
                reader.read_exact(&mut buffer).unwrap();
                let buf_contents = match str::from_utf8(&buffer) {
                    Ok(s) => s,
                    Err(_) => "",
                };
                match buf_contents.find("</QvdTableHeader>") {
                    Some(offset) => {
                        let upto_end_element = &buf_contents[0..offset + "</QvdTableHeader>".len()];
                        xml_string.push_str(upto_end_element);
                        break;
                    }
                    None => {
                        xml_string.push_str(buf_contents);
                    }
                }
            }
            &mut xml_string
        }
        Err(_) => &mut xml_string,
    };
    // There is a line break, carriage return and a null terminator between the XMl and data
    let skip = 3;
    let binary_section_offset = source.as_bytes().len() + skip;
    let qvd_structure: QvdTableHeader = from_str(&source).unwrap();
    let mut symbol_map: HashMap<String, Symbol> = HashMap::new();

    if let Ok(mut f) = File::open(&file_name) {
        // Seek to the end of the XML section
        f.seek(SeekFrom::Start(binary_section_offset as u64))
            .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        let mut v: Vec<u32> = vec![0; 10];
        for field_header in qvd_structure.fields.headers {
            let start = field_header.offset as usize;
            let end = start + field_header.length as usize;
            //println!("start offset = {} - byte {}", start, buf[start]);
            match buf[start] {
                4 => {
                    symbol_map.insert(
                        field_header.field_name,
                        Symbol::Strings(process_string_to_offset(&buf[start..end])),
                    );
                }
                2 => process_double_to_offset(&buf[start..end]),
                _ => {}
            }
            let num = buf[start] as usize;
            v[num] += 1;
        }
        println!("{:?}", v);
    }
}

fn process_string_to_offset(buf: &[u8]) -> Vec<String> {
    let mut current_string = String::new();
    let mut strings: Vec<String> = Vec::new();
    for byte in buf {
        match byte {
            0 => {
                strings.push(current_string.clone());
                current_string.clear();
            }
            4 | b'\r' | b'\n' => (),
            _ => {
                let c = *byte as char;
                current_string.push(c);
            }
        }
    }

    return strings;
}

fn process_double_to_offset(buf: &[u8]) {}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_file<P>(filename: P) -> io::Result<io::BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file))
}
