use crate::dir::{EntryKind, WalkOptions, walk_dir};
use bytesize::ByteSize;
use clap::Parser;
use std::{cmp::Ordering, collections::HashMap};

mod dir;
mod icons;

#[derive(Debug, Parser)]
struct Args {
    #[clap(default_value = ".")]
    path: String,

    #[clap(short, long, default_value = "false")]
    all: bool,

    #[clap(short, long, default_value = "false")]
    size: bool,
}

fn main() {
    let args = Args::parse();

    let options = WalkOptions::new()
        .skip_hidden(!args.all)
        .depth(if args.size { usize::MAX } else { 1 });

    let mut entries = walk_dir(&args.path, &options);

    entries.sort_by(|a, b| match (a.kind, b.kind) {
        (EntryKind::Directory, EntryKind::File { .. }) => Ordering::Less,
        (EntryKind::File { .. }, EntryKind::Directory) => Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    let mut file_sizes = HashMap::new();
    let mut dir_sizes = HashMap::new();
    let mut total_size = 0;

    if args.size {
        for entry in &entries {
            if let EntryKind::File { size } = entry.kind {
                file_sizes.insert(entry.path.clone(), size); // Use full path, not just name
            }
        }

        for entry in entries
            .iter()
            .filter(|e| e.depth == 0 && matches!(e.kind, EntryKind::Directory))
        {
            let dir_path = &entry.path;

            let total_size = file_sizes
                .iter()
                .filter_map(|(file_path, &size)| {
                    if file_path.starts_with(dir_path) {
                        Some(size)
                    } else {
                        None
                    }
                })
                .sum::<u64>();

            dir_sizes.insert(dir_path, total_size);
        }

        total_size = file_sizes.values().sum();
    }

    let display = entries
        .iter()
        .filter(|e| e.depth == 0)
        .map(|entry| {
            let (icon, size) = match entry.kind {
                EntryKind::File { size } => (icons::FILE, size),
                EntryKind::Directory => (
                    icons::FOLDER,
                    dir_sizes.get(&entry.path).copied().unwrap_or(0),
                ),
            };

            let spacing = match entry.hidden {
                true => " ",
                false => "  ",
            };

            if args.size {
                format!(" {}{}{} ({})", icon, spacing, entry.name, ByteSize(size))
            } else {
                format!(" {}{}{}", icon, spacing, entry.name)
            }
        })
        .collect::<Vec<_>>();

    if args.size {
        println!("total: {} ({})", display.len(), ByteSize(total_size));
    } else {
        println!("total: {}", display.len());
    }
    println!("{}", display.join("\n"));
}
