use crate::utils::get_permissions;
use chrono::{DateTime, Local};
use std::{
    fs::{DirEntry, FileType},
    path::PathBuf,
};

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
            FileKind::File => "f".to_string(),
            FileKind::Directory => "d".to_string(),
            FileKind::Symlink => "l".to_string(),
            FileKind::Other => "?".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetailedEntry {
    path: PathBuf,
    name: String,
    kind: FileKind,
    size: u64,
    permissions: String,
    timestamp: Option<DateTime<Local>>,
}

impl From<DirEntry> for DetailedEntry {
    fn from(entry: DirEntry) -> Self {
        let path = entry.path();
        let meta = path.metadata().ok();

        // Get the file name if it exists, otherwise use the full path.
        // (Likely that its a root directory or similar, so something very short)
        let name = path.file_name().map_or_else(
            || path.as_os_str().to_string_lossy().to_string(),
            |name| name.to_string_lossy().to_string(),
        );

        // Note: this cant fail because in the Walk iterator only entries
        // with valid file types are returned.
        let kind = entry.file_type().map_or(FileKind::Other, FileKind::from);

        let size = meta.as_ref().map_or(0, |m| m.len());

        let permissions = meta
            .as_ref()
            .map(|m| get_permissions(m))
            .unwrap_or_else(|| "unknown".to_string());

        let timestamp = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| Some(DateTime::from(t)));

        Self {
            path: path.strip_prefix("./").unwrap_or(&path).to_path_buf(),
            name,
            kind,
            size,
            permissions,
            timestamp,
        }
    }
}

impl DetailedEntry {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn permissions(&self) -> &str {
        self.permissions.as_str()
    }

    pub fn timestamp(&self) -> Option<DateTime<Local>> {
        self.timestamp
    }
}
