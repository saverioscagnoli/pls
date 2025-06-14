use std::collections::HashMap;
use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;

use nix::libc::RSI;

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

pub fn format_template(template: &str, values: &HashMap<&str, &str>) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                let peeked = chars.peek();

                if peeked == Some(&'{') {
                    result.push('{');
                    continue;
                }

                let mut var_str = String::new();

                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next();
                        break;
                    }

                    match chars.next() {
                        Some(name_ch) => var_str.push(name_ch),
                        None => todo!("Fallback"),
                    }
                }

                match values.get(var_str.as_str()) {
                    Some(value) => result.push_str(value),
                    None => todo!("Fallback"),
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    result.push('}');
                } else {
                    result.push(ch);
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}
