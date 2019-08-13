use super::object::ObjectType;
use super::id::Id;
use super::commit::Identity;

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    object_type: ObjectType,
    object: Id,
    tag: String,
    tagger: Identity,
    message: String
}
