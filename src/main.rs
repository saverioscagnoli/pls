mod config;
mod table;
mod utils;
mod walk;

use crate::{config::Config, table::Table, walk::DirWalk};
use chrono::{DateTime, Local};
use clap::Parser;
use figura::{Template, Value};
use smacro::map;
use std::{collections::HashMap, path::PathBuf, usize};

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(index = 1, default_value = ".")]
    path: PathBuf,

    #[arg(short, long, default_value = "1")]
    depth: usize,

    #[arg(short, long, default_value = "false")]
    all: bool,

    #[arg(short, long, default_value = "false")]
    timed: bool,
}

fn main() {
    let args = Args::parse();
    let mut t = None;

    if args.timed {
        use std::time::Instant;
        t = Some(Instant::now());
    }

    let config = Config::parse();

    let mut flags = HashMap::new();

    for v in Config::VARIABLES {
        flags.insert(v, false);
    }

    for t in &config.ls.templates {
        for v in Config::VARIABLES {
            if t.contains(v) {
                flags.insert(v, true);
            }
        }
    }

    let templates = config
        .ls
        .templates
        .iter()
        .filter_map(|t| Template::<'{', '}'>::parse(t).ok())
        .collect::<Vec<_>>();

    let walker = DirWalk::new(&args.path)
        .skip_hidden(!args.all)
        .max_depth(args.depth);

    let mut table = Table::new().padding(config.ls.padding);

    table.add_headers(config.ls.headers.as_slice());

    let mut context = map![];
    let mut row = Vec::new();

    for (entry, depth) in walker {
        context.clear();
        row.clear();

        let Ok(meta) = entry.metadata() else {
            continue;
        };

        if flags["name"] {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if args.all {
                if name.starts_with('.') {
                    context.insert("name", Value::String(name.into_owned()));
                } else {
                    context.insert("name", Value::String(format!(" {}", name.into_owned())));
                }
            } else {
                context.insert("name", Value::String(name.into_owned()));
            }
        }

        if flags["type"] {
            let ft = meta.file_type();

            if ft.is_dir() {
                context.insert("type", Value::String("directory".into()));
            } else if ft.is_file() {
                if utils::is_executable(&meta) {
                    context.insert("type", Value::String("executable".into()));
                } else {
                    context.insert("type", Value::String("file".into()));
                }
            } else if ft.is_symlink() {
                context.insert("type", Value::String("symlink".into()));
            } else {
                context.insert("type", Value::String("unknown".into()));
            }
        }

        if flags["depth"] {
            context.insert("depth", Value::Int(depth as i64 - 1));
        }

        if flags["permissions"] {
            context.insert(
                "permissions",
                Value::String(utils::display_permissions(&meta)),
            );
        }

        if flags["size"] {
            context.insert("size", Value::Int(meta.len() as i64));
        }

        if flags["last_modified"] {
            if let Ok(m) = meta.modified() {
                let local = DateTime::<Local>::from(m);
                let formatted = local.format(&config.ls.time_format).to_string();

                context.insert("last_modified", Value::String(formatted));
            } else {
                context.insert("last_modified", Value::String("unknown".into()));
            }
        }

        if flags["nlink"] {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                context.insert("nlink", Value::Int(meta.nlink() as i64));
            }

            #[cfg(windows)]
            {
                context.insert("nlink", Value::Int(1)); // Windows does not support nlink
            }
        }

        for t in &templates {
            match t.format(&context) {
                Ok(f) => row.push((f, t.alignment())),
                Err(e) => {
                    eprintln!("Error formatting template: {}", e);
                    continue;
                }
            }
        }

        table.add_row(row.as_slice());
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);

    if let Some(t) = t {
        println!("done in {:.2?}", t.elapsed());
    }
}
