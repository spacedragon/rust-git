use std::path::{Path, PathBuf};

use os_str_generic::OsStrGenericExt;
use std::collections::HashMap;
use std::fs::File;
use crate::errors::*;
use memmap::{MmapOptions};
use std::io::{Read, Cursor};

pub trait FileSystem {
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool;
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Box<dyn Iterator<Item= PathBuf>>;
    fn ls_files<P: AsRef<Path>>(&self, path: P) -> Box<dyn Iterator<Item= PathBuf>> {
        let path = path.as_ref().to_path_buf();
        let dir;
        let prefix ;
        if self.is_dir(&path) {
            prefix = None;
            dir = path.as_path();
        } else {
            prefix = path.file_name().map(|f|f.to_os_string());
            dir = path.parent().unwrap_or(Path::new("."));
        }
        if let Some(prefix) = prefix {
            let iter = self.read_dir(&dir)
                .filter( move|p| p.file_name().is_some() &&
                    p.file_name().unwrap().starts_with(&prefix));
            Box::new(iter)
        } else {
            self.read_dir(dir)
        }
    }
    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Box<dyn Read>>;
}

#[derive(Debug)]
pub struct OsFs;

impl FileSystem for OsFs {
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir()
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Box<dyn Iterator<Item=PathBuf>>{
        let path = path.as_ref();
        if self.is_dir(path) {
            if let Ok(read) = path.read_dir() {
                let read = read
                    .flat_map(|e| e.ok())
                    .map(|e| e.path());
                Box::new(read)
            } else {
                Box::new(std::iter::empty::<PathBuf>())
            }
        } else {
            Box::new(std::iter::empty::<PathBuf>())
        }
    }

    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Box<dyn Read>> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Box::new(Cursor::new(mmap)))
    }
}



#[derive(Debug, Default)]
pub struct MemFs (HashMap<PathBuf, Vec<u8>>);

impl MemFs {
    pub fn add_file<P: AsRef<Path>>(&mut self, file_name: P, content: Vec<u8>) {
        self.0.insert(file_name.as_ref().to_path_buf(), content);
    }
}

impl FileSystem for MemFs {
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        for (p, _) in &self.0 {
            let mut dir = p.parent();
            while let Some(p) = dir {
                if p == path {
                    return true
                }
                dir = p.parent();
            }
        }
        false
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Box<dyn Iterator<Item=PathBuf>> {
        let path = path.as_ref();
        if self.is_dir(path) {
            let keys: Vec<PathBuf> = self.0.keys()
                .map(|k| k.to_path_buf())
                .filter( |p| {
                     if let Some(parent) = p.parent() {
                         return parent == path;
                     }
                    false
                })
                .collect();
            Box::new(keys.into_iter())
        } else {
            Box::new(std::iter::empty::<PathBuf>())
        }
    }

    fn read_file<P: AsRef<Path>>(& self, path: P) -> Result<Box<dyn Read>> {
        let path = path.as_ref().to_path_buf();
        if let Some(content) = self.0.get(&path) {
            Ok(Box::new(Cursor::new(content.to_owned())))
        } else {
            Err(ErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound, format!("{:?} not found!", path)
            )).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::*;

    #[test]
    fn fs_can_ls_files() {
        let fs = OsFs {};
        assert_eq!(fs.ls_files("./Cargo").take(2).count(), 2);
    }

    #[test]
    fn fs_mem_fs() {
        let mut fs = MemFs::default();
        fs.add_file("/test/2/1.txt", vec![]);
        fs.add_file("/test/2/2.txt", vec![]);
        assert!(fs.is_dir("/test"));
        assert!(fs.is_dir("/test/2"));
        assert!(!fs.is_dir("/tes"));
        assert!(!fs.is_dir("/test/2/1"));

        assert_eq!(fs.ls_files("/test/2/").count(), 2);
        assert_eq!(fs.ls_files("/test/2/1").count(), 1);
    }
}