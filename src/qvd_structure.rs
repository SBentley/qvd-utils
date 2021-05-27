use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct QvdTableHeader {
    #[serde(rename = "TableName")]
    pub table_name: String,
    #[serde(rename = "CreatorDoc")]
    pub creator_doc: String,
    #[serde(rename = "Fields")]
    pub fields: Fields,
    #[serde(rename = "NoOfRecords")]
    pub no_of_records: u32,
    #[serde(rename = "RecordByteSize")]
    pub record_byte_size: usize,
    #[serde(rename = "Offset")]
    pub offset: usize,
    #[serde(rename = "Length")]
    pub length: usize,
}

#[derive(Debug, Deserialize)]
pub struct Fields {
    #[serde(rename = "$value", default)]
    pub headers: Vec<QvdFieldHeader>,
}
#[derive(Debug, Deserialize)]
pub struct QvdFieldHeader {
    #[serde(rename = "FieldName")]
    pub field_name: String,
    #[serde(rename = "Offset")]
    pub offset: usize,
    #[serde(rename = "Length")]
    pub length: usize,
    #[serde(rename = "BitOffset")]
    pub bit_offset: usize,
    #[serde(rename = "BitWidth")]
    pub bit_width: usize,
    #[serde(rename = "Bias")]
    pub bias: i32,
}
