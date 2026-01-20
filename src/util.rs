use crate::{
    config::{ConfigKind, LsConfig},
    consts,
};
use chrono::{DateTime, Local};
use figura::{Context, Value};
use std::{
    fs::{self},
    io::{self, Write},
    path::PathBuf,
    time::SystemTime,
};

pub fn config_path(kind: ConfigKind) -> Option<PathBuf> {
    let Some(dir) = dirs::config_dir() else {
        return None;
    };

    fs::create_dir_all(&dir).ok()?;

    let dir = dir.join(consts::PLS_CONFIG_DIRNAME);

    match kind {
        ConfigKind::List => Some(dir.join(consts::LS_CONFIG_FILENAME)),
        ConfigKind::Find => Some(dir.join(consts::FIND_CONFIG_FILENAME)),
    }
}

pub fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

#[cfg(unix)]
fn format_permissions(mode: u32) -> String {
    let user = triplet(mode, 6);
    let group = triplet(mode, 3);
    let other = triplet(mode, 0);

    format!("{}{}{}", user, group, other)
}

#[cfg(unix)]
fn triplet(mode: u32, offset: u32) -> String {
    let r = if mode & (0o4 << offset) != 0 {
        'r'
    } else {
        '-'
    };

    let w = if mode & (0o2 << offset) != 0 {
        'w'
    } else {
        '-'
    };

    let x = if mode & (0o1 << offset) != 0 {
        'x'
    } else {
        '-'
    };

    format!("{}{}{}", r, w, x)
}

pub fn is_executable(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();

        return mode & 0o111 != 0;
    }

    false
}

pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub depth: usize,
    pub ext: String,
    pub perms: String,
    pub size: u64,
    pub modified: SystemTime,
    pub is_file: bool,
    pub is_directory: bool,
    pub is_executable: bool,
    pub is_symlink_file: bool,
    pub is_symlink_directory: bool,
    pub is_broken_symlink: bool,
}

impl FileInfo {
    pub fn from_dir_entry(entry: fs::DirEntry, depth: usize) -> Option<Self> {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        let link_meta = if path.is_symlink() {
            fs::symlink_metadata(&path).ok()
        } else {
            None
        };

        let ext = path
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let meta = entry.metadata().ok()?;

        let is_symlink = link_meta.is_some();
        let is_directory = meta.is_dir();
        let is_file = meta.is_file();

        let is_broken_symlink = if is_symlink {
            path.read_link().is_err() || !path.exists()
        } else {
            false
        };

        let is_symlink_directory = is_symlink && is_directory && !is_broken_symlink;
        let is_symlink_file = is_symlink && is_file && !is_broken_symlink;

        #[cfg(unix)]
        let perms = {
            use std::os::unix::fs::PermissionsExt;
            let mode = meta.permissions().mode();
            format_permissions(mode)
        };

        #[cfg(not(unix))]
        let perms = if meta.permissions().readonly() {
            "r--".to_string()
        } else {
            "rw-".to_string()
        };

        let size = meta.len();

        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        Some(Self {
            name,
            path,
            depth,
            ext,
            perms,
            size,
            modified,
            is_file,
            is_directory,
            is_executable: is_executable(&meta),
            is_symlink_file,
            is_symlink_directory,
            is_broken_symlink,
        })
    }


    #[rustfmt::skip]
    pub fn write_to_context(&self, ctx: &mut Context, config: &LsConfig) {
        ctx.insert("name", Value::owned_str(self.name.clone()));
        ctx.insert("path", Value::owned_str(self.path.to_string_lossy().to_string()));
        ctx.insert("depth", Value::Int(self.depth as i64));
        ctx.insert("icon", Value::owned_str(config.resolve_icon(&self)));
        ctx.insert("ext", Value::owned_str(self.ext.clone()));
        ctx.insert("perms", Value::owned_str(self.perms.clone()));
        ctx.insert("size", Value::owned_str(config.size_unit.format(self.size)));
        ctx.insert("modified",Value::owned_str(DateTime::<Local>::from(self.modified).format(&config.date_format).to_string()));
        ctx.insert("is_file",Value::Bool(self.is_file));
        ctx.insert("is_directory", Value::Bool(self.is_directory));
        ctx.insert("is_executable", Value::Bool(self.is_executable));
        ctx.insert("is_symlink_file", Value::Bool(self.is_symlink_file));
        ctx.insert("is_symlink_directory", Value::Bool(self.is_symlink_directory));
        ctx.insert("is_broken_symlink", Value::Bool(self.is_broken_symlink));
    }
}
