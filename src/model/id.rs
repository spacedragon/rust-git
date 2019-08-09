

#[derive(Default, PartialEq, Eq, PartialOrd, Clone, Hash)]
pub struct Id {
    bytes: [u8; 20]
}

impl Id {
    pub fn new(buf: &[u8]) -> Id {
        let mut bytes = [0u8; 20];
        bytes.clone_from_slice(buf);
        Id { bytes }
    }
}