use std::path::PathBuf;

/// Returns ~/.config, if set. Otherwise it will return /tmp
pub fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from) {
        if dir.is_absolute() {
            return dir.join("pls");
        }
    }

    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        return home.join(".config").join("pls");
    }

    eprintln!("XDG_CONFIG_HOME is not set. Falling back to /tmp/pls");
    PathBuf::from("/tmp/pls")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.lua")
}
