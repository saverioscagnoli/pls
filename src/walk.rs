use std::{
    fs::{DirEntry, ReadDir},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct WalkOptions {
    max_depth: usize,
    skip_hidden: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            max_depth: usize::MAX,
            skip_hidden: true,
        }
    }
}

#[derive(Debug)]
pub struct SyncWalk {
    stack: Vec<(ReadDir, usize)>, // store (ReadDir, depth)
    options: WalkOptions,
}

impl SyncWalk {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let rd = std::fs::read_dir(&path).expect("Root directory should be valid");
        Self {
            stack: vec![(rd, 1)],
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
}

impl Iterator for SyncWalk {
    type Item = (DirEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((rd, depth)) = self.stack.last_mut() {
            let depth = *depth;

            let Some(rd) = rd.next() else {
                self.stack.pop();
                continue;
            };

            let Ok(e) = rd else {
                continue;
            };

            let Ok(ft) = e.file_type() else {
                continue;
            };

            if ft.is_symlink() {
                continue;
            }

            if let Some(name) = e.file_name().to_str() {
                if self.options.skip_hidden && name.starts_with('.') {
                    continue;
                }
            }

            if ft.is_dir() && depth + 1 <= self.options.max_depth {
                if let Ok(subrd) = std::fs::read_dir(&e.path()) {
                    self.stack.push((subrd, depth + 1));
                }
            }

            // Return the current entry with its correct depth
            return Some((e, depth));
        }
        None
    }
}
