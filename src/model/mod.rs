pub mod repository;
pub mod object;
pub mod id;
pub mod commit;
pub mod blob;
pub mod tree;

#[cfg(test)]
mod tests {
    use crate::fs::*;
    use crate::model::repository::*;
    use crate::model::id::Id;
    use flate2::write::ZlibEncoder;
    use crate::model::object::*;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_lookup_prefix() {
        let mut repo = FileRepository::<MemFs>::default();
        let id : Option<Id>= "1234567890123456789012345678901234567890".parse().ok();
        repo.add_file("objects/12/34567890123456789012345678901234567890", vec![]);
        assert_eq!(repo.lookup_loose_object_by_prefix("1234"), id)
    }

    #[test]
    fn test_lookup_object() {
        let mut repo = FileRepository::<MemFs>::default();
        let content = b"hello blob.";
        let header = ObjectHeader {
            object_type: ObjectType::BLOB,
            length: content.len()
        };
        let mut bytes: Vec<u8> = header.clone().into();
        bytes.extend_from_slice(content);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes).unwrap();
        repo.add_file("objects/12/34567890123456789012345678901234567890", encoder.finish().unwrap());
        let obj = repo.lookup("1234");
        assert!(obj.is_some());
        assert_eq!(obj.unwrap().header(), &header);
    }
}
