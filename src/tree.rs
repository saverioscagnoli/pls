use std::path::{Path, PathBuf};

pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
}

pub struct WalkOptions {
    pub skip_hidden: bool,
    pub skip_symlinks: bool,
    pub skip_dirs: Vec<PathBuf>,
    pub max_depth: usize,
    pub sort_fn: Option<Box<dyn Fn(&Path, &Path) -> std::cmp::Ordering>>,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            skip_hidden: true,
            skip_symlinks: true,
            skip_dirs: vec![
                PathBuf::from("/proc"),
                PathBuf::from("/run"),
                PathBuf::from("/sys"),
                PathBuf::from("/dev"),
            ],
            max_depth: usize::MAX,
            sort_fn: None,
        }
    }
}

pub struct SyncWalk {
    options: WalkOptions,
    // Store (path, depth) pairs instead of just paths
    stack: Vec<(PathBuf, usize)>,
    root_depth: usize,
}

impl SyncWalk {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root = path.as_ref().to_path_buf();
        Self {
            options: WalkOptions::default(),
            // Start with depth 0 for the root
            stack: vec![(root, 0)],
            root_depth: 0,
        }
    }

    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.options.skip_hidden = skip;
        self
    }

    pub fn skip_symlinks(mut self, skip: bool) -> Self {
        self.options.skip_symlinks = skip;
        self
    }

    pub fn skip_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.options.skip_dirs = dirs;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = depth;
        self
    }

    pub fn sort_by<F>(mut self, sort_fn: F) -> Self
    where
        F: Fn(&Path, &Path) -> std::cmp::Ordering + 'static,
    {
        self.options.sort_fn = Some(Box::new(sort_fn));
        self
    }
}

impl Iterator for SyncWalk {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entry_path, current_depth)) = self.stack.pop() {
            // Skip if max depth is reached
            if current_depth > self.options.max_depth {
                continue;
            }

            // Skip if it's a symlink and skipping is enabled
            if self.options.skip_symlinks && entry_path.is_symlink() {
                continue;
            }

            let name = match entry_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => entry_path.to_string_lossy().to_string(),
            };

            // Skip if it's a hidden file or directory and skipping is enabled
            if self.options.skip_hidden && name.starts_with('.') && name.len() > 1 {
                continue;
            }

            let is_dir = entry_path.is_dir();

            if is_dir {
                // Skip if it's a blacklisted directory
                if self.options.skip_dirs.iter().any(|d| d == &entry_path) {
                    continue;
                }

                // Only recurse if we haven't reached max depth
                if current_depth < self.options.max_depth {
                    // Collect child paths
                    if let Ok(entries) = std::fs::read_dir(&entry_path) {
                        let mut children: Vec<_> = entries
                            .flatten()
                            .map(|entry| (entry.path(), current_depth + 1))
                            .collect();

                        // Sort if a sort function is provided
                        if let Some(ref sort_fn) = self.options.sort_fn {
                            children.sort_by(|a, b| sort_fn(&a.0, &b.0));
                        }

                        // Add to stack in reverse order since we pop from the end
                        // This ensures we process in the correct sorted order
                        for child in children.into_iter().rev() {
                            self.stack.push(child);
                        }
                    }
                }
            }

            return Some(Entry {
                name,
                path: entry_path,
                is_dir,
                depth: current_depth,
            });
        }
        None
    }
}
