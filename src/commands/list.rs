use crate::args::ListArgs;
use crate::config::{ListConfig, ListVariable};
use crate::err::PlsError;
use crate::table::Table;
use crate::util;
use crate::walk::DirWalk;
use chrono::{DateTime, Local};
use figura::Template;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::Metadata;
use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    File,
    Directory,
    SymlinkFile,
    SymlinkDirectory,
    Executable,
}

impl Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self {
            FileKind::File => "file",
            FileKind::Directory => "directory",
            FileKind::SymlinkFile => "symlink_file",
            FileKind::SymlinkDirectory => "symlink_directory",
            FileKind::Executable => "executable",
        };

        write!(f, "{}", kind_str)
    }
}

impl<'de> Deserialize<'de> for FileKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;

        match s {
            "file" => Ok(FileKind::File),
            "directory" => Ok(FileKind::Directory),
            "symlink_file" => Ok(FileKind::SymlinkFile),
            "symlink_directory" => Ok(FileKind::SymlinkDirectory),
            "executable" => Ok(FileKind::Executable),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown file kind: {}",
                s
            ))),
        }
    }
}

impl FileKind {
    pub fn from_path<P: AsRef<Path>>(path: P) -> (Self, Metadata) {
        let metadata = std::fs::symlink_metadata(&path).unwrap();

        if metadata.file_type().is_symlink() {
            let target_metadata = std::fs::metadata(&path).unwrap();

            if target_metadata.is_dir() {
                (FileKind::SymlinkDirectory, metadata)
            } else {
                (FileKind::SymlinkFile, metadata)
            }
        } else if metadata.is_dir() {
            (FileKind::Directory, metadata)
        } else if metadata.permissions().mode() & 0o111 != 0 {
            (FileKind::Executable, metadata)
        } else {
            (FileKind::File, metadata)
        }
    }
}

pub fn execute(args: &ListArgs, config: &ListConfig) -> Result<(), PlsError> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let used_variables = config.list_variables();
    let mut context = HashMap::new();
    let mut table = Table::new().padding(config.padding);

    if !config.headers.is_empty() {
        table.add_headers(config.headers.as_slice());
    }

    let mut row = Vec::new();

    let templates = config
        .format
        .iter()
        .map(|t| Template::<'{', '}'>::parse(t))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| PlsError::ParsingError(format!("Template parsing error: {}", e)))?;

    for (entry, i) in DirWalk::new(&args.path)
        .skip_hidden(!args.all)
        .max_depth(args.depth)
        .sort_by(|a, b| {
            // Sort directories first, then files, then symlinks
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        })
    {
        let name = entry.file_name();

        let Some(name) = name.to_str() else {
            writeln!(handle, "Skipping entry with faulty name")?;
            continue;
        };

        if !args.all && name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());

        for t in &templates {
            match t.format(&context) {
                Ok(f) => row.push((f, t.alignment())),
                Err(e) => {
                    writeln!(handle, "{}", e)?;
                    continue;
                }
            }
        }

        table.add_row(row.as_slice());

        row.clear();
        context.clear();
    }

    writeln!(handle, "total: {}", table.rows().len())?;
    writeln!(handle, "{}", table)?;

    Ok(())
}
