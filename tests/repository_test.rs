use rust_git::model::repository::{FileRepository, Repository};
use rust_git::model::id::Id;
use rust_git::model::object::ObjectType;


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
    assert_eq!(obj.header.object_type, ObjectType::BLOB);
    let mut c = String::new();
    let size = obj.content.read_to_string(&mut c).unwrap();
    assert_eq!(size, obj.header.length);
    assert!(c.contains("target/"))
}