use std::fs::{DirEntry, FileType};

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

#[derive(Debug, Clone)]
pub struct DetailedEntry {
    name: String,
    kind: FileKind,
}

impl DetailedEntry {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }
}

impl From<DirEntry> for DetailedEntry {
    fn from(entry: DirEntry) -> Self {
        let path = entry.path();

        // Get the file name if it exists, otherwise use the full path.
        // (Likely that its a root directory or similar, so something very short)
        let name = path.file_name().map_or_else(
            || path.as_os_str().to_string_lossy().to_string(),
            |name| name.to_string_lossy().to_string(),
        );

        // Note: this cant fail because in the Walk iterator only entries
        // with valid file types are returned.
        let kind = entry.file_type().map_or(FileKind::Other, FileKind::from);

        Self { name, kind }
    }
}
