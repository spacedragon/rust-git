use nom::{IResult, combinator::map};
use nom::branch::alt;
use nom::bytes::complete::{take_until, tag};
use std::str::FromStr;
use nom::combinator::map_res;
use std::str;
use crate::errors::*;
use std::io::{BufRead, Read};
use std::fmt::{Display, Formatter};
use crate::model::id::Id;

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    BLOB,
    COMMIT,
    TREE,
    TAG
}

fn usize_from_str_bytes(input: &[u8]) -> Result<usize> {
    let str = str::from_utf8(input)?;
    usize::from_str(str).chain_err(|| "bad length str")
}

fn parse_length(input: &[u8]) -> IResult<&[u8], usize> {
    map_res(
        take_until("\0"),
        usize_from_str_bytes
    )(input)
}

fn parse_type(input: &[u8]) -> IResult<&[u8] , ObjectType> {
    let types = alt((
                    map(tag("blob"), |_| ObjectType::BLOB),
                     map(tag("commit"), |_| ObjectType::COMMIT),
                     map(tag("tree"), |_| ObjectType::TREE),
                     map(tag("tag"), |_| ObjectType::TAG)));
    types(input)
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], ObjectHeader> {
    let (input, object_type) = parse_type(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, length) = parse_length(input)?;
    let (input, _) = tag("\0")(input)?;
    Ok((input, ObjectHeader { object_type, length }))
}



#[derive(Debug, PartialEq, Clone)]
pub struct ObjectHeader {
    pub object_type: ObjectType,
    pub length: usize
}

impl From<ObjectHeader> for Vec<u8> {
    fn from(o: ObjectHeader) -> Self {
        let mut ret: Vec<u8> = vec![];
        match o.object_type {
            ObjectType::TAG => ret.extend_from_slice(b"tag"),
            ObjectType::BLOB => ret.extend_from_slice(b"blob"),
            ObjectType::COMMIT => ret.extend_from_slice(b"commit"),
            ObjectType::TREE => ret.extend_from_slice(b"tree")
        }
        ret.push(b' ');
        ret.extend(o.length.to_string().as_bytes());
        ret.push(0u8);
        ret
    }
}

pub struct GitObject {
    id: Id,
    header: ObjectHeader,
    content: Box<dyn BufRead>
}

impl GitObject {
    pub fn new(id: &Id, header: ObjectHeader, content: Box<dyn BufRead>) -> Self {
        GitObject {
            id: id.to_owned(),
            header,
            content
        }
    }
    pub fn header(&self) -> &ObjectHeader {
        &self.header
    }

    pub fn object_type(&self) -> ObjectType {
        self.header.object_type.clone()
    }

    pub fn id(&self) -> &Id{
        &self.id
    }
}

impl Read for GitObject {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.content.read(buf)
    }
}

pub trait AsObject<T> {
    fn as_object(&mut self) -> Result<T>;
}


impl Display for GitObject {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.header.object_type, self.id)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let header = "commit 20\0";
        let expected = ObjectHeader { object_type: ObjectType::COMMIT , length: 20 };
        let parsed = parse_header(header.as_bytes());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), ("".as_bytes(), expected.clone()));
        let expected_bytes: Vec<u8> = expected.into();
        assert_eq!(expected_bytes, header.as_bytes());
    }
}