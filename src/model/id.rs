use crate::errors::*;
use std::fmt;
use hex::*;

type IDBytes = [u8; 20];

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Clone, Hash)]
pub struct Id {
    bytes: IDBytes
}

impl Id {
    pub fn new(buf: &[u8]) -> Id {
        let mut bytes = [0u8; 20];
        bytes.clone_from_slice(buf);
        Id { bytes }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
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
        id.bytes = IDBytes::from_hex(trimmed).chain_err(|| ErrorKind::BadId)?;
        Ok(id)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.bytes.write_hex(f)?;
        Ok(())
    }
}