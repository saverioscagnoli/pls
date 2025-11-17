use crate::{SizeArgs, config::SizeUnit, walk::ThreadedWalk};

use std::path::Path;

fn should_skip_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/proc") || path_str.starts_with("/sys") || path_str.starts_with("/dev")
}

pub fn execute(args: &SizeArgs) {
    if args.path.is_file() {
        let metadata = args.path.metadata().expect("Failed to get metadata");

        println!(
            "{} {}",
            args.path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            SizeUnit::Auto.format_bytes(metadata.len())
        );

        return;
    }

    let mut total = 0;

    for (path, _) in ThreadedWalk::new(&args.path)
        .max_depth(args.depth)
        .skip_hidden(!args.all)
    {
        if should_skip_path(&path) {
            continue;
        }

        if path.is_file() {
            let metadata = path.metadata().expect("Failed to get metadata");
            if metadata.file_type().is_file() {
                total += metadata.len();
            }
        }
    }

    println!(
        "{} {}",
        std::fs::canonicalize(&args.path)
            .unwrap()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        SizeUnit::Auto.format_bytes(total)
    )
}
