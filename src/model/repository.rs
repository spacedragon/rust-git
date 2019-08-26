use std::path::{Path, PathBuf};
use flate2::bufread::ZlibDecoder;
use super::object::GitObject;
use super::id::Id;
use crate::fs::{FileSystem, OsFs, MemFs};

use crate::errors::*;
use std::str::FromStr;
use crate::model::object::{parse_header};
use std::io::{BufReader, BufRead, Write};
use crate::fs::pack_file::PackFile;
use crate::fs::locator::Locator;
use crate::fs::pack_idx::PackIdx;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use crate::fs::content_reader::ContentReader;

pub trait Repository {
    fn lookup(&self, id: &str) -> Option<GitObject>;
    fn get_object(&self, id: &Id) -> Option<GitObject>;
    fn read_content(&self, git_object: &GitObject) -> Result<(Vec<u8>)>;
    fn write_content(&self, git_object: &GitObject, writer: &mut dyn Write) -> Result<u64>;
}

pub struct FileRepository<FS: FileSystem> {
    path: PathBuf,
    git_dir: PathBuf,
    is_bare: bool,
    pub(crate) fs: FS,
    pub(crate) packfiles: HashMap<Id, PackFile>,
}

impl<FS: FileSystem> Display for FileRepository<FS> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Repository({:?}) ", self.git_dir)
    }
}

impl<FS: FileSystem> FileRepository<FS> {
    pub fn lookup_packfile_by_prefix(&self, idstr: &str) -> Option<Id> {
        if idstr.len() <= 2 {
            return None;
        }
        if let Ok(id) = Id::from_str(idstr) {
            return self.packfiles.values()
                .find_map(|p| {
                    if let Some(idx) = p.idx() {
                        idx.lookup(&id).map(|r| r.0)
                    } else {
                        None
                    }
                });
        }
        None
    }
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
            if let Some(file_name) = file.file_name() {
                let mut hex = String::new();
                hex.push_str(prefix);
                hex.push_str(file_name.to_string_lossy().as_ref());
                Id::from_str(&hex).ok()
            } else {
                None
            }
        } else {
            None
        }
    }


    pub fn read_from_packfile(&self, id: &Id) -> Option<GitObject> {
        self.packfiles.values().find_map(|p| p.find_object(&id))
    }

    pub fn read_loose_object(&self, id: &Id) -> Result<GitObject> {
        if let Id::Partial(_) = id {
            return Err(ErrorKind::BadId.into());
        }
        let id_string = id.to_string();
        let (prefix, rest) = id_string.split_at(2);
        let path = self.git_dir
            .join("objects")
            .join(prefix)
            .join(rest);
        let file_reader = self.fs.read_file(&path)?;
        let mut reader = BufReader::new(
            ZlibDecoder::new(BufReader::new(file_reader))
        );

        let mut vec = Vec::new();
        let offset = reader.read_until(0, &mut vec).chain_err(|| "read file failed.")?;
        if let Ok((_, header)) = parse_header(&vec) {
            Ok(GitObject::new(id, header, Locator::LooseObject(path, offset)))
        } else {
            Err(ErrorKind::ParseError.into())
        }
    }

    pub fn read_content_by_id(&self, id: &Id) -> Result<ContentReader> {
        if let Some(obj) = self.get_object(id) {
            self.read_content(&obj.locator, obj.size())
        } else {
            Err(ErrorKind::NotBelongThisRepo.into())
        }
    }

    fn read_content(&self, locator: &Locator, size: usize) -> Result<ContentReader> {
        match locator {
            Locator::PackOfs(pack_id, offset, base_offset) => {
                if let Some(pack) = self.packfiles.get(pack_id) {
                    let (base_locator, _, base_len) =
                        pack.read_object(*base_offset)?;
                    let base = self.read_content(&base_locator, base_len)?;
                    let obj = pack.read_object_content(*offset, size)?;
                    let obj = obj.attach_base(base, size)?;
                    Ok(obj)
                } else {
                    Err(ErrorKind::NotBelongThisRepo.into())
                }
            }
            Locator::PackRef(pack_id, offset, ref_id) => {
                if let Some(pack) = self.packfiles.get(pack_id) {
                    let base =
                        self.read_content_by_id(ref_id)?;
                    let obj = pack.read_object_content(*offset, size)?;
                    let obj = obj.attach_base(base, size)?;
                    Ok(obj)
                } else {
                    Err(ErrorKind::NotBelongThisRepo.into())
                }
            }
            Locator::Packfile(pack_id, offset) => {
                if let Some(pack) = self.packfiles.get(pack_id) {
                    let obj = pack.read_object_content(*offset, size)?;
                    Ok(obj)
                } else {
                    Err(ErrorKind::NotBelongThisRepo.into())
                }
            }
            Locator::LooseObject(path, offset) => {
                let file_reader = self.fs.read_file(&path)?;
                Ok(ContentReader::from_loose_file(file_reader, *offset, size))
            }
        }
    }
}

impl<FS: FileSystem> Repository for FileRepository<FS> {
    fn lookup(&self, id: &str) -> Option<GitObject> {
        if let Some(id) = self.lookup_loose_object_by_prefix(id) {
            self.read_loose_object(&id).ok()
        } else {
            Id::from_str(id)
                .ok()
                .and_then(|id| self.read_from_packfile(&id))
        }
    }

    fn get_object(&self, id: &Id) -> Option<GitObject> {
        self.read_loose_object(id).ok().or_else(|| self.read_from_packfile(&id))
    }

    fn read_content(&self, git_object: &GitObject) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(git_object.size());
        self.write_content(git_object, &mut buf)?;
        Ok(buf)
    }

    fn write_content(&self, git_object: &GitObject, writer: &mut dyn Write) -> Result<u64> {
        let mut reader = self.read_content(&git_object.locator, git_object.size())?;
        let size = std::io::copy(&mut reader, writer)?;
        Ok(size)
    }
}

impl FileRepository<MemFs> {
    pub fn default() -> Self {
        FileRepository::<MemFs> {
            git_dir: Path::new("").to_path_buf(),
            is_bare: true,
            path: Path::new("").to_path_buf(),
            fs: MemFs::default(),
            packfiles: HashMap::new(),
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

    pub fn scan_packs(&mut self) -> Result<()> {
        self.packfiles.clear();
        let dir = self.git_dir.join("objects").join("pack");
        let readir = dir.read_dir()?;
        for entry in readir {
            if let Ok(e) = entry {
                if let Some(ext) = e.path().extension() {
                    if ext == "pack" {
                        let pack = self.load_packfile(&e.path())?;
                        self.packfiles.insert(pack.id(), pack);
                    }
                }
            }
        }
        Ok(())
    }

    fn load_packfile(&mut self, path: &Path) -> Result<PackFile> {
        let map_file = self.fs.map_file(path)?;
        let mut packfile = PackFile::try_from(map_file)?;
        let idx_path = path.with_extension("idx");
        if idx_path.is_file() {
            let idx_file = self.fs.read_file(idx_path)?;
            let packidx: PackIdx = idx_file.try_into()?;
            packfile.load_idx(packidx);
        }
        Ok(packfile)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo_path = PathBuf::new().join(path);
        let git_dir = repo_path.join(".git");
        let fs = OsFs {
            prefix: git_dir.clone()
        };
        let mut repo = if fs.is_dir(&git_dir) {
            FileRepository {
                git_dir,
                is_bare: false,
                path: repo_path,
                fs,
                packfiles: HashMap::default(),
            }
        } else if fs.is_dir(&repo_path) {
            FileRepository {
                git_dir: repo_path.clone(),
                is_bare: true,
                path: repo_path,
                fs,
                packfiles: HashMap::default(),
            }
        } else {
            return Err(ErrorKind::InvalidRepository(repo_path).into());
        };
        repo.scan_packs()?;
        Ok(repo)
    }
}