use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;

pub fn get_permissions(metadata: &Metadata) -> String {
    let mode = metadata.permissions().mode();

    format!(
        "{}{}{}",
        display_permission((mode >> 6) & 0o7), // owner
        display_permission((mode >> 3) & 0o7), // group
        display_permission(mode & 0o7),        // others
    )
}

fn display_permission(bits: u32) -> String {
    format!(
        "{}{}{}",
        if bits & 0b100 != 0 { 'r' } else { '-' },
        if bits & 0b010 != 0 { 'w' } else { '-' },
        if bits & 0b001 != 0 { 'x' } else { '-' },
    )
}
