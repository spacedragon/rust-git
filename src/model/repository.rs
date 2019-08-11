use std::path::{Path, PathBuf};
use flate2::bufread::ZlibDecoder;
use super::object::GitObject;
use super::id::Id;
use crate::fs::{FileSystem, OsFs, MemFs};

use crate::errors::*;
use std::str::FromStr;
use crate::model::object::parse_header;
use std::io::{BufReader, BufRead};

pub trait Repository {
    fn lookup(&self, id: &str) -> Option<GitObject>;
    fn get_object(&self, id: &Id) -> Option<GitObject>;
}

#[derive(Debug)]
pub struct FileRepository<FS: FileSystem> {
    path: PathBuf,
    git_dir: PathBuf,
    is_bare: bool,
    fs: FS,
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

    pub fn read_loose_object(&self, id: &Id) -> Result<GitObject> {
        let id_string = id.to_string();
        let (prefix, rest) = id_string.split_at(2);
        let path = self.git_dir
            .join("objects")
            .join(prefix)
            .join(rest);
        let file_reader= self.fs.read_file(path)?;
        let mut reader = BufReader::new(
            ZlibDecoder::new(BufReader::new(file_reader))
        );

        let mut vec = Vec::new();
        reader.read_until(0, &mut vec).chain_err(||"read file failed.")?;
        if let Ok((_, header)) = parse_header(&vec) {
            return Ok(GitObject {
                header,
                content: Box::new(reader)
            });
        } else {
            return Err(ErrorKind::ParseError.into());
        }
    }
}

 impl<FS: FileSystem> Repository for FileRepository<FS> {
    fn lookup(&self, id: &str) -> Option<GitObject> {
        if let Some(id) = self.lookup_loose_object_by_prefix(id) {
            return self.get_object(&id);
        }
        None
    }

    fn get_object(&self, id: &Id) -> Option<GitObject> {
        self.read_loose_object(id).ok()
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