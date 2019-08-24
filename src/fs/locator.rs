



use std::path::PathBuf;
use crate::model::id::Id;




pub enum Locator {
    LooseObject(PathBuf, usize),
    Packfile(Id, usize),
    PackRef(Id, usize, Id),
    PackOfs(Id, usize, usize)
}


