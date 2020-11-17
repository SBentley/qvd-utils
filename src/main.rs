#![feature(seek_convenience)]
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{self, BufReader, Read, Write};
use std::str;
use std::{io::BufRead, path::Path};

extern crate quick_xml;
extern crate serde;

use quick_xml::de::{from_str, DeError};

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
    #[serde(rename = "BitOffset")]
    pub bit_offset: u32,
    #[serde(rename = "BitWidth")]
    pub bit_width: u32,
    #[serde(rename = "Bias")]
    pub bias: i32,
}

fn main() {
    let mut xml_string = String::new();
    let mystr = String::from("sam");
    let source: &mut String = match env::args().nth(1) {
        Some(file_name) => {
            // File hosts must exist in current path before this produces output
            if let Ok(mut reader) = read_lines(file_name) {
                // Consumes the iterator, returns an (Optional) String
                loop {
                    let mut buffer = [0; 1000];
                    reader.read(&mut buffer[..]).unwrap();
                    let buf_contents = match str::from_utf8(&buffer) {
                        Ok(s) => s,
                        Err(_) => "",
                    };
                    //println!("{}", buf_contents);
                    match buf_contents.find("</QvdTableHeader>") {
                        Some(offset) => {
                            println!("Offset: {}", offset);
                            let upto_end_element = &buf_contents[0..5 + "</QvdTableHeader>".len()];
                            println!("{}", upto_end_element);
                            xml_string.push_str(upto_end_element);
                            break;
                        }
                        None => {xml_string.push_str(buf_contents);}
                    }                    
                }
            }
            &mut xml_string
        }
        None => &mut xml_string,
    };
    println!("xml-string {}", source);
    let mut file = File::open("nice.xml").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf);
    let table_header: QvdTableHeader = from_str(&buf).unwrap();

    //let table_header: QvdTableHeader = from_str(&source).unwrap();
    //println!("{:?}", table_header);
    //println!("{}", table_header.fields.headers.len());
    println!("xml-string-len {}", xml_string.len());
    println!("xml-string-cap {}", xml_string.capacity());
    match env::args().nth(1) {
        Some(file_name) => {
            if let Ok(mut f) = File::open(file_name) {
                println!("before seek: {}", f.stream_position().unwrap());
                f.seek(SeekFrom::Start(35819)).unwrap();
                println!("after seek: {}", f.stream_position().unwrap());
                let mut i = 0;
                for byte in f.bytes() {
                    //println!("looping: {}", f.stream_position().unwrap());
                    println!("{}", byte.unwrap() as char);
                    i += 1;
                    if i == 10 {
                        break;
                    }
                }
            }
        }
        None => {}
    };
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file))
}
