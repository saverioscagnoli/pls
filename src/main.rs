mod args;
mod find;
mod list;
mod util;

use crate::{args::Args, find::find, list::list};

fn main() {
    let args = match Args::parse() {
        Ok(a) => a,
        Err(e) => return eprintln!("{}", e),
    };

    let config_dir = util::config_dir();

    if !config_dir.exists()
        && let Err(e) = std::fs::create_dir_all(&config_dir)
    {
        eprintln!("failed to create {}: {}", config_dir.display(), e);
        std::process::exit(1);
    }

    let config_content =
        std::fs::read_to_string(util::config_file()).expect("Failed to read config file");

    match args {
        Args::List(a) => {
            if let Err(e) = list(a) {
                eprintln!("{}", e);
            }
        }
        Args::Find(a) => {
            if let Err(e) = find(a) {
                eprintln!("{}", e)
            }
        }
    }
}
