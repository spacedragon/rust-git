use crate::errors::*;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Clone, Hash)]
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

impl std::str::FromStr for Id {
    type Err = Error;

    fn from_str(target: &str) -> Result<Self> {
        let trimmed = target.trim();
        if trimmed.len() != 40 {
            return Err(ErrorKind::BadId.into())
        }
        let mut id = Id::default();
        for i in 0..20 {
            let slice = &trimmed[i*2..i*2+2];
            id.bytes[i] = u8::from_str_radix(slice, 16).chain_err(||ErrorKind::BadId)?;
        }
        Ok(id)
    }
}