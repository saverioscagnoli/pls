use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;

pub fn display_permissions(meta: &Metadata) -> String {
    let mode = meta.permissions().mode();
    let mut result = String::new();

    // User
    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group
    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Others
    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

pub fn is_executable(meta: &Metadata) -> bool {
    let mode = meta.permissions().mode();
    mode & 0o111 != 0 // Check if any execute bit is set
}
