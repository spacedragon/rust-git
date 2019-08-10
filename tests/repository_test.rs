use rust_git::model::repository::FileRepository;
use rust_git::model::id::Id;


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