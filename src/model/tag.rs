use super::object::ObjectType;
use super::id::Id;
use super::commit::Identity;
use nom::IResult;
use nom::bytes::complete::{tag, take_while, take_until, take_till};
use nom::combinator::{map_res, rest};
use nom::character::complete::{not_line_ending, line_ending};
use nom::character::is_space;
use crate::model::commit::*;
use crate::model::object::{parse_object_type, GitObject};
use nom::branch::alt;
use nom::multi::separated_list;
use std::collections::HashMap;
use std::io::Read;
use std::convert::TryFrom;
use crate::errors::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    id : Id,
    object_type: ObjectType,
    object: Id,
    tag: String,
    tagger: Option<Identity>,
    message: String,
    other: HashMap<String, String>,
}

impl Tag {
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn tag(&self) -> &str {
        self.tag.as_str()
    }
    pub fn tagger(&self) -> &Option<Identity> {
        &self.tagger
    }
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
    pub fn object(&self) -> &Id {
        &self.object
    }
    pub fn object_type(&self) -> ObjectType {
        self.object_type.clone()
    }
}

pub enum Attr {
    Object(Id),
    Tagger(Identity),
    Tag(String),
    Type(ObjectType),
    Unknown(String, String),
}

fn parse_object(input :&[u8]) -> IResult<&[u8] , Attr> {
    let (input, _key) = tag("object")(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, id) = map_res(not_line_ending, id_from_str_bytes)(input)?;
    Ok((input, Attr::Object(id)))
}

fn parse_tagger(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("tagger")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, identity) = parse_identity(input)?;
    Ok((input, Attr::Tagger(identity)))
}

fn parse_tag(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("tag")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, tag) = map_res(not_line_ending, std::str::from_utf8)(input)?;
    Ok((input, Attr::Tag(tag.to_string())))
}
fn parse_type(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("type")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, object_type) = parse_object_type(input)?;
    Ok((input, Attr::Type(object_type)))
}

fn parse_attr(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, key) = map_res(take_till(is_space), std::str::from_utf8)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, value) = map_res(not_line_ending, std::str::from_utf8)(input)?;
    Ok((input, Attr::Unknown(key.to_owned(), value.to_owned())))
}

fn parse_attrs(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, attr) = alt((
        parse_object,
        parse_type,
        parse_tag,
        parse_tagger,
        parse_attr
    ))(input)?;
    Ok((input, attr))
}

fn parse_tag_object<'a>(input :&'a[u8], id: &Id) -> IResult<&'a[u8] , Tag> {
    let (input, attrs_str) = take_until("\n\n")(input)?;
    let (_, attrs) = separated_list(line_ending, parse_attrs)(attrs_str)?;
    let (input, message) = map_res(rest, std::str::from_utf8)(input)?;
    let mut tag = Tag {
        object_type: ObjectType::BLOB,
        id: id.to_owned(),
        object: Id::default(),
        tag: "".to_string(),
        message: message.trim().to_owned(),
        tagger: None,
        other: HashMap::new()
    };
    for attr in attrs {
        match attr {
            Attr::Tag(t) => tag.tag = t,
            Attr::Tagger(identity) => tag.tagger = Some(identity),
            Attr::Object(id) => tag.object = id,
            Attr::Type(t) => tag.object_type = t,
            Attr::Unknown(k, v) => { tag.other.insert(k, v); }
        }
    }
    Ok((input, tag))
}

impl TryFrom<GitObject> for Tag {
    type Error = Error;
    fn try_from(mut obj: GitObject) -> Result<Self> {
        if obj.object_type() == ObjectType::TAG {
            let mut buf: Vec<u8> = Vec::new();
            obj.read_to_end(&mut buf)?;
            let tag: Tag = parse_tag_object(&buf, obj.id()).map(|res| res.1)
                .map_err(|_|ErrorKind::ParseError)?;
            Ok(tag)
        } else {
            Err(ErrorKind::InvalidObjectType.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn test_parse_tag() {
        let str = b"object a541069eb298c4969982721adea07e526d899351
type commit
tag v0.1
tagger spacedragon <allendragon@gmail.com> 1565707955 +0800

a tag";
            let id = Id::default();
            let (_, tag) = parse_tag_object(str, &id).expect("parse failed");
        assert_eq!(tag.object, Id::from_str("a541069eb298c4969982721adea07e526d899351").unwrap());
        assert_eq!(tag.object_type, ObjectType::COMMIT);
        assert_eq!(tag.tagger, Some(Identity {
            name: "spacedragon".to_string(),
            email: "allendragon@gmail.com".to_string(),
            date: FixedOffset::east(8 * 3600).timestamp(1565707955, 0), }));
        assert_eq!(tag.tag, "v0.1".to_string());
        assert_eq!(tag.message, "a tag".to_string());

    }
}

