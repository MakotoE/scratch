use serde::{Serialize, Deserialize};
use wasm_bindgen::__rt::std::collections::HashMap;

// https://en.scratch-wiki.info/wiki/Scratch_File_Format
#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    pub targets: Vec<Target>,
    pub monitors: Vec<String>,
    pub extensions: Vec<String>,
    pub meta: Meta,
}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, (String, i64)>,
    pub blocks: HashMap<String, Block>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub inputs: HashMap<String, serde_json::Value>,
    pub top_level: bool,
}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}

impl SaveFile {
    pub fn parse() {
        use bytes::buf::BufExt;
        let buf: bytes::Bytes = bytes::Bytes::new();
        zip::read::read_zipfile_from_stream(&mut buf.reader()).unwrap();
    }
}
