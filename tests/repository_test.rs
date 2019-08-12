use rust_git::model::repository::{FileRepository, Repository};
use rust_git::model::id::Id;
use rust_git::model::object::{ObjectType, AsObject};
use std::io::Read;
use rust_git::model::commit::Commit;
use std::str::FromStr;
use std::str;
use rust_git::model::blob::Blob;
use std::convert::TryInto;


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
    let mut obj = obj.unwrap();
    let commit: Commit = obj.try_into().expect("parse commit faild.");
    assert_eq!(commit.tree(), &Id::from_str("e4f31b37d03f304f38d6f1d6c545848c8d70194c").unwrap());
    assert_eq!(commit.message(), "parse commit");
}

#[test]
fn test_lookup_blob() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("a221ac1");
    assert!(obj.is_some());
    let mut obj = obj.unwrap();
    let blob: Blob = obj.try_into().expect("parse commit faild.");
    assert!(str::from_utf8(blob.content()).unwrap().contains("target/"));


}