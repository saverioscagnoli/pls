use std::path::PathBuf;

#[derive(Debug)]
pub struct ListArgs {
    path: PathBuf,
    all: bool,
}

#[derive(Debug)]
pub struct FindArgs {
    target: String,
    path: PathBuf,
    all: bool,
}

pub enum Args {
    List(ListArgs),
    Find(FindArgs),
}

impl Args {
    pub fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_env();
        let mut all = false;
        let mut positionals = Vec::new();

        while let Some(arg) = parser.next()? {
            match arg {
                Short('a') | Long("all") => all = true,
                Value(val) => positionals.push(val),
                _ => return Err(arg.unexpected()),
            }
        }

        let mut pos = positionals.into_iter();

        let args = match pos.next() {
            // list behaviour
            // pls
            None => Self::List(ListArgs {
                path: ".".into(),
                all,
            }),

            // find behaviour
            // list <target> [path] ...args
            Some(first) if first == "find" => {
                let target = pos.next().ok_or("find: missing <target>")?.string()?;
                let path = pos.next().map(PathBuf::from).unwrap_or_else(|| ".".into());
                Self::Find(FindArgs { target, path, all })
            }

            // list behaviour
            // pls <path> ...args
            Some(first) => {
                let path = PathBuf::from(&first);
                Self::List(ListArgs { path, all })
            }
        };

        if let Some(extra) = pos.next() {
            return Err(lexopt::Error::UnexpectedArgument(extra));
        }

        Ok(args)
    }
}
