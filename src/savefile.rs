use super::*;
use serde::{Deserialize, Serialize};

// https://en.scratch-wiki.info/wiki/Scratch_File_Format
#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct ScratchFile {
    pub project: Project,
    pub images: Vec<String>,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
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
    pub costumes: Vec<Costume>,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub size: f64,
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
pub struct Costume {
    pub name: String,
    pub md5ext: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}

impl ScratchFile {
    pub fn parse<R>(file: R) -> Result<ScratchFile>
    where
        R: std::io::Read + std::io::Seek,
    {
        use std::io::Read;

        let mut archive = zip::ZipArchive::new(file)?;
        let project: Project = serde_json::from_reader(archive.by_name("project.json")?)?;

        let mut image_names: Vec<String> = Vec::new();
        for name in archive.file_names() {
            if name.ends_with(".svg") {
                image_names.push(name.to_string());
            }
        }

        let mut images: Vec<String> = Vec::new();
        for name in &image_names {
            let mut str = String::new();
            archive.by_name(name)?.read_to_string(&mut str)?;
            images.push(str);
        }

        Ok(Self{
            project,
            images,
        })
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
        let savefile = ScratchFile::parse(&file).unwrap();
        let target = &savefile.project.targets[1];
        assert_eq!(target.name, "Sprite1");
    }
}
