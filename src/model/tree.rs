use crate::model::id::Id;
use nom::IResult;
use nom::bytes::complete::{take_while, tag, take_until, take};
use nom::character::{is_digit, is_space};
use crate::errors::*;
use std::str::FromStr;
use nom::combinator::{map_res, map};



use nom::multi::many0;
use crate::model::object::*;
use std::convert::TryFrom;
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    BLOB,
    COMMIT,
    TREE,
}

impl FromStr for EntryType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "commit" => Ok(EntryType::COMMIT),
            "blob" => Ok(EntryType::BLOB),
            "tree" => Ok(EntryType::TREE),
            _ => Err(ErrorKind::ParseError.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    id: Id,
    entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn entries(&self) -> &[TreeEntry] {
        self.entries.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileMode {
    FILE,
    EXE,
    DIR,
    LINK,
    SUBMODULE
}

impl FileMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileMode::DIR => "40000",
            FileMode::FILE => "100644",
            FileMode::EXE => "100755",
            FileMode::LINK => "120000",
            FileMode::SUBMODULE => "160000"
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeEntry {
    mode: FileMode,
    id: Id,
    name: String,
}

impl TreeEntry {
    pub fn mode(&self) -> FileMode {
        self.mode.clone()
    }
    pub fn entry_type(&self) -> EntryType {
        match self.mode {
            FileMode::DIR => EntryType::TREE,
            FileMode::LINK | FileMode::EXE | FileMode::FILE => EntryType::BLOB,
            FileMode::SUBMODULE => EntryType::COMMIT
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn id(&self) -> &Id {
        &self.id
    }
}

impl FromStr for FileMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "40000" => Ok(FileMode::DIR),
            "100644" => Ok(FileMode::FILE),
            "100755" => Ok(FileMode::EXE),
            "120000" => Ok(FileMode::LINK),
            "160000" => Ok(FileMode::SUBMODULE),
            _ => Err(ErrorKind::BadMode.into()),
        }
    }
}
fn parse_from_str<T: FromStr>(input: &[u8]) -> Result<T> {
    let str = std::str::from_utf8(input)?;
    T::from_str(str).map_err(|_| ErrorKind::ParseError.into())
}

fn parse_mode(input: &[u8]) -> IResult<&[u8], FileMode> {
     map_res(
        take_while(is_digit),
        parse_from_str::<FileMode>
    )(input)
}

fn parse_id(input: &[u8]) -> IResult<&[u8], Id> {
    map(
        take(20u8),
        Id::new
    )(input)
}

fn parse_name(input: &[u8]) -> IResult<&[u8], String> {
    map_res(
        take_until("\0"),
        parse_from_str::<String>
    )(input)
}

fn parse_entry(input: &[u8]) -> IResult<&[u8], TreeEntry> {
    let (input, mode) = parse_mode(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, name) = parse_name(input)?;
    let (input, _) = tag("\0")(input)?;
    let (input, id) = parse_id(input)?;
    Ok((input, TreeEntry {
        id,
        name,
        mode
    }))
}

fn parse_tree<'a>(input: &'a[u8], id: &Id) -> IResult<&'a[u8], Tree> {
    let (input, entries) = many0(parse_entry)(input)?;
    Ok((input, Tree {
        id: id.to_owned(),
        entries
    }))
}

impl TryFrom<GitObject> for Tree {
    type Error = Error;
    fn try_from(mut obj: GitObject) -> Result<Self> {
        if obj.object_type() == ObjectType::TREE {
            let mut buf: Vec<u8> = Vec::new();
            obj.read_to_end(&mut buf)?;
            let tree: Tree = parse_tree(&buf, obj.id()).map(|res| res.1)
                .map_err(|_|ErrorKind::ParseError)?;
            Ok(tree)
        } else {
            Err(ErrorKind::InvalidObjectType.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::tree::{parse_tree, TreeEntry, FileMode, EntryType, Tree, parse_entry};
    use crate::model::id::Id;
    use std::str::FromStr;

    #[test]
    fn test_parse_tree() {
        let id = Id::from_str("916269d397a334666906f57d69b297decf25da41").expect("");
        let mut str = b"100644 README.md\0".to_vec();
        str.extend(id.bytes());
        let (_, entry) = parse_entry(&str).expect("parse failed.");
        assert_eq!(entry, TreeEntry {
            mode: FileMode::FILE,
            id,
            name: "README.md".to_string(),
        })
    }
}