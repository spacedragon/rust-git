use rust_git::model::repository::{FileRepository, Repository};
use rust_git::model::id::Id;
use rust_git::model::object::{ObjectType};

use rust_git::model::commit::Commit;
use std::str::FromStr;
use std::str;
use rust_git::model::blob::Blob;
use std::convert::TryInto;
use rust_git::model::tree::*;
use rust_git::model::tag::Tag;
use std::path::Path;
use rust_git::fs::{OsFs, FileSystem};
use rust_git::fs::pack_idx::PackIdx;
use rust_git::fs::pack_file::PackFile;



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
    let obj = obj.unwrap();
    assert_eq!(obj.header().object_type, ObjectType::BLOB);
    let buf = repo.read_content(&obj).unwrap();
    assert_eq!(buf.len(), obj.header().length);
    let c = String::from_utf8(buf).unwrap();
    assert!(c.contains("target/"))
}

#[test]
fn test_lookup_commit() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("d586c1ae7");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let commit: Commit = Commit::from(&repo, &obj).unwrap();
    assert_eq!(commit.tree(), &Id::from_str("e4f31b37d03f304f38d6f1d6c545848c8d70194c").unwrap());
    assert_eq!(commit.message(), "parse commit");
}

#[test]
fn test_lookup_blob() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("a221ac1");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let blob: Blob = Blob::from(&repo, &obj).expect("parse commit failed.");
    let content = str::from_utf8(blob.content()).unwrap();
    assert_eq!(blob.content().len(), obj.size());
    assert!(content.contains("target/"));
    println!("{}", content)
}

#[test]
fn test_lookup_tree() {
    let repo = FileRepository::open(".").unwrap();
    let obj = repo.lookup("e4f31b37d0");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    let tree: Tree = Tree::from(&repo, &obj).expect("parse tree failed.");
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
    let tag: Tag = Tag::from(&repo, &obj).expect("parse tag failed.");
    assert_eq!(tag.object(), &Id::from_str("a541069eb298c4969982721adea07e526d899351").unwrap());
    assert_eq!(tag.object_type(), ObjectType::COMMIT);
    assert_eq!(tag.tag(), "v0.1");
    assert_eq!(tag.message(), "a tag");
}

#[test]
fn test_parse_idx() {
    let dir = Path::new(".")
        .join("tests").join("fixture").join("objects").join("pack");
    let idx = dir.join("pack-1dba36995240d4e37eb9c1aae367accc94169fc4.idx");
    let os = OsFs::new("./tests");
    let reader = os.read_file(idx).expect("read file failed.");
    let idx: PackIdx = reader.try_into().expect("parse idx failed");
    assert_eq!(idx.version(), 2);
    let id = Id::from_str("a6952adde41289267215c9cdd0487df025214952").expect("");
    let offset = idx.lookup(&id).unwrap().1;
    assert_eq!(offset, 12);

    let id = Id::from_str("a9d37c56").expect("");
    let offset = idx.lookup(&id).unwrap().1;
    assert_eq!(offset, 8474);

    let id = Id::from_str("aad37c56").expect("");
    assert_eq!(idx.lookup(&id), None);
 }

#[test]
fn test_parse_pack() {
    let dir = Path::new(".")
        .join("tests").join("fixture").join("objects").join("pack");
    let pack = dir.join("pack-1dba36995240d4e37eb9c1aae367accc94169fc4.pack");
    let os = OsFs::new("./tests");
    let mmap = os.map_file(pack).expect("read file failed");
    let pack = PackFile::try_from(mmap).expect("parse pack failed");
    assert_eq!(pack.version(), 2);
    assert_eq!(pack.count(), 147);
    assert_eq!(pack.id().to_string(), "1dba36995240d4e37eb9c1aae367accc94169fc4");
}

#[test]
fn test_load_object_from_pack() {
    let repo = FileRepository::open("./tests/fixture").expect("open repo failed");
    let obj = repo.lookup("a6952add");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    assert_eq!(obj.object_type(), ObjectType::COMMIT);
    let commit = Commit::from(&repo, &obj).expect("parse tree failed");
    assert_eq!(commit.tree(), &Id::from_str("a31f42a223bbd8415781fcb4ad2c235778730e45").unwrap());
}

#[test]
fn test_load_deltified_object_from_pack() {
    let repo = FileRepository::open("./tests/fixture").expect("open repo failed");
    let obj = repo.lookup("86930390cd94497678a0ee06fa09bdf838e794f5");
    assert!(obj.is_some());
    let obj = obj.unwrap();
    assert_eq!(obj.object_type(), ObjectType::BLOB);
    let blob = Blob::from(&repo, &obj).expect("parse blob failed");
    let content = str::from_utf8(blob.content()).expect("parse content failed");
    assert!(content.len() > 0);
}