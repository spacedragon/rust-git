use std::path::{Path, PathBuf};

use super::object::GitObject;
use super::id::Id;
use crate::fs::FileSystem;


trait Repository {
    fn lookup(id: &str) -> Option<GitObject>;
}

#[derive(Debug)]
pub struct FileRepository{
    path: PathBuf,
    git_dir: PathBuf,
    is_bare: bool,
    fs: FileSystem
}


impl Repository for FileRepository {
    fn lookup(id: &str) -> Option<GitObject> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct InvalidRepositoryError(PathBuf);

impl std::fmt::Display for InvalidRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not a valid repository" , self.0.to_string_lossy())
    }
}



impl FileRepository {
    pub fn is_bare(&self) -> bool {
        self.is_bare
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn git_dir(&self) -> &Path {
        self.git_dir.as_path()
    }

    fn check_git_dir<P: AsRef<Path>>(path: P) -> bool {
        if path.as_ref().is_dir() {
            true
        } else {
            false
        }
    }
    pub fn open<P: AsRef<Path>>(path: P) -> Result<FileRepository, InvalidRepositoryError> {
        let fs = FileSystem {};
        let repo_path = PathBuf::new().join(path);
        let git_dir = repo_path.join(".git");
        if fs.is_dir(&git_dir) {
            Ok(FileRepository {
                git_dir,
                is_bare: false,
                path: repo_path,
                fs,
            })
        } else if fs.is_dir(&repo_path) {
            Ok(FileRepository {
                git_dir: repo_path.clone(),
                is_bare: true,
                path: repo_path,
                fs,
            })
        } else {
            Err(InvalidRepositoryError(repo_path))
        }
    }
    pub fn lookup_loose_object_by_prefix(&self, idstr: &str) -> Option<Id> {
        if idstr.len() <= 2 {
            return None;
        }
        let (prefix, rest) = &idstr.split_at(2);
        let objects_dir = self.git_dir.join("objects").join(prefix);
        if objects_dir.is_dir() {
            let mut ret = None;
            if let Ok(entries) = objects_dir.read_dir() {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let mut filename = format!("{:?}", entry.file_name());
                        if filename.starts_with(rest) {
                            if ret == None {
                                filename.insert_str(0, prefix);
                                ret = Some(Id::new(filename.as_bytes()))
                            } else {
                                return None;
                            }
                        }
                    }
                }
                return ret;
            } else {
                None
            }
        } else {
            None
        }
    }
}