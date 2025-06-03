use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File { size: u64 },
    Directory,
}

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub depth: usize,
    pub hidden: bool,
}

pub struct WalkOptions {
    skip_hidden: bool,
    depth: usize,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            skip_hidden: true,
            depth: usize::MAX,
        }
    }
}

impl WalkOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.skip_hidden = skip;
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}

pub fn walk_dir<P: AsRef<Path>>(path: P, options: &WalkOptions) -> Vec<Entry> {
    walk_dir_inner(path.as_ref(), options, 0)
}

fn walk_dir_inner(path: &Path, options: &WalkOptions, current_depth: usize) -> Vec<Entry> {
    if current_depth >= options.depth {
        return vec![];
    }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return vec![],
    };

    entries
        .into_par_iter()
        .flat_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().into_string().unwrap_or_default();
            let hidden = name.starts_with('.');

            if options.skip_hidden && hidden {
                return vec![];
            }

            if let Ok(metadata) = entry.metadata() {
                if metadata.file_type().is_symlink() {
                    return vec![];
                }

                if metadata.is_dir() {
                    let mut sub_entries = walk_dir_inner(&path, options, current_depth + 1);

                    sub_entries.push(Entry {
                        name,
                        kind: EntryKind::Directory,
                        path: path.to_string_lossy().to_string(),
                        depth: current_depth,
                        hidden,
                    });

                    return sub_entries;
                } else {
                    let size = metadata.len();

                    return vec![Entry {
                        name,
                        kind: EntryKind::File { size },
                        path: path.to_string_lossy().to_string(),
                        depth: current_depth,
                        hidden,
                    }];
                }
            }

            vec![]
        })
        .collect()
}
