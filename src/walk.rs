use std::{
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

pub struct WalkOptions {
    skip_hidden: bool,
    max_depth: usize,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            skip_hidden: true,
            max_depth: usize::MAX,
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
                            continue;
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

// fn next(&mut self) -> Option<Self::Item> {
//     while let Some((mut dir, depth)) = self.stack.pop() {
//         if let Some(entry) = dir.next() {
//             // Put the directory back on the stack since it might have more entries
//             self.stack.push((dir, depth));

//             if let Ok(entry) = entry {
//                 let path = entry.path();
//                 if path.is_dir() {
//                     if let Ok(sub_dir) = std::fs::read_dir(&path) {
//                         self.stack.push((sub_dir, depth + 1));
//                     }
//                 }
//                 return Some((path, depth));
//             }
//         }
//         // If current_dir.next() returned None, the directory is exhausted
//         // and we don't put it back on the stack
//     }
//     None
// }
