use std::{
    collections::VecDeque,
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
    queue: VecDeque<fs::ReadDir>,
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

// BFS search https://en.wikipedia.org/wiki/Breadth-first_search#Pseudocode
// 1  procedure BFS(G, root) is
// 2      let Q be a queue
// 3      label root as explored
// 4      Q.enqueue(root)
// 5      while Q is not empty do
// 6          v := Q.dequeue()
// 7          if v is the goal then
// 8              return v
// 9          for all edges from v to w in G.adjacentEdges(v) do
// 10              if w is not labeled as explored then
// 11                  label w as explored
// 12                  w.parent := v
// 13                  Q.enqueue(w)

impl Walk {
    pub fn bfs(&mut self, mut f: impl FnMut(&fs::DirEntry)) {
        let root = match fs::read_dir(&self.options.root) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error reading {}: {}", self.options.root.display(), e);
                return;
            }
        };

        self.queue.push_back(root);

        while let Some(rd) = self.queue.pop_front() {
            for entry in rd.filter_map(Result::ok) {
                f(&entry);

                let Ok(ft) = entry.file_type() else {
                    continue;
                };

                if ft.is_dir() {
                    let Ok(sub_rd) = fs::read_dir(&entry.path()) else {
                        continue;
                    };

                    self.queue.push_back(sub_rd);
                    continue;
                }
            }
        }
    }
}
