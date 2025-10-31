use std::{
    cmp::Ordering,
    fs::{DirEntry, ReadDir},
    path::Path,
    usize,
};

#[derive(Debug)]
enum StackItem {
    ReadDir(ReadDir, usize),
    Entries(Vec<DirEntry>, usize, usize), // entries, depth, index
}

#[derive(Debug)]
pub struct DirWalker {
    stack: Vec<StackItem>,
    max_depth: usize,
    skip_hidden: bool,
    follow_symlinks: bool,
    sort_by: Option<fn(&DirEntry, &DirEntry) -> Ordering>,
}

impl DirWalker {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let rd = std::fs::read_dir(&path).expect("Root directory should be valid");

        Self {
            stack: vec![StackItem::ReadDir(rd, 1)],
            max_depth: usize::MAX,
            skip_hidden: true,
            follow_symlinks: false,
            sort_by: None,
        }
    }

    pub fn max_depth(mut self, val: usize) -> Self {
        self.max_depth = val;
        self
    }

    pub fn skip_hidden(mut self, val: bool) -> Self {
        self.skip_hidden = val;
        self
    }

    pub fn follow_symlinks(mut self, val: bool) -> Self {
        self.follow_symlinks = val;
        self
    }

    pub fn sort_by(mut self, sort_fn: fn(&DirEntry, &DirEntry) -> Ordering) -> Self {
        self.sort_by = Some(sort_fn);
        self
    }
}

impl Iterator for DirWalker {
    type Item = (DirEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.last_mut() {
            match item {
                StackItem::ReadDir(rd, depth) => {
                    let depth = *depth;

                    if self.sort_by.is_some() {
                        // Need to sort, so collect all entries
                        let rd = std::mem::replace(rd, std::fs::read_dir(".").unwrap());
                        let mut entries: Vec<DirEntry> =
                            rd.filter_map(|entry| entry.ok()).collect();

                        if let Some(sort_fn) = self.sort_by {
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
                        if self.skip_hidden && name.starts_with('.') {
                            continue;
                        }
                    }

                    if ft.is_dir() && depth + 1 <= self.max_depth {
                        // Only follow symlinks if the option is set
                        if !ft.is_symlink() || self.follow_symlinks {
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
                        if self.skip_hidden && name.starts_with('.') {
                            continue;
                        }
                    }

                    if ft.is_dir() && depth + 1 <= self.max_depth {
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
