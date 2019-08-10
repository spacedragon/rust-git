use std::path::{Path, PathBuf};

use super::object::GitObject;
use super::id::Id;
use crate::fs::{FileSystem, OsFs, MemFs};

use crate::errors::*;
use std::str::FromStr;

trait Repository {
    fn lookup(id: &str) -> Option<GitObject>;
}

#[derive(Debug)]
pub struct FileRepository<FS: FileSystem> {
    path: PathBuf,
    git_dir: PathBuf,
    is_bare: bool,
    fs: FS,
}


impl Repository for FileRepository<OsFs> {
    fn lookup(_id: &str) -> Option<GitObject> {
        unimplemented!()
    }
}

impl<FS: FileSystem> FileRepository<FS> {
    pub fn lookup_loose_object_by_prefix(&self, idstr: &str) -> Option<Id> {
        if idstr.len() <= 2 {
            return None;
        }
        let (prefix, rest) = idstr.split_at(2);
        let objects_dir = self.git_dir.join("objects").join(prefix);
        let files: Vec<PathBuf> = self.fs.ls_files(objects_dir.join(rest))
            .take(2).collect();
        if files.len() == 1 {
            let file = files.first().unwrap();
            if let Some(file) =  file.file_name() {
                let mut hex = String::new();
                hex.push_str(prefix);
                hex.push_str(file.to_string_lossy().as_ref());
                Id::from_str(&hex).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl FileRepository<MemFs> {
    pub fn default() -> Self {
        FileRepository::<MemFs> {
            git_dir: Path::new("").to_path_buf(),
            is_bare: true,
            path: Path::new("").to_path_buf(),
            fs: MemFs::default(),
        }
    }
    pub fn add_file<P: AsRef<Path>>(&mut self, file_name: P, content: Vec<u8>) {
        self.fs.add_file(file_name, content);
    }
}


impl FileRepository<OsFs> {
    pub fn is_bare(&self) -> bool {
        self.is_bare
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn git_dir(&self) -> &Path {
        self.git_dir.as_path()
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fs = OsFs {};
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
            Err(ErrorKind::InvalidRepository(repo_path).into())
        }
    }
}