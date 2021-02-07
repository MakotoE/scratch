use super::*;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// https://en.scratch-wiki.info/wiki/Scratch_File_Format
#[derive(PartialEq, Clone, Default, Debug)]
pub struct ScratchFile {
    pub project: Project,

    /// Filename to file contents
    pub images: HashMap<String, Image>,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub targets: Vec<Target>,
    pub monitors: Vec<Monitor>,
    pub extensions: Vec<String>,
    pub meta: Meta,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, Variable>,
    pub blocks: HashMap<BlockID, Block>,
    pub costumes: Vec<Costume>,
    #[serde(default)]
    pub layer_order: usize,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub size: f64,
    #[serde(default)]
    pub visible: bool,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            is_stage: false,
            name: String::new(),
            variables: HashMap::new(),
            blocks: HashMap::new(),
            costumes: Vec::new(),
            layer_order: 0,
            x: 0.0,
            y: 0.0,
            size: 0.0,
            visible: true,
        }
    }
}

impl Hash for Target {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_stage.hash(state);
        self.name.hash(state);
        sorted_entries(&self.variables).hash(state);
        sorted_entries(&self.blocks).hash(state);
        self.costumes.hash(state);
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.size.to_bits().hash(state);
    }
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        compare(&self, other)
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub id: String,
    pub value: Value,
    #[serde(default)]
    pub i_dont_know_what_this_does: bool,
}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        hash_value(&self.value, state);
        self.i_dont_know_what_this_does.hash(state);
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        compare(&self, other)
    }
}

fn compare<A, B>(a: A, b: B) -> bool
where
    A: Hash,
    B: Hash,
{
    let mut hasher_a = DefaultHasher::new();
    a.hash(&mut hasher_a);
    let mut hasher_b = DefaultHasher::new();
    b.hash(&mut hasher_b);
    hasher_a.finish() == hasher_b.finish()
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    pub next: Option<BlockID>,
    pub inputs: HashMap<String, Value>,
    pub fields: HashMap<String, Vec<Option<String>>>,
    pub top_level: bool,
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.opcode.hash(state);
        self.next.hash(state);

        for entry in sorted_entries(&self.inputs) {
            entry.0.hash(state);
            hash_value(&entry.1, state);
        }

        sorted_entries(&self.fields).hash(state);

        self.top_level.hash(state);
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        compare(&self, other)
    }
}

fn sorted_entries<K, V>(map: &HashMap<K, V>) -> Vec<(&K, &V)>
where
    K: std::cmp::Ord,
{
    let mut result: Vec<(&K, &V)> = map.iter().collect();
    result.sort_unstable_by(|a, b| a.0.cmp(b.0));
    result
}

fn hash_value<H>(value: &Value, state: &mut H)
where
    H: Hasher,
{
    value.to_string().hash(state)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub md5ext: Option<String>,
    pub asset_id: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
    #[serde(default)]
    pub bitmap_resolution: f64,
}

impl Default for Costume {
    fn default() -> Self {
        Self {
            name: String::new(),
            md5ext: None,
            asset_id: String::new(),
            rotation_center_x: 0.0,
            rotation_center_y: 0.0,
            bitmap_resolution: 1.0,
        }
    }
}

impl Hash for Costume {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.md5ext.hash(state);
        self.rotation_center_x.to_bits().hash(state);
        self.rotation_center_y.to_bits().hash(state);
    }
}

impl PartialEq for Costume {
    fn eq(&self, other: &Self) -> bool {
        compare(&self, other)
    }
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: String,
    pub mode: String,
    pub opcode: String,
    pub params: MonitorParams,
    pub sprite_name: Option<String>,
    pub value: Value,
    pub x: f64,
    pub y: f64,
    pub visible: bool,
    pub slider_min: f64,
    pub slider_max: f64,
    pub is_discrete: bool,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorParams {
    #[serde(rename = "VARIABLE")]
    pub variable: String,
}

#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Image {
    SVG(Vec<u8>),
    PNG(Vec<u8>),
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut vec_len = 0usize;
        match self {
            Image::SVG(v) => {
                vec_len = v.len();
                write!(f, "SVG(")?;
            }
            Image::PNG(v) => {
                vec_len = v.len();
                write!(f, "PNG(")?;
            }
        }

        if vec_len > 0 {
            write!(f, "[...]")?;
        } else {
            write!(f, "[]")?;
        }

        write!(f, ")")
    }
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Default, Hash)]
pub struct BlockID {
    id: [u8; 20],
}

impl Debug for BlockID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("BlockID { ")?;
        Display::fmt(self, f)?;
        f.write_str(" }")
    }
}

impl Display for BlockID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.id[..10]).map_err(|_| std::fmt::Error {})?)
    }
}

impl TryFrom<&str> for BlockID {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let mut id: [u8; 20] = [0; 20];
        let s_bytes = s.as_bytes();
        if s_bytes.len() == id.len() {
            id.copy_from_slice(s_bytes);
            Ok(Self { id })
        } else {
            Err(Error::msg("invalid string"))
        }
    }
}

impl Serialize for BlockID {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BlockID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = BlockID;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.try_into().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(BytesVisitor)
    }
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
            if name.ends_with(".svg") | name.ends_with(".png") {
                image_names.push(name.to_string());
            }
        }

        let mut images: HashMap<String, Image> = HashMap::new();
        for name in &image_names {
            let mut b: Vec<u8> = Vec::new();
            archive.by_name(name).unwrap().read_to_end(&mut b)?;
            let image = if name.ends_with(".svg") {
                Image::SVG(b)
            } else if name.ends_with(".png") {
                Image::PNG(b)
            } else {
                return Err(Error::msg("unrecognized file extension"));
            };
            images.insert(name.clone(), image);
        }

        Ok(Self { project, images })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_savefile() {
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

    mod block_id {
        use super::*;

        #[test]
        fn test_from_str() {
            {
                assert!(BlockID::try_from("").is_err());
            }
            {
                assert!(BlockID::try_from("a").is_err());
            }
            {
                let s = "G@pZX]3ynBGB)L`_LJk8";
                let id = BlockID::try_from(s).unwrap();
                assert_eq!(&id.to_string(), "G@pZX]3ynB");
            }
        }
    }
}
