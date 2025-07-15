use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use std::{
    fs::{self, ReadDir},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Debug, Clone)]
pub struct WalkOptions {
    skip_hidden: bool,
    max_depth: usize,
    follow_symlinks: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            skip_hidden: true,
            max_depth: usize::MAX,
            follow_symlinks: false,
        }
    }
}

pub struct DirWalk {
    root: Option<PathBuf>,
    stack: Vec<(ReadDir, usize)>,
    options: WalkOptions,
}

impl DirWalk {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            root: Some(path.as_ref().to_path_buf()),
            stack: Vec::new(),
            options: WalkOptions::default(),
        }
    }

    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.options.skip_hidden = skip;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = depth;
        self
    }
}

impl Iterator for DirWalk {
    type Item = (fs::DirEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.root.take() {
            self.stack.push((std::fs::read_dir(root).ok()?, 1));
        }

        while let Some((mut rd, depth)) = self.stack.pop() {
            if depth > self.options.max_depth {
                continue;
            }

            if let Some(entry) = rd.next() {
                self.stack.push((rd, depth));

                if let Ok(entry) = entry {
                    if self.options.skip_hidden
                        && entry.file_name().to_string_lossy().starts_with('.')
                    {
                        continue;
                    }

                    if let Ok(ft) = entry.file_type() {
                        if ft.is_symlink() {
                            return Some((entry, depth));
                        }

                        if ft.is_dir()
                            && let Ok(sub_dir) = std::fs::read_dir(entry.path())
                        {
                            self.stack.push((sub_dir, depth + 1));
                        }

                        return Some((entry, depth));
                    }
                }
            }
        }

        None
    }
}

pub struct ThreadedWalk {
    rx: Option<Receiver<(PathBuf, usize)>>,
    path: PathBuf,
    options: WalkOptions,
    started: bool,
}

impl ThreadedWalk {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ThreadedWalk {
            rx: None,
            path: path.as_ref().to_path_buf(),
            options: WalkOptions::default(),
            started: false,
        }
    }

    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.options.skip_hidden = skip;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = depth;
        self
    }

    fn start(&mut self) {
        if self.started {
            return;
        }

        let (tx, rx) = mpsc::channel();
        let path = self.path.clone();
        let options = self.options.clone();

        rayon::spawn(move || {
            Self::walk(path, &tx, false, &options, 1);
        });

        self.rx = Some(rx);
        self.started = true;
    }

    fn walk(
        path: PathBuf,
        tx: &Sender<(PathBuf, usize)>,
        is_file: bool,
        options: &WalkOptions,
        depth: usize,
    ) {
        // Check if the maximum depth has been reached
        if depth > options.max_depth {
            return;
        }

        // Duplicate the sender.send function
        // to avoid cloning the path, which can improve performance
        if is_file {
            // If this is a file, just send the path and return
            let _ = tx.send((path, depth));
            return;
        }

        let Ok(entries) = std::fs::read_dir(&path) else {
            let _ = tx.send((path, depth));
            return;
        };

        // If this point is reached, it means we are processing a directory
        // Send the directory path and depth
        let _ = tx.send((path, depth));

        // Separate into files and directories
        entries
            .par_bridge()
            .into_par_iter()
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') && options.skip_hidden {
                        return;
                    }
                }

                let path = entry.path();

                match entry.file_type() {
                    Ok(ft) if ft.is_dir() => {
                        if options.follow_symlinks || !ft.is_symlink() {
                            // If it's a directory, recursively walk it
                            Self::walk(path, tx, false, options, depth + 1);
                        }
                    }

                    Ok(ft) if ft.is_file() => {
                        // If it's a file, send the path
                        let _ = tx.send((path, depth));
                    }

                    _ => {}
                }
            });
    }
}

impl Iterator for ThreadedWalk {
    type Item = (PathBuf, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.start();
        self.rx.as_ref().and_then(|rx| rx.recv().ok())
    }
}
