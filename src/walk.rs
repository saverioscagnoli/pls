use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct WalkOptions {
    pub root: PathBuf,
    pub all: bool,
    pub max_depth: usize,
    pub follow_symlinks: bool,
    pub threads: usize,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            all: false,
            max_depth: 1,
            follow_symlinks: false,
            threads: 0,
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct Walk {
    options: WalkOptions,
}

impl Walk {
    pub fn root<P: AsRef<Path>>(mut self, root: P) -> Self {
        self.options.root = root.as_ref().to_path_buf();
        self
    }

    pub fn all(mut self, v: bool) -> Self {
        self.options.all = v;
        self
    }

    pub fn max_depth(mut self, v: usize) -> Self {
        self.options.max_depth = v;
        self
    }

    pub fn follow_symlinks(mut self, v: bool) -> Self {
        self.options.follow_symlinks = v;
        self
    }

    pub fn threads(mut self, v: usize) -> Self {
        self.options.threads = v;
        self
    }
}

pub struct WalkIter {
    queue: VecDeque<(fs::ReadDir, usize)>,
    options: WalkOptions,
}

impl IntoIterator for Walk {
    type Item = (fs::DirEntry, usize);
    type IntoIter = WalkIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut queue = VecDeque::new();

        match fs::read_dir(&self.options.root) {
            Ok(r) => queue.push_back((r, 0)),
            Err(e) => eprintln!("Error reading {}: {}", self.options.root.display(), e),
        };

        WalkIter {
            queue,
            options: self.options,
        }
    }
}

impl Iterator for WalkIter {
    type Item = (fs::DirEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (mut rd, depth) = self.queue.pop_front()?;

            while let Some(entry) = rd.next() {
                let Ok(entry) = entry else {
                    continue;
                };

                // Skip hidden entries
                if !self.options.all
                    && entry
                        .file_name()
                        .as_encoded_bytes()
                        .first()
                        .map_or(false, |&b| b == b'.')
                {
                    continue;
                }

                if let Ok(ft) = entry.file_type()
                    && depth < self.options.max_depth - 1
                {
                    if self.options.follow_symlinks
                        && ft.is_symlink()
                        && let Ok(sub_rd) = fs::read_dir(&entry.path())
                    {
                        self.queue.push_back((sub_rd, depth + 1));
                    }

                    // ft.is_dir() and ft.is_symlink() are mutually exclusive
                    if ft.is_dir()
                        && let Ok(sub_rd) = fs::read_dir(&entry.path())
                    {
                        self.queue.push_back((sub_rd, depth + 1));
                    }
                }

                self.queue.push_front((rd, depth));

                return Some((entry, depth));
            }
        }
    }
}
