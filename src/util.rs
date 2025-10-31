use crate::style::VariableStyle;
use std::collections::HashMap;

pub fn keep_letters_whitespace(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .collect()
}

pub fn permissions_to_string(mode: u32) -> String {
    let mut perms = String::with_capacity(9);

    let user_perms = (mode >> 6) & 0o7;
    let group_perms = (mode >> 3) & 0o7;
    let other_perms = mode & 0o7;

    for &perm in &[user_perms, group_perms, other_perms] {
        perms.push(if perm & 0o4 != 0 { 'r' } else { '-' });
        perms.push(if perm & 0o2 != 0 { 'w' } else { '-' });
        perms.push(if perm & 0o1 != 0 { 'x' } else { '-' });
    }

    perms
}

/// Apply a `VariableStyle` from a style map to a string.
/// If no style is present for `key`, returns the original string.
pub fn apply_style<S: AsRef<str>>(
    style_map: &HashMap<String, VariableStyle>,
    key: &str,
    s: S,
) -> String {
    if let Some(style) = style_map.get(key) {
        style.apply(s)
    } else {
        s.as_ref().to_string()
    }
}
