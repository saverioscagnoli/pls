use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::FindArgs;
use crate::config::{Apply, Color, Style};
use crate::walk::ThreadedWalk;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

pub fn execute(args: &FindArgs) {
    let t0 = if args.timed {
        Some(Instant::now())
    } else {
        None
    };

    let mut lock = std::io::stdout().lock();
    let mut output = String::new();

    let paths: Vec<_> = ThreadedWalk::new(&args.root)
        .skip_hidden(!args.all)
        .max_depth(args.depth)
        .follow_symlinks(args.follow_symlinks)
        .collect();

    let green_style = Style {
        foreground: Some(Color::Named("green".to_string())),
        background: None,
        text: None,
    };

    let red_style = Style {
        foreground: Some(Color::Named("red".to_string())),
        background: None,
        text: None,
    };

    let buffer: Vec<_> = paths
        .par_iter()
        .filter_map(|(path, _)| {
            path.file_name()
                .and_then(|f| f.to_str())
                .filter(|n| {
                    if args.exact {
                        *n == args.pattern
                    } else {
                        n.contains(&args.pattern)
                    }
                })
                .map(|_| {
                    let filename = path.file_name().unwrap().to_string_lossy();
                    let parent = path.parent().unwrap_or(Path::new(""));

                    let colored_dir = if parent.as_os_str().is_empty() {
                        String::new()
                    } else {
                        green_style.apply(Some(parent.to_string_lossy().to_string() + "/"))
                    };

                    let colored_name = red_style.apply(Some(filename.to_string()));

                    format!("{}{}", colored_dir, colored_name)
                })
        })
        .collect();

    let count = buffer.len();

    for p in buffer {
        output.push_str(&(p + "\n"));
    }

    _ = lock.write_all(output.as_bytes());

    if let Some(t0) = t0 {
        _ = writeln!(lock, "Found {} entries in {:?}", count, t0.elapsed());
    } else {
        _ = writeln!(lock, "Found {} entries", count);
    }
}
