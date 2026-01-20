use crate::{
    config::LsConfig,
    table::{Alignment, Table},
    util::FileInfo,
    walk::Walk,
};
use figura::{Context, Template};
use std::path::PathBuf;

/// The default behavior of the cli tool.
///
/// This will be flattened into the general args
#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct LsArgs {
    #[clap(index = 1, default_value = ".")]
    pub path: PathBuf,

    #[clap(short, long, default_value_t = false)]
    pub all: bool,

    #[clap(short, long, default_value_t = 1)]
    pub depth: usize,
}

impl Default for LsArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            all: false,
            depth: 1,
        }
    }
}

pub fn exec(args: LsArgs, config: LsConfig) {
    let templates = config
        .format
        .iter()
        .filter_map(|f| Template::<'{', '}'>::compile(f).ok())
        .collect::<Vec<_>>();

    let mut table = Table::<String>::new().padding(config.padding);
    let mut row = Vec::new();
    let mut context = Context::new();
    let mut skipped = 0;

    for (entry, depth) in Walk::default()
        .root(&args.path)
        .all(args.all)
        .max_depth(args.depth)
        .follow_symlinks(false)
    {
        let Some(info) = FileInfo::from_dir_entry(entry, depth) else {
            skipped += 1;
            continue;
        };

        info.write_to_context(&mut context, &config);

        for t in &templates {
            if let Ok(output) = t.format(&context) {
                row.push((output, Alignment::default()));
            }
        }

        table.add_row(row.as_slice());

        row.clear();
        context.clear();
    }

    println!("total {}", table.rows().len());

    if skipped > 0 {
        println!("skipped: {}", skipped);
    }

    println!("{}", table);
}
