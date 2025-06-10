use clap::Parser;
use std::{path::PathBuf, time::Instant};

use crate::{dir::DetailedEntry, table::Table, walk::Walk};

mod dir;
mod table;
mod walk;

#[derive(Debug, Parser)]
struct Args {
    /// Path to walk
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut table = Table::new().padding(2);

    for (entry, _) in Walk::new(&args.path)
        .with_max_depth(1)
        .map(|(e, d)| (DetailedEntry::from(e), d))
    {
        let name = entry
            .path()
            .file_name()
            .unwrap_or(entry.path().as_os_str())
            .to_string_lossy()
            .to_string();

        let namepad = match name.starts_with('.') {
            true => "",
            false => " ",
        };

        let name = format!("{namepad}{name}");
        let kind = entry.kind().to_string();

        table.add_row(vec![name, kind]);
    }

    println!("{}", table);
}
