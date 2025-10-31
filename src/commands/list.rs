use crate::{
    Args,
    config::{ListConfig, ListVariable},
    style::{self},
    table::Table,
    util,
    walk::DirWalker,
};
use chrono::{DateTime, Local};
use figura::{Template, Value};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::Metadata,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

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

pub fn execute(args: &Args, config: &ListConfig) {
    let templates = config
        .format
        .iter()
        .filter_map(|s| Template::<'{', '}'>::parse(s).ok())
        .collect::<Vec<_>>();

    let mut context = HashMap::new();
    let mut styled_context = HashMap::new();
    let mut table = Table::new().padding(config.padding);
    let mut row = Vec::new();

    for (entry, depth) in DirWalker::new(&args.path)
        .max_depth(args.depth)
        .skip_hidden(!args.all)
    {
        let path = entry.path();
        let (kind, metadata) = FileKind::from_path(&path);

        let name = entry.file_name().to_string_lossy().to_string();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        let path = path.to_string_lossy().to_string();
        let icon = "f".to_string();
        let size = metadata.len();
        let permissions = util::permissions_to_string(metadata.mode());
        let created = metadata
            .created()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let modified = metadata
            .modified()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let accessed = metadata
            .accessed()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let owner = users::get_user_by_uid(metadata.uid())
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| metadata.uid().to_string());

        let group = users::get_group_by_gid(metadata.gid())
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or_else(|| metadata.gid().to_string());

        let nlink = metadata.nlink();

        context.insert("name", Value::String(name.clone()));
        context.insert(
            "extension",
            figura::Value::String(extension.unwrap_or("".to_string())),
        );
        context.insert("path", Value::String(path.clone()));
        context.insert("kind", Value::String(kind.to_string()));
        context.insert("icon", Value::String(icon));
        context.insert("depth", Value::Int(depth as i64));
        context.insert("size", Value::Int(size as i64));
        context.insert("permissions", Value::String(permissions.clone()));
        context.insert("created", Value::String(created.clone()));
        context.insert("modified", Value::String(modified.clone()));
        context.insert("accessed", Value::String(accessed.clone()));
        context.insert("owner", Value::String(owner.clone()));
        context.insert("group", Value::String(group.clone()));
        context.insert("nlink", Value::Int(nlink as i64));

        let name = config.apply_field_style("name", &name, &context);
        let path = config.apply_field_style("path", &path, &context);
        let kind = config.apply_field_style("kind", &kind.to_string(), &context);
        let icon = config.apply_field_style("icon", "f", &context);
        let size_str = config.apply_field_style(
            "size",
            &config.size_unit.format_bytes(size),
            &styled_context,
        );
        let permissions = config.apply_field_style("permissions", &permissions, &context);
        let created = config.apply_field_style("created", &created, &context);
        let modified = config.apply_field_style("modified", &modified, &context);
        let accessed = config.apply_field_style("accessed", &accessed, &context);
        let owner = config.apply_field_style("owner", &owner, &context);
        let group = config.apply_field_style("group", &group, &context);
        let nlink = config.apply_field_style("nlink", &nlink.to_string(), &context);

        styled_context.insert("name", Value::String(name));
        styled_context.insert("path", Value::String(path));
        styled_context.insert("kind", Value::String(kind));
        styled_context.insert("icon", Value::String(icon));
        styled_context.insert("size", Value::String(size_str));
        styled_context.insert("permissions", Value::String(permissions));
        styled_context.insert("created", Value::String(created));
        styled_context.insert("modified", Value::String(modified));
        styled_context.insert("accessed", Value::String(accessed));
        styled_context.insert("owner", Value::String(owner));
        styled_context.insert("group", Value::String(group));
        styled_context.insert("nlink", Value::String(nlink));

        for t in &templates {
            if let Ok(output) = t.format(&styled_context) {
                row.push((output, t.alignment()));
            }
        }

        table.add_row(row.as_slice());

        row.clear();
        context.clear();
        styled_context.clear();
    }

    println!("{}", table);
}
