use rust_git::model::repository::FileRepository;
use std::fs;

#[test]
fn test_open_repo() {
    let repo = FileRepository::open(".").unwrap();
    assert!(!repo.is_bare());
}