use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// https://en.scratch-wiki.info/wiki/Scratch_File_Format
#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    pub targets: Vec<Target>,
    pub monitors: Vec<String>,
    pub extensions: Vec<String>,
    pub meta: Meta,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, (String, f64)>,
    pub blocks: HashMap<String, Block>,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub inputs: HashMap<String, serde_json::Value>,
    pub fields: HashMap<String, Vec<String>>,
    pub top_level: bool,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}

impl SaveFile {
    pub fn parse<R>(file: R) -> Result<SaveFile>
    where
        R: std::io::Read + std::io::Seek,
    {
        let mut archive = zip::ZipArchive::new(file)?;
        let project = archive.by_name("project.json")?;
        Ok(serde_json::from_reader(project)?)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn savefile() {
        let dir = std::path::Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_saves")
            .join("say.sb3");
        let file = std::fs::File::open(dir).unwrap();
        let savefile = SaveFile::parse(&file).unwrap();
        let target = &savefile.targets[1];
        assert_eq!(target.name, "Sprite1");
    }
}
