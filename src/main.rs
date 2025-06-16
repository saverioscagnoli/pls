mod bytes;
mod config;
mod dir;
mod error;
mod git;
mod table;
mod utils;
mod walk;

use crate::{
    bytes::Size,
    config::{Config, FormatPart, TemplateVariable},
    dir::{DetailedEntry, FileKind},
    git::{GitCache, GitStatus},
    table::{Alignment, Table},
    walk::{SyncWalk, ThreadedWalk},
};

use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::de;
use std::{cmp::Ordering, collections::HashMap, path::PathBuf, time::Instant};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// This is the default command, it will trigger if no subcommand is provided.
    /// Its args are defined in `Args`.
    List,

    /// Finds a file or directory by name.
    Find {
        /// The name of the file or directory to search for.
        #[clap(index = 1)]
        name: String,

        /// The root path to start the search from.
        /// Defaults to the current directory.
        #[clap(default_value = ".", index = 2)]
        path: PathBuf,

        /// Show all files, including hidden ones.
        /// Defaults to `false`.
        #[clap(short, long, default_value = "false")]
        all: bool,
    },
}

#[derive(Debug, Parser)]
struct Args {
    /// Path to walk
    #[clap(default_value = ".", index = 1)]
    path: PathBuf,

    // The maximum depth to walk
    #[clap(short, long, default_value = "1")]
    depth: usize,

    /// Show all files, including hidden ones
    #[clap(short, long, default_value = "false")]
    all: bool,

    /// The command to run
    /// If not provided, the default command is `List`.
    #[clap(subcommand)]
    command: Option<Command>,
}

type Walker<T> = Box<dyn Iterator<Item = T>>;

fn main() {
    let args = Args::parse();
    let config = Config::parse();

    match args.command {
        Some(Command::Find { name, path, all }) => find(name, path, all, &config),
        _ => {
            // If the user sets the max depth to >= 3, it makes more sense to use a multithreaded walker
            // to speed up the process
            // If not, use a single-threaded walker
            let walker: Box<dyn Iterator<Item = (DetailedEntry, usize)>> = match args.depth {
                d if d < 3 => Box::new(
                    SyncWalk::new(&args.path)
                        .skip_hidden(!args.all)
                        .max_depth(args.depth)
                        .map(|(entry, path)| {
                            let detailed_entry = DetailedEntry::from(entry);
                            (detailed_entry, path)
                        }),
                ),

                _ => Box::new(
                    ThreadedWalk::new(&args.path)
                        .skip_hidden(!args.all)
                        .max_depth(args.depth)
                        .map(|(path, depth)| {
                            let detailed_entry = DetailedEntry::from(path.as_path());
                            (detailed_entry, depth)
                        }),
                ),
            };

            ls(&args, &config, walker);
        }
    }
}

/// This is the command that lists the files and directories.
/// It's the default behavior if no subcommand is provided
fn ls(args: &Args, conf: &Config, walker: Walker<(DetailedEntry, usize)>) {
    // Table for pretty printing the output
    let mut table: Table<String> = Table::new().padding(conf.ls.padding);

    // Create a vector to hold all the variables that are actually used in the format
    // This is used to calculate only the variables that are actually needed
    // saving some time and memory

    // If `args.path` is not a git repository, the default git cache will be empty.
    // see `GitCache::new`
    let git_cache = GitCache::new(&args.path).unwrap_or_default();

    for (entry, depth) in walker {
        let mut row: Vec<(String, Alignment)> = Vec::new();

        for t in &conf.ls.format {
            let mut formatted = String::new();

            for part in t.iter() {
                match part {
                    FormatPart::Variable(var) => match var {
                        TemplateVariable::Depth => {
                            formatted.push_str(&depth.to_string());
                        }

                        TemplateVariable::Name => {
                            let name = entry.name().to_string();
                            formatted.push_str(&name);
                        }

                        TemplateVariable::Icon => {
                            let icon = match entry.kind() {
                                FileKind::File => {
                                    conf.indicators.file(entry.ext().as_deref().unwrap_or(""))
                                }
                                FileKind::Directory => conf.indicators.dir(entry.name()),
                                _ => conf.indicators.unknown(),
                            };

                            formatted.push_str(&icon);
                        }

                        TemplateVariable::Permissions => {
                            let permissions = entry.permissions();
                            formatted.push_str(&permissions);
                        }

                        TemplateVariable::Size => {
                            let size = entry.size();
                            formatted.push_str(&size.to_string());
                        }

                        TemplateVariable::LastModified => {
                            if let Some(timestamp) = entry.timestamp() {
                                formatted
                                    .push_str(&timestamp.format(&conf.ls.time_format).to_string());
                            } else {
                                formatted.push_str("N/A");
                            }
                        }

                        TemplateVariable::GitStatus => match git_cache.get_status(&entry.path()) {
                            Some(s) => formatted.push_str(&s.to_string()),
                            None => formatted.push_str("N/A"),
                        },

                        TemplateVariable::Nlink => {
                            formatted.push_str(&entry.nlink().to_string());
                        }

                        TemplateVariable::LinkTarget => {
                            if let Some(target) = entry.link_target() {
                                formatted.push_str(&target.to_string_lossy());
                            }
                        }

                        _ => {}
                    },

                    FormatPart::Literal(s) => formatted.push_str(s),
                }
            }

            row.push((formatted, Alignment::Left));
        }

        // Skip empty rows
        if row.is_empty() {
            continue;
        }

        table.add_row(row);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}

fn find(name: String, path: PathBuf, all: bool, _conf: &Config) {
    let t0 = Instant::now();
    let mut c = 0;

    for (path, _) in ThreadedWalk::new(&path).skip_hidden(!all) {
        if path
            .file_name()
            .map(|os_str| os_str.to_string_lossy() == name)
            .unwrap_or(false)
        {
            c += 1;
            println!("{}", path.as_path().display());
        }
    }

    println!("Found {} entries in {:?}", c, t0.elapsed());
}
