use std::collections::HashMap;
use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;

use crate::error::PlsError;

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

pub fn format_template(template: &str, values: &HashMap<&str, String>) -> Result<String, PlsError> {
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
                        None => return Err(PlsError::NoClosedBracket),
                    }
                }

                match values.get(var_str.as_str()) {
                    Some(value) => result.push_str(value),
                    None => return Err(PlsError::NoValueForTemplate(var_str)),
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

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_basic() {
        let values = HashMap::from([("name", "John".to_string()), ("age", "25".to_string())]);
        let template = "Hello, my name is {name} and I am {age}";

        assert_eq!(
            format_template(template, &values).unwrap(),
            "Hello, my name is John and I am 25"
        );
    }

    fn template_parsing_error() {}
}
