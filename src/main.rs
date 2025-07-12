mod config;
mod style;
mod table;
mod utils;
mod walk;

use crate::{config::Config, table::Table, walk::DirWalk};
use chrono::{DateTime, Local};
use clap::Parser;
use figura::{Template, Value};
use smacro::s;
use std::{collections::HashMap, os::unix::fs::MetadataExt, path::PathBuf};

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(index = 1, default_value = ".")]
    path: PathBuf,

    #[arg(short, long, default_value_t = false)]
    all: bool,

    #[arg(short, long, default_value_t = 1)]
    depth: usize,
}

fn main() {
    let args = Args::parse();
    let conf = Config::parse();

    println!("{:#?}", conf);

    ls(&args, &conf);
}

fn ls(args: &Args, conf: &Config) {
    let mut depth_used = false;
    let mut name_used = false;
    let mut perm_used = false;
    let mut size_used = false;
    let mut lm_used = false;
    let mut nlink_used = false;

    for t in conf.ls.templates.iter() {
        if t.contains("depth") {
            depth_used = true;
        }

        if t.contains("name") {
            name_used = true;
        }

        if t.contains("permissions") {
            perm_used = true;
        }

        if t.contains("size") {
            size_used = true;
        }

        if t.contains("last_modified") {
            lm_used = true;
        }

        if t.contains("nlink") {
            nlink_used = true;
        }
    }

    let templates = conf
        .ls
        .templates
        .iter()
        .filter_map(|t| Template::<'{', '}'>::parse(t).ok())
        .collect::<Vec<_>>();

    let mut table = Table::new().padding(2);

    for (entry, depth) in DirWalk::new(&args.path)
        .skip_hidden(!args.all)
        .max_depth(args.depth)
    {
        let mut row = Vec::new();
        let mut context = HashMap::new();
        let meta = entry.metadata();

        if depth_used {
            context.insert("depth", Value::Int(depth as i64 - 1));
        }

        if name_used {
            let mut name = entry.file_name().to_string_lossy().to_string();

            // Add a space before the name if it's not hidden and all is true
            // This is done to make sure that the alphabetic part of the name
            // is aligned
            if args.all && !name.starts_with(".") {
                name.insert(0, ' ');
            }

            context.insert("name", Value::String(name));
        }

        if perm_used {
            if let Ok(ref meta) = meta {
                let permissions = utils::display_permissions(&meta);
                context.insert("permissions", Value::String(permissions));
            } else {
                context.insert("permissions", Value::String(s!("?")));
            }
        }

        if size_used {
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            context.insert("size", Value::String(s!(size)));
        }

        if lm_used {
            if let Ok(ref meta) = meta
                && let Ok(last_modified) = meta.modified()
            {
                let date: DateTime<Local> = last_modified.into();
                let date = date.format(&conf.ls.time_format);

                context.insert("last_modified", Value::String(s!(date)));
            } else {
                context.insert("last_modified", Value::String(s!("?")));
            }
        }

        if nlink_used {
            if let Ok(ref meta) = meta {
                let nlink = meta.nlink();

                context.insert("nlink", Value::String(s!(nlink)));
            } else {
                context.insert("nlink", Value::String(s!("?")));
            }
        }

        for t in &templates {
            match t.format(&context) {
                Ok(f) => row.push((f, t.alignment())),
                Err(e) => eprintln!("{}", e),
            }
        }

        table.add_row(row);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}
