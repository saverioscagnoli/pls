use git2::{Repository, StatusOptions};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    /// THe file is not tracked by git.
    Untracked,

    /// The file has been modified and the changes are staged.
    Modified,

    /// The file has been deleted and the deletion is staged.
    Deleted,

    /// The file has been renamed and the rename is staged.
    Renamed,

    /// The file is ignored by git.
    Ignored,

    /// The file is in conflict state.
    Conflict,

    /// The file is clean, meaning no changes are staged or unstaged.
    Clean,
}

impl GitStatus {
    fn priority(&self) -> u8 {
        match self {
            GitStatus::Conflict => 6,
            GitStatus::Modified | GitStatus::Deleted | GitStatus::Renamed => 5,
            GitStatus::Untracked => 4,
            GitStatus::Ignored => 3,
            GitStatus::Clean => 1,
        }
    }
}

impl ToString for GitStatus {
    fn to_string(&self) -> String {
        match self {
            GitStatus::Untracked => "Untracked".to_string(),
            GitStatus::Modified => "Modified".to_string(),
            GitStatus::Deleted => "Deleted".to_string(),
            GitStatus::Renamed => "Renamed".to_string(),
            GitStatus::Ignored => "Ignored".to_string(),
            GitStatus::Conflict => "Conflict".to_string(),
            GitStatus::Clean => "Clean".to_string(),
        }
    }
}

impl From<git2::Status> for GitStatus {
    fn from(status: git2::Status) -> Self {
        if status.is_ignored() {
            GitStatus::Ignored
        } else if status.is_conflicted() {
            GitStatus::Conflict
        } else if status.is_wt_new() {
            GitStatus::Untracked
        } else if status.is_wt_modified() {
            GitStatus::Modified
        } else if status.is_wt_deleted() || status.is_index_deleted() {
            GitStatus::Deleted
        } else if status.is_wt_renamed() || status.is_index_renamed() {
            GitStatus::Renamed
        } else if status.is_empty() {
            GitStatus::Clean
        } else {
            GitStatus::Clean // Default case, should not happen
        }
    }
}

pub struct GitCache {
    status: HashMap<PathBuf, GitStatus>,
}

impl GitCache {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let repo = Repository::open(".").unwrap_or_else(|_| {
            panic!(
                "Could not find a git repository at {}",
                root.as_ref().display()
            )
        });

        let mut status_opts = StatusOptions::new();

        status_opts
            .include_untracked(true)
            .renames_head_to_index(true)
            .recurse_ignored_dirs(true)
            .recurse_untracked_dirs(true)
            .include_ignored(true)
            .show(git2::StatusShow::IndexAndWorkdir);

        let statuses = repo
            .statuses(Some(&mut status_opts))
            .expect("Failed to get git statuses");

        let mut status_map = HashMap::new();
        let mut directories: HashMap<PathBuf, Vec<GitStatus>> = HashMap::new();

        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let status = GitStatus::from(entry.status());
                let path: PathBuf = path.into();

                if let Some(parent) = path.parent() {
                    directories
                        .entry(parent.to_path_buf())
                        .or_default()
                        .push(status);
                }

                status_map.insert(path, status);
            }
        }

        for (dir, files) in directories.iter() {
            let highest_status = files
                .iter()
                .max_by_key(|s| s.priority())
                .cloned()
                .unwrap_or(GitStatus::Clean);

            status_map.insert(dir.to_path_buf(), highest_status);
        }

        Self { status: status_map }
    }

    pub fn get_status(&self, path: &Path) -> Option<&GitStatus> {
        self.status.get(path)
    }
}
