use std::{
    cmp::Ordering,
    fs::{DirEntry, ReadDir},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct WalkOptions {
    max_depth: usize,
    skip_hidden: bool,
    follow_symlinks: bool,
    sort_by: Option<fn(&DirEntry, &DirEntry) -> Ordering>,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            max_depth: usize::MAX,
            skip_hidden: true,
            follow_symlinks: false,
            sort_by: None,
        }
    }
}

#[derive(Debug)]
enum StackItem {
    ReadDir(ReadDir, usize),
    Entries(Vec<DirEntry>, usize, usize), // entries, depth, index
}

#[derive(Debug)]
pub struct SyncWalk {
    stack: Vec<StackItem>,
    options: WalkOptions,
}

impl SyncWalk {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let rd = std::fs::read_dir(&path).expect("Root directory should be valid");
        Self {
            stack: vec![StackItem::ReadDir(rd, 1)],
            options: WalkOptions::default(),
        }
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = depth;
        self
    }

    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.options.skip_hidden = skip;
        self
    }

    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.options.follow_symlinks = follow;
        self
    }

    pub fn sort_by(mut self, sort_fn: fn(&DirEntry, &DirEntry) -> Ordering) -> Self {
        self.options.sort_by = Some(sort_fn);
        self
    }
}

impl Iterator for SyncWalk {
    type Item = (DirEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.last_mut() {
            match item {
                StackItem::ReadDir(rd, depth) => {
                    let depth = *depth;

                    if self.options.sort_by.is_some() {
                        // Need to sort, so collect all entries
                        let rd = std::mem::replace(rd, std::fs::read_dir(".").unwrap());
                        let mut entries: Vec<DirEntry> =
                            rd.filter_map(|entry| entry.ok()).collect();

                        if let Some(sort_fn) = self.options.sort_by {
                            entries.sort_by(sort_fn);
                        }

                        self.stack.pop();

                        if !entries.is_empty() {
                            self.stack.push(StackItem::Entries(entries, depth, 0));
                        }

                        continue;
                    }

                    // No sorting needed, use ReadDir directly
                    let Some(rd_result) = rd.next() else {
                        self.stack.pop();
                        continue;
                    };

                    let Ok(e) = rd_result else {
                        continue;
                    };

                    let Ok(ft) = e.file_type() else {
                        continue;
                    };

                    if let Some(name) = e.file_name().to_str() {
                        if self.options.skip_hidden && name.starts_with('.') {
                            continue;
                        }
                    }

                    if ft.is_dir() && depth + 1 <= self.options.max_depth {
                        // Only follow symlinks if the option is set
                        if !ft.is_symlink() || self.options.follow_symlinks {
                            if let Ok(subrd) = std::fs::read_dir(&e.path()) {
                                self.stack.push(StackItem::ReadDir(subrd, depth + 1));
                            }
                        }
                    }

                    return Some((e, depth));
                }

                StackItem::Entries(entries, depth, index) => {
                    let depth = *depth;
                    let index = *index;

                    if index >= entries.len() {
                        self.stack.pop();
                        continue;
                    }

                    let entry = if entries.is_empty() {
                        self.stack.pop();
                        continue;
                    } else {
                        // Always take the first element to maintain sorted order
                        entries.remove(0)
                    };

                    let Ok(ft) = entry.file_type() else {
                        eprintln!("cannot get ft for {}", entry.path().display());
                        continue;
                    };

                    if let Some(name) = entry.file_name().to_str() {
                        if self.options.skip_hidden && name.starts_with('.') {
                            continue;
                        }
                    }

                    if ft.is_dir() && depth + 1 <= self.options.max_depth {
                        if let Ok(subrd) = std::fs::read_dir(&entry.path()) {
                            self.stack.push(StackItem::ReadDir(subrd, depth + 1));
                        }
                    }

                    return Some((entry, depth));
                }
            }
        }
        None
    }
}
