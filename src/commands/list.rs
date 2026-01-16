use crate::{args::ListArgs, walk::Walk};
use std::{
    fs::{self, Metadata},
    path::PathBuf,
};

struct EntryInfo {
    name: String,
    path: PathBuf,
    meta: Metadata,
    sym_meta: Option<Metadata>,
}

impl EntryInfo {
    fn from_dir_entry(entry: fs::DirEntry) -> Option<Self> {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let mut sym_meta = None;

        if path.is_symlink() {
            sym_meta = fs::symlink_metadata(&path).ok();
        }

        let meta = entry.metadata().ok()?;

        Some(Self {
            name,
            path,
            meta,
            sym_meta,
        })
    }
}

pub fn exec(args: ListArgs) {
    let mut tree = Walk::default().root(&args.path);
    let mut count = 0;

    tree.bfs(|entry| {
        if entry.path().starts_with("/sys") {
            //   println!("{}", entry.path().display());
        }
        count += 1;
    });

    println!("{}", count);
}
