use std::path::{Path, PathBuf};
use std::fs::{DirEntry, ReadDir};
use os_str_generic::OsStrGenericExt;
use std::sync::Arc;

#[derive(Debug)]
pub struct FileSystem;


pub enum LsFiles {
    Empty,
    Dir(ReadDir, PathBuf),
}

impl Iterator for LsFiles {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LsFiles::Empty => None,
            LsFiles::Dir(readDir, prefix) => {
                while let Some(e) = readDir.next() {
                    if let Ok(e) = e {
                        if e.file_name().starts_with(prefix.file_name().unwrap()) {
                            return Some(e);
                        }
                    }
                }
                return None;
            }
        }
    }
}

impl FileSystem {
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir()
    }

    pub fn ls_files<P : AsRef<Path>>(&self, path: P) -> LsFiles {
        let p = path.as_ref();
        let dir = p.parent().unwrap_or(Path::new("."));

        if dir.is_dir() {
            if let Ok(readDir) = dir.read_dir() {
                LsFiles::Dir(readDir, p.to_path_buf())
            } else {
                LsFiles::Empty
            }
        } else {
            LsFiles::Empty
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::FileSystem;

    #[test]
    fn fs_can_ls_files() {
        let fs = FileSystem {};
        assert_eq!(fs.ls_files("./Cargo").take(2).count(), 2);
    }
}