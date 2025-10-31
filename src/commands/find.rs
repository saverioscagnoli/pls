use crate::FindArgs;
use crate::walk::ThreadedWalk;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::io::Write;
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
                .map(|_| path.to_string_lossy())
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
