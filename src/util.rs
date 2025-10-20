use crate::config::{ListVariable, StyleConfig};

pub fn keep_ascii_letters_and_whitespace(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphabetic() { c } else { ' ' })
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
