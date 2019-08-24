use crate::model::id::Id;
use crate::model::object::*;

use crate::errors::*;

use crate::model::repository::Repository;

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

    pub fn from(repo: &dyn Repository, obj: &GitObject) -> Result<Self> {
        if obj.header().object_type == ObjectType::BLOB {
            let content = repo.read_content(&obj)?;
            Ok(Blob {
                id: obj.id().to_owned(),
                content
            })
        } else {
            Err(ErrorKind::InvalidObjectType.into())
        }
    }
}

