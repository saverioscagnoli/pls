use crate::args::ListArgs;
use crate::config::{ListConfig, ListVariable, SizeUnit};
use crate::err::PlsError;
use crate::table::Table;
use crate::util;
use crate::walk::DirWalk;
use chrono::{DateTime, Local};
use figura::{Template, Value};
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
        let name = entry.file_name().to_string_lossy().to_string();

        if !args.all && name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let (kind, metadata) = FileKind::from_path(&path);

        for var in &used_variables {
            match var {
                ListVariable::Name => {
                    context.insert("name", Value::String(name.to_string()));
                }
                ListVariable::Path => {
                    context.insert("path", Value::String(path.to_string_lossy().to_string()));
                }

                ListVariable::Kind => {
                    context.insert("kind", Value::String(kind.to_string()));
                }

                ListVariable::Size => {
                    context.insert(
                        "size",
                        Value::String(config.size_unit.format_bytes(metadata.len())),
                    );
                }

                ListVariable::Depth => {
                    context.insert("depth", Value::Int(i as i64));
                }

                ListVariable::Icon => {
                    let icon = match kind {
                        FileKind::File => config
                            .icons
                            .extensions
                            .get(ext)
                            .unwrap_or(&config.icons.file),
                        FileKind::Directory => &config.icons.directory,
                        FileKind::SymlinkFile => &config.icons.symlink_file,
                        FileKind::SymlinkDirectory => &config.icons.symlink_directory,
                        FileKind::Executable => &config.icons.executable,
                    };

                    context.insert("icon", Value::String(icon.to_string()));
                }

                ListVariable::Permissions => {
                    context.insert(
                        "permissions",
                        Value::String(util::permissions_to_string(metadata.mode())),
                    );
                }

                ListVariable::Created => {
                    if let Ok(ctime) = metadata.created() {
                        let date = DateTime::<Local>::from(ctime)
                            .format(&config.created_format)
                            .to_string();

                        context.insert("created", Value::String(date));
                    } else {
                        context.insert("created", Value::Str("N/A"));
                    }
                }

                ListVariable::Modified => {
                    if let Ok(mtime) = metadata.modified() {
                        let date = DateTime::<Local>::from(mtime)
                            .format(&config.modified_format)
                            .to_string();

                        context.insert("modified", Value::String(date));
                    } else {
                        context.insert("modified", Value::Str("N/A"));
                    }
                }

                ListVariable::Accessed => {
                    if let Ok(atime) = metadata.accessed() {
                        let date = DateTime::<Local>::from(atime)
                            .format(&config.accessed_format)
                            .to_string();

                        context.insert("accessed", Value::String(date));
                    } else {
                        context.insert("accessed", Value::Str("N/A"));
                    }
                }

                ListVariable::Owner => {
                    #[cfg(target_family = "unix")]
                    {
                        use users::get_user_by_uid;

                        let uid = metadata.uid();

                        if let Some(user) = get_user_by_uid(uid) {
                            context.insert(
                                "owner",
                                Value::String(user.name().to_string_lossy().to_string()),
                            );
                        } else {
                            context.insert("owner", Value::Str("N/A"));
                        }
                    }

                    #[cfg(not(target_family = "unix"))]
                    {
                        context.insert("owner", Value::Str("N/A"));
                    }
                }

                ListVariable::Group => {
                    #[cfg(target_family = "unix")]
                    {
                        use users::get_group_by_gid;

                        let gid = metadata.gid();

                        if let Some(group) = get_group_by_gid(gid) {
                            context.insert(
                                "group",
                                Value::String(group.name().to_string_lossy().to_string()),
                            );
                        } else {
                            context.insert("group", Value::Str("N/A"));
                        }
                    }

                    #[cfg(not(target_family = "unix"))]
                    {
                        context.insert("group", Value::Str("N/A"));
                    }
                }

                ListVariable::NLink => {
                    context.insert("nlink", Value::Int(metadata.nlink() as i64));
                }
            }
        }

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
