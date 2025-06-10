use std::{
    fs::{DirEntry, FileType},
    path::PathBuf,
};

use getset::{CopyGetters, Getters};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl From<FileType> for FileKind {
    fn from(ft: FileType) -> Self {
        if ft.is_dir() {
            FileKind::Directory
        } else if ft.is_file() {
            FileKind::File
        } else if ft.is_symlink() {
            FileKind::Symlink
        } else {
            FileKind::Other
        }
    }
}

impl ToString for FileKind {
    fn to_string(&self) -> String {
        match self {
            FileKind::File => "file".to_string(),
            FileKind::Directory => "dir".to_string(),
            FileKind::Symlink => "symlink".to_string(),
            FileKind::Other => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct DetailedEntry {
    #[getset(get = "pub")]
    path: PathBuf,

    #[getset(get_copy = "pub")]
    kind: FileKind,
}

impl From<DirEntry> for DetailedEntry {
    fn from(entry: DirEntry) -> Self {
        let path = entry.path();

        // Note: this cant fail because in the Walk iterator only entries
        // with valid file types are returned.
        let kind = entry.file_type().map_or(FileKind::Other, FileKind::from);

        Self { path, kind }
    }
}
