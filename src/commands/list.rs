use figura::{Context, Template, Value};

use crate::{
    args::LsArgs,
    config::LsConfig,
    table::{Alignment, Table},
    walk::Walk,
};
use std::{
    fs::{self, Metadata},
    path::PathBuf,
};

struct EntryInfo {
    name: String,
    path: PathBuf,
    meta: Metadata,
    link_meta: Option<Metadata>,
}

impl EntryInfo {
    fn from_dir_entry(entry: fs::DirEntry) -> Option<Self> {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let mut link_meta = None;

        if path.is_symlink() {
            link_meta = fs::symlink_metadata(&path).ok();
        }

        let meta = entry.metadata().ok()?;

        Some(Self {
            name,
            path,
            meta,
            link_meta,
        })
    }
}

fn insert_context(ctx: &mut Context, info: &EntryInfo) {
    ctx.insert("name", Value::owned_str(info.name.clone()));
    ctx.insert("icon", Value::static_str("d"));
}

pub fn exec(args: LsArgs, config: LsConfig) {
    let templates = config
        .format
        .iter()
        .filter_map(|f| Template::<'{', '}'>::compile(f).ok())
        .collect::<Vec<_>>();

    let mut table = Table::<String>::new().padding(config.padding);
    let mut row = Vec::new();
    let mut context = Context::new();
    let mut skipped = 0;

    for (entry, depth) in Walk::default()
        .root(&args.path)
        .all(args.all)
        .max_depth(args.depth)
        .follow_symlinks(false)
    {
        let Some(info) = EntryInfo::from_dir_entry(entry) else {
            skipped += 1;
            continue;
        };

        insert_context(&mut context, &info);

        for t in &templates {
            if let Ok(output) = t.format(&context) {
                row.push((output, Alignment::default()));
            }
        }

        table.add_row(row.as_slice());

        row.clear();
        context.clear();
    }

    println!("total {}", table.rows().len());
    println!("{}", table);
}
