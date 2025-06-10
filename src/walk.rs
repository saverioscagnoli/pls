use getset::{Setters, WithSetters};
use std::{
    fs::{DirEntry, ReadDir},
    path::{Path, PathBuf},
};

#[derive(Debug, Setters, WithSetters)]
pub struct Walk {
    stack: Vec<(ReadDir, usize)>, // store (ReadDir, depth)
    #[getset(set_with = "pub")]
    max_depth: usize,
}

impl Walk {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let rd = std::fs::read_dir(&path).expect("Root directory should be valid");
        Self {
            stack: vec![(rd, 1)],
            max_depth: usize::MAX,
        }
    }
}

impl Iterator for Walk {
    type Item = (DirEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((rd, depth)) = self.stack.last_mut() {
            let current_depth = *depth; // Capture the current depth

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

            // If it's a directory and we haven't exceeded max depth, add it to stack
            if ft.is_dir() && current_depth + 1 <= self.max_depth {
                if let Ok(subrd) = std::fs::read_dir(&e.path()) {
                    self.stack.push((subrd, current_depth + 1));
                }
            }

            // Return the current entry with its correct depth
            return Some((e, current_depth));
        }
        None
    }
}
