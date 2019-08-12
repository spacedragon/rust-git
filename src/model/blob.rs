use crate::model::id::Id;
use crate::model::object::*;
use std::io::Read;
use crate::errors::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub struct Blob {
    id: Id,
    content: Vec<u8>
}

impl Blob {
    pub fn id(&self) -> &Id {
        &self.id
    }
    pub fn content(&self) -> &[u8] {
        self.content.as_slice()
    }
}

impl TryFrom<GitObject> for Blob {
    type Error = Error;
    fn try_from(mut obj: GitObject) -> Result<Self> {
        if obj.header().object_type == ObjectType::BLOB {
            let mut content: Vec<u8> = Vec::new();
            obj.read_to_end(&mut content)?;
            Ok(Blob {
                id: obj.id().to_owned(),
                content
            })
        } else {
            Err(ErrorKind::InvalidObjectType.into())
        }
    }
}