mod args;
mod commands;
mod walk;

use crate::args::{Args, SubCommand};
use clap::Parser;

fn main() {
    let args = Args::parse();

    match args.subcommand {
        Some(sub) => match sub {
            SubCommand::Find(find) => {}
        },

        // Default list behavior
        None => {
            commands::list::exec(args.list);
        }
    }
}
