
use std::collections::HashMap;
use crate::model::id::Id;
use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};
use nom::IResult;
use nom::bytes::complete::{take_until, tag, take_while, take_till};
use nom::multi::{separated_list};
use nom::sequence::{delimited};
use nom::character::{is_space};

use std::str;
use std::str::FromStr;
use crate::errors::*;
use nom::combinator::{map_res, rest};
use nom::branch::alt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use nom::character::complete::not_line_ending;

#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    committer: Option<Identity>,
    parent: Vec<Id>,
    author: Option<Identity>,
    message: String,
    tree: Id,
    other: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identity {
    name: String,
    email: String,
    date: DateTime<FixedOffset>,
}

impl Display for Identity {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} <{}> {}", self.name, self.email, self.date.format("%s %z"))
    }
}

enum Attr {
    Tree(Id),
    Parent(Id),
    Author(Identity),
    Committer(Identity),
    Unknown(String, String),
}





fn id_from_bytes(input: &[u8]) -> Result<Id> {
    let str = str::from_utf8(input)?;
    Id::from_str(str)
}

fn parse_tree(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("tree")(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, id) = map_res(not_line_ending, id_from_bytes)(input)?;
    Ok((input, Attr::Tree(id)))
}

fn parse_parent(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("parent")(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, id) = map_res(not_line_ending, id_from_bytes)(input)?;
    Ok((input, Attr::Parent(id)))
}

fn time_from_bytes(input: &[u8]) -> Result<DateTime<FixedOffset>> {
    let str = str::from_utf8(input)?;
    DateTime::parse_from_str(str, "%s %z").chain_err(|| ErrorKind::ParseError)
}

fn parse_datetime(input: &[u8]) -> IResult<&[u8], DateTime<FixedOffset>> {
    let (input, dt) = map_res(
        not_line_ending,
        time_from_bytes)(input)?;
    Ok((input, dt))
}

fn number_from_bytes<N: FromStr>(input: &[u8]) -> Result<N> {
    let str = str::from_utf8(input)?;
    N::from_str(str).map_err(|_| ErrorKind::ParseError.into())
}

fn parse_identity(input: &[u8]) -> IResult<&[u8], Identity> {
    let (input, name) = map_res(
        take_until(" <"), str::from_utf8)(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, email) = map_res(delimited(
        tag("<"),
        take_until(">"),
        tag(">"),
    ), str::from_utf8)(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, date) = parse_datetime(input)?;
    Ok((input, Identity { name: name.to_owned(), email: email.to_owned(), date }))
}

fn parse_author(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("author")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, identity) = parse_identity(input)?;
    Ok((input, Attr::Author(identity)))
}

fn parse_committer(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, _key) = tag("committer")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, identity) = parse_identity(input)?;
    Ok((input, Attr::Committer(identity)))
}

fn parse_attrs(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, attr) = alt((
        parse_committer,
        parse_author,
        parse_tree,
        parse_parent,
        parse_attr
    ))(input)?;
    Ok((input, attr))
}

fn parse_attr(input: &[u8]) -> IResult<&[u8], Attr> {
    let (input, key) = map_res(take_till(is_space), str::from_utf8)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, value) = map_res(not_line_ending, str::from_utf8)(input)?;
    Ok((input, Attr::Unknown(key.to_owned(), value.to_owned())))
}

fn parse_commit(input: &[u8]) -> IResult<&[u8], Commit> {
    let (input, attrs_str) = take_until("\n\n")(input)?;
    let (_, attrs) = separated_list(tag("\n"), parse_attrs)(attrs_str)?;
    let (input, message) = map_res(rest, str::from_utf8)(input)?;
    let mut commit = Commit {
        committer: None,
        parent: vec![],
        author: None,
        message: message.trim().to_owned(),
        tree: Id::default(),
        other: HashMap::new()
    };
    for attr in attrs {
        match attr {
            Attr::Author(identity) => commit.author = Some(identity),
            Attr::Committer(identity) => commit.committer = Some(identity),
            Attr::Tree(id) => commit.tree = id,
            Attr::Parent(id) => commit.parent.push(id),
            Attr::Unknown(k, v) => { commit.other.insert(k, v); }
        }
    }
    Ok((input, commit))
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_parse_identity() {
        let expected = Identity {
            email: "allendragon@gmail.com".to_string(),
            name: "space dragon".to_string(),
            date: FixedOffset::east(8 * 3600).timestamp(1_500_000_000, 0),
        };
        let str = b"space dragon <allendragon@gmail.com> 1500000000 +0800";
        let result = parse_identity(str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, expected);
    }

    #[test]
    fn test_parse_commit() {
        let message = b"Merge remote-tracking branch 'origin/master'

# Conflicts:
#       Cargo.lock
#       src/main.rs
#       src/model/mod.rs
#       src/model/repository.rs";
        let mut str = b"tree b2a72b0fe6f44f4839db41106ca11ad6db372327
parent 28a4a7af6a414d38e87b775bfeac430aeeb4985d
parent 1a8f251087b9d51b18f1a821cd87dae0acee2936
author space dragon <allendragon@gmail.com> 1500000000 +0800
committer space dragon <allendragon@gmail.com> 1500000000 +0800

".to_vec();
        str.extend_from_slice(message);
        let author = Identity {
            email: "allendragon@gmail.com".to_string(),
            name: "space dragon".to_string(),
            date: FixedOffset::east(8 * 3600).timestamp(1500000000, 0),
        };

        let expected = Commit {
            committer: Some(author.clone()),
            parent: vec![Id::from_str("28a4a7af6a414d38e87b775bfeac430aeeb4985d").unwrap(),
                         Id::from_str("1a8f251087b9d51b18f1a821cd87dae0acee2936").unwrap()],
            author: Some(author),
            message: str::from_utf8(message).unwrap().to_owned(),
            tree: Id::from_str("b2a72b0fe6f44f4839db41106ca11ad6db372327").unwrap(),
            other: HashMap::new()
        };
        let result = parse_commit(&str);
        assert_eq!(result.unwrap(), ("".as_bytes() ,expected));
    }
}