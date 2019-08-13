use rust_git::model::repository::{FileRepository, Repository};
use rust_git::model::id::Id;
use rust_git::model::object::{ObjectType};
use std::io::Read;
use rust_git::model::commit::Commit;
use std::str::FromStr;
use std::str;
use rust_git::model::blob::Blob;
use std::convert::TryInto;
use rust_git::model::tree::*;
use rust_git::model::tag::Tag;


#[test]
fn test_open_repo() {
    let repo = FileRepository::open(".").unwrap();
    assert!(!repo.is_bare());
}


#[test]
fn test_lookup_prefix() {
    let repo = FileRepository::open(".").unwrap();
    let id_opt = repo.lookup_loose_object_by_prefix("00b05df6");
    let expected: Option<Id> = "00b05df6a04840cd719b750ed53db08c8a1a4624".parse().ok();
    assert_eq!(id_opt, expected);
}

#[test]
fn test_lookup_object() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("a221ac1");
    assert!(obj.is_some());
    let mut obj = obj.unwrap();
    assert_eq!(obj.header().object_type, ObjectType::BLOB);
    let mut c = String::new();
    let size = obj.read_to_string(&mut c).unwrap();
    assert_eq!(size, obj.header().length);
    assert!(c.contains("target/"))
}

#[test]
fn test_lookup_commit() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("d586c1ae7");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let commit: Commit = obj.try_into().expect("parse commit failed.");
    assert_eq!(commit.tree(), &Id::from_str("e4f31b37d03f304f38d6f1d6c545848c8d70194c").unwrap());
    assert_eq!(commit.message(), "parse commit");
}

#[test]
fn test_lookup_blob() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("a221ac1");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let blob: Blob = obj.try_into().expect("parse commit failed.");
    assert!(str::from_utf8(blob.content()).unwrap().contains("target/"));
}

#[test]
fn test_lookup_tree() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("e4f31b37d0");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let tree: Tree = obj.try_into().expect("parse tree failed.");
    let file = tree.entries().iter().find(|e| e.name() == "Cargo.toml");
    assert!(file.is_some());
    assert_eq!(file.unwrap().mode() , FileMode::FILE);
}
#[test]
fn test_lookup_tag() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("a8903f510");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let tag: Tag = obj.try_into().expect("parse tag failed.");
    assert_eq!(tag.object(), &Id::from_str("a541069eb298c4969982721adea07e526d899351").unwrap());
    assert_eq!(tag.object_type(), ObjectType::COMMIT);
    assert_eq!(tag.tag(), "v0.1");
    assert_eq!(tag.message(), "a tag");
}

