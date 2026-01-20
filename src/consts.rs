pub const PLS_CONFIG_DIRNAME: &str = "pls";
pub const LS_CONFIG_FILENAME: &'static str = "pls.toml";
pub const FIND_CONFIG_FILENAME: &'static str = "find.toml";

pub const RESET_CONFIG_PROMPT: &str =
    "Are you sure you want to reset the configuration file(s)? [y/n] ";

pub const UNITS: [(u64, &str); 5] = [
    (1024_u64.pow(4), "T"),
    (1024_u64.pow(3), "G"),
    (1024_u64.pow(2), "M"),
    (1024, "K"),
    (1, "B"),
];
