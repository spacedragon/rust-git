pub mod repository;
pub mod object;
pub mod id;





#[cfg(test)]
mod tests {
    use crate::fs::*;
    use crate::model::repository::FileRepository;
    use crate::model::id::Id;

    #[test]
    fn test_lookup_prefix() {
        let mut repo = FileRepository::<MemFs>::default();
        let id : Option<Id>= "1234567890123456789012345678901234567890".parse().ok();
        repo.add_file("objects/12/34567890123456789012345678901234567890", vec![]);
        assert_eq!(repo.lookup_loose_object_by_prefix("1234"), id)
    }
}