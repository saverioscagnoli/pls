use crate::{
    Args,
    config::{FileKind, ListConfig},
    table::Table,
    util,
    walk::DirWalker,
};
use chrono::{DateTime, Local};
use figura::{Template, Value};
use std::{cmp::Ordering, collections::HashMap, fs::DirEntry, os::unix::fs::MetadataExt};

struct FileInfo {
    name: String,
    path: String,
    extension: String,
    kind: FileKind,
    depth: usize,
    size: u64,
    permissions: String,
    created: String,
    modified: String,
    accessed: String,
    owner: String,
    group: String,
    nlink: u64,
}

impl FileInfo {
    fn new(entry: DirEntry, depth: usize, config: &ListConfig) -> Self {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        let (kind, meta) = FileKind::from_path(&path);

        let size = meta.len();
        let permissions = util::permissions_to_string(meta.mode());

        let created = meta
            .created()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format(&config.created_fmt).to_string()
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let modified = meta
            .modified()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format(&config.modified_fmt).to_string()
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let accessed = meta
            .accessed()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format(&config.accessed_fmt).to_string()
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let owner = if cfg!(unix) {
            users::get_user_by_uid(meta.uid())
                .map(|u| u.name().to_string_lossy().to_string())
                .unwrap_or_else(|| meta.uid().to_string())
        } else {
            "N/A".to_string()
        };

        let group = if cfg!(unix) {
            users::get_group_by_gid(meta.gid())
                .map(|g| g.name().to_string_lossy().to_string())
                .unwrap_or_else(|| meta.gid().to_string())
        } else {
            "N/A".to_string()
        };

        let nlink = meta.nlink();

        Self {
            name,
            path: path.to_string_lossy().to_string(),
            extension,
            kind,
            depth: depth - 1,
            size,
            permissions,
            created,
            modified,
            accessed,
            owner,
            group,
            nlink,
        }
    }
}

fn insert_info_raw(map: &mut HashMap<&'static str, Value>, f: &FileInfo) {
    map.insert("name", Value::String(f.name.to_string()));
    map.insert("extension", Value::String(f.extension.to_string()));
    map.insert("path", Value::String(f.path.to_string()));
    map.insert("kind", Value::String(f.kind.to_string()));
    map.insert("depth", Value::Int(f.depth as i64));
    map.insert("size", Value::Int(f.size as i64));
    map.insert("permissions", Value::String(f.permissions.to_string()));
    map.insert("created", Value::String(f.created.to_string()));
    map.insert("modified", Value::String(f.modified.to_string()));
    map.insert("accessed", Value::String(f.accessed.to_string()));
    map.insert("owner", Value::String(f.owner.to_string()));
    map.insert("group", Value::String(f.group.to_string()));
    map.insert("nlink", Value::Int(f.nlink as i64));
}

fn apply_styles(map: &mut HashMap<&'static str, Value>, config: &ListConfig, args: &Args) {
    let context = map.clone();

    if args.pad_names && args.all {
        if let Some(Value::String(name)) = map.get_mut("name") {
            if !name.starts_with('.') {
                *name = format!(" {}", name);
            }
        }
    }

    for field in [
        "name",
        "path",
        "kind",
        "icon",
        "size",
        "permissions",
        "created",
        "modified",
        "accessed",
        "owner",
        "group",
        "nlink",
    ] {
        if let Some(Value::String(s)) = map.get_mut(field) {
            if let Some(style) = config.style.get(field) {
                *s = style.resolve(&s, &context);
            }
        }
    }

    if let Some(Value::Int(d)) = map.get("depth") {
        if let Some(style) = config.style.get("depth") {
            let s = style.resolve(&d.to_string(), &context);

            map.insert("depth_str", Value::String(s));
        }
    }

    if let Some(Value::Int(size)) = map.get("size") {
        let s = config.size_unit.format_bytes(*size as u64);

        if let Some(style) = config.style.get("size") {
            let s = style.resolve(&s, &context);

            map.insert("size", Value::String(s));
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
    let mut table = Table::new().padding(config.padding);
    let mut row = Vec::new();

    for (entry, depth) in DirWalker::new(&args.path)
        .max_depth(args.depth)
        .skip_hidden(!args.all)
        .follow_symlinks(args.follow_symlinks)
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
        let info = FileInfo::new(entry, depth, config);

        insert_info_raw(&mut context, &info);

        let icon = config.icons.resolve(&context);

        context.insert("icon", Value::String(icon));

        apply_styles(&mut context, &config, args);

        for t in &templates {
            if let Ok(output) = t.format(&context) {
                row.push((output, t.alignment()));
            }
        }

        table.add_row(row.as_slice());

        row.clear();
        context.clear();
    }

    println!("total {}", table.rows().len());
    println!("{}", table);
}
