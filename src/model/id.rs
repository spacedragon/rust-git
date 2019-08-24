use crate::errors::*;
use std::fmt;
use hex::*;
use std::cmp::{Ordering};
use nom::AsBytes;

type IDBytes = [u8; 20];

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Hash)]
pub enum Id {
    Full(IDBytes),
    Partial(Vec<u8>)
}

impl Id {
    pub fn new(buf: &[u8]) -> Id {
        if buf.len() == 20 {
            let mut bytes = [0u8; 20];
            bytes.clone_from_slice(buf);
            Id::Full(bytes)
        } else {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&buf);
            Id::Partial(bytes)
        }
    }
    pub fn bytes(&self) -> &[u8] {
        match self {
            Id::Full(bytes) => bytes,
            Id::Partial(bytes) => bytes
        }
    }

    pub fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        match self {
            Id::Full(bytes) => {
                match other {
                    Id::Full(other_bytes) => bytes.partial_cmp(other_bytes),
                    Id::Partial(other_bytes) =>
                        bytes.as_bytes().partial_cmp(other_bytes.as_slice())
                }
            },
            Id::Partial(bytes) => {
                let other_bytes = &other.bytes()[0..bytes.len()];
                bytes.as_slice().partial_cmp(other_bytes)
            }
        }
    }
}

impl std::str::FromStr for Id {
    type Err = Error;

    fn from_str(target: &str) -> Result<Self> {
        let trimmed = target.trim();
        if trimmed.len() > 40 {
            return Err(ErrorKind::BadId.into())
        }
        match trimmed.len() {
            40 => {
              let bytes =  IDBytes::from_hex(trimmed).chain_err(|| ErrorKind::BadId)?;
              Ok(Id::Full(bytes))
            },
            len if len < 40 =>  {
                let bytes = Vec::from_hex(trimmed).chain_err(|| ErrorKind::BadId)?;
                Ok(Id::Partial(bytes))
            },
            _ => Err(ErrorKind::BadId.into())
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.bytes().write_hex(f)?;
        Ok(())
    }
}

impl Default for Id {
    fn default() -> Self { Id::Full([0u8;20]) }
}

