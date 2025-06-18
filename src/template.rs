use serde::Deserialize;

use crate::error::PlsError;

/// Alignment for formatting string in the terminal
/// These can be parsed in the configuration by putting
/// one of the following characters:
///
/// - '<' -> Left alignment
/// - '>' -> Right alignment
/// - '^' -> Center alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

impl Alignment {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '<' => Some(Alignment::Left),
            '>' => Some(Alignment::Right),
            '^' => Some(Alignment::Center),
            _ => None,
        }
    }
}

/// Represents the different template variables that can be used in the output format.
/// The enum variants dont carry any data, because they are used to identify the variable type
/// and the actual values will be provided at runtime when formatting the output.
///
/// Idk if there's a more idiomatic way to do this someone can make a PR :)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum Var {
    /// The depth of the entry in the tree.
    /// Relative to the root directory.
    Depth,
    /// The icon for the entry.
    /// Can be configured in the `indicators` section of the config.
    /// If not configured, it will use the default icon.
    /// # Note
    /// This must be a valid Unicode string, emoji or font icon.
    /// For font icons, nerd fonts are recommended.
    Icon,
    /// The name of the entry.
    /// If the name is not available, it will use the path as a fallback.
    /// (e.g., for `/`, name is not available, so use `/`).
    Name,
    /// The permissions of the file or directory.
    /// This the same as the output of `ls -l`.
    /// It is a string that contains the permissions in the format `rwxr-xr-x`.
    /// The first 3 characters are the permissions for the owner,
    /// the next 3 characters are the permissions for the group,
    /// and the last 3 characters are the permissions for others.
    /// `r` means read, `w` means write, and `x` means execute.
    Permissions,
    /// The size of the entry in bytes.
    Size,
    /// The last modified date and time of the file or directory.
    /// This can be null if the entry does not have a timestamp
    /// or if it is not applicable
    LastModified,
    /// The git status of the file or directory.
    /// This is available only if `args.path` is a git repository.
    GitStatus,
    /// The number of hard links to the entry.
    Nlink,
    /// The target of the symlink, if the entry is a symlink.
    LinkTarget,
    /// Fallback for unknown or unsupported template variables.
    /// Store the name for potential custom script parsing
    Unknown(String),
}

impl Var {
    fn from_str(s: &str) -> Self {
        match s {
            "depth" => Var::Depth,
            "icon" => Var::Icon,
            "name" => Var::Name,
            "permissions" => Var::Permissions,
            "size" => Var::Size,
            "last_modified" => Var::LastModified,
            "git_status" => Var::GitStatus,
            "nlink" => Var::Nlink,
            "link_target" => Var::LinkTarget,
            s => Var::Unknown(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarOp {
    /// No particular operation, just replace the template variable
    Replace(Var),

    /// Repeat a particular pattern for a certain count variable
    Repeat { pattern: Var, count_var: Var },

    Conditional {
        condition_var: Var,
        true_part: Var,
        false_part: Var,
    },
}

/// For a formatted string, like this "{name} - {icon}"
/// This enum acts as a token for parsing these types of string,
/// where `Part::Literal()`  is anything that does not need replacing, so
/// in this case `" - "`;
/// instead Part::Variable(v) represents the actual things to replace, so
/// in this case `"{name}"` and `"{icon}"`
///
/// Effectively, parsing the string in a vector of parts would result in:
/// ```no_run
/// [
///     Part::Variable("name"),
///     Part::Literal(" - "),
///     Part::Variable("icon")
/// ]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Literal(String),
    Op(VarOp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template(Vec<Part>, Alignment);

impl Template {
    pub fn parse<T: AsRef<str>>(template: T) -> Result<Self, PlsError> {
        let template = template.as_ref();

        if !Template::validate_brackets(&template) {
            return Err(PlsError::NoClosedBracket);
        }

        let mut parts: Vec<Part> = Vec::new();
        let mut chars = template.chars().peekable();
        let mut alignment = Alignment::default();

        // Keep track of the current literal string,
        // meaning that if a non-variable is encountered,
        // accumulate it, and when an open bracket is found, push it
        let mut literal = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &'{' {
                            // Double opening bracket - escape sequence
                            chars.next(); // consume the second '{'
                            literal.push('{'); // add single '{' to literal
                            continue;
                        }
                    }

                    // If a literal string has been accumulated,
                    // push it as a part
                    if !literal.is_empty() {
                        parts.push(Part::Literal(literal.clone()));
                        literal.clear();
                    }

                    let mut content = String::new();

                    while let Some(var_ch) = chars.next() {
                        if var_ch == '}' {
                            break;
                        }

                        content.push(var_ch);
                    }

                    // Check if an alignment character has been found
                    if let Some(last_ch) = content.chars().last() {
                        if let Some(a) = Alignment::from_char(last_ch) {
                            alignment = a;
                            content = content[..content.len() - last_ch.len_utf8()].to_string();
                        }
                    }

                    parts.push(Part::Op(Template::parse_operation(&content)));
                }
                '}' => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &'}' {
                            // Double closing bracket - escape sequence
                            chars.next(); // consume the second '}'
                            literal.push('}'); // add single '}' to literal
                            continue;
                        }
                    }
                }

                _ => {
                    literal.push(ch);
                }
            }
        }

        // Check for hanging literals
        if !literal.is_empty() {
            parts.push(Part::Literal(literal));
        }

        Ok(Template(parts, alignment))
    }

    fn validate_brackets(input: &str) -> bool {
        let mut depth = 0;
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &'{' {
                            // Skip double bracket escape sequence
                            chars.next();
                            continue;
                        }
                    }
                    depth += 1;
                }
                '}' => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &'}' {
                            // Skip double bracket escape sequence
                            chars.next();
                            continue;
                        }
                    }
                    depth -= 1;
                    if depth < 0 {
                        return false;
                    }
                }
                _ => {}
            }
        }

        depth == 0
    }

    fn parse_operation(content: &str) -> VarOp {
        let has_colon = content.contains(':');
        let has_qm = content.contains('?');

        if has_colon && !has_qm {
            let parts = content.split(':').collect::<Vec<_>>();

            if parts.len() != 2 {
                return VarOp::Replace(Var::Unknown(content.to_owned()));
            }

            let pattern = Var::from_str(&parts[0]);
            let count_var = Var::from_str(&parts[1]);

            return VarOp::Repeat { pattern, count_var };
        }

        if has_qm {
            let parts = content.split('?').collect::<Vec<_>>();

            if parts.len() != 2 {
                return VarOp::Replace(Var::Unknown(content.to_owned()));
            }

            let cond = parts[1].split(':').collect::<Vec<_>>();

            if cond.len() != 2 {
                return VarOp::Replace(Var::Unknown(co   ntent.to_owned()));
            }

            let condition_var = Var::from_str(&parts[0]);
            let true_part = Var::from_str(&cond[0]);
            let false_part = Var::from_str(&cond[1]);

            return VarOp::Conditional {
                condition_var,
                true_part,
                false_part,
            };
        }

        VarOp::Replace(Var::from_str(content))
    }

    pub fn parts(&self) -> &Vec<Part> {
        &self.0
    }

    pub fn alignment(&self) -> Alignment {
        self.1
    }

    pub fn default_templates() -> Vec<Template> {
        vec![
            Template(
                vec![Part::Op(VarOp::Repeat {
                    pattern: Var::Unknown(" ".to_owned()),
                    count_var: Var::Depth,
                })],
                Alignment::Left,
            ),
            Template(vec![Part::Op(VarOp::Replace(Var::Icon))], Alignment::Left),
            Template(vec![Part::Op(VarOp::Replace(Var::Name))], Alignment::Left),
            Template(vec![Part::Op(VarOp::Replace(Var::Size))], Alignment::Right),
        ]
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Template::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_with_var() {
        let template = "{name}";
        let result = Template::parse(template);

        assert!(result.is_ok());

        let template = result.unwrap();

        assert_eq!(template.parts(), &vec![Part::Op(VarOp::Replace(Var::Name))]);
    }

    #[test]
    fn replace_with_unknown() {
        let template = "Hello, I am {age} years old!";
        let result = Template::parse(template);

        assert!(result.is_ok());

        let template = result.unwrap();

        assert_eq!(
            template.parts(),
            &vec![
                Part::Literal("Hello, I am ".to_owned()),
                Part::Op(VarOp::Replace(Var::Unknown("age".to_owned()))),
                Part::Literal(" years old!".to_owned())
            ]
        );
    }

    #[test]
    fn replace_with_both() {
        let template = "Hello, my name is {name} and I am {age} years old!";
        let result = Template::parse(template);

        assert!(result.is_ok());

        let template = result.unwrap();

        assert_eq!(
            template.parts(),
            &vec![
                Part::Literal("Hello, my name is ".to_owned()),
                Part::Op(VarOp::Replace(Var::Name)),
                Part::Literal(" and I am ".to_owned()),
                Part::Op(VarOp::Replace(Var::Unknown("age".to_owned()))),
                Part::Literal(" years old!".to_owned())
            ]
        );
    }

    #[test]
    fn repeat_operation() {
        let template = "{abc:depth}";
        let result = Template::parse(template);

        assert!(result.is_ok());

        let template = result.unwrap();

        assert_eq!(
            template.parts(),
            &vec![Part::Op(VarOp::Repeat {
                pattern: Var::Unknown("abc".to_owned()),
                count_var: Var::Depth
            })]
        )
    }

    #[test]
    fn conditional_operation() {
        let template = "Git: {git_status?git_status:-}";
        let result = Template::parse(template);

        assert!(result.is_ok());

        let template = result.unwrap();

        assert_eq!(
            template.parts(),
            &vec![
                Part::Literal("Git: ".to_owned()),
                Part::Op(VarOp::Conditional {
                    condition_var: Var::GitStatus,
                    true_part: Var::GitStatus,
                    false_part: Var::Unknown("-".to_string())
                })
            ]
        )
    }

    #[test]
    fn empty_template() {
        let template = "";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.parts(), &vec![]);
    }

    #[test]
    fn only_literal() {
        let template = "just text with no variables";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(
            template.parts(),
            &vec![Part::Literal("just text with no variables".to_owned())]
        );
    }

    #[test]
    fn escaped_brackets() {
        let template = "Literal {{ and }}";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(
            template.parts(),
            &vec![Part::Literal("Literal { and }".to_owned())]
        );
    }

    #[test]
    fn unclosed_bracket_should_fail() {
        let template = "text with {unclosed";
        let result = Template::parse(template);
        assert!(matches!(result, Err(PlsError::NoClosedBracket)));
    }

    #[test]
    fn unmatched_closing_bracket_should_fail() {
        let template = "text with unmatched } bracket";
        let result = Template::parse(template);
        assert!(matches!(result, Err(PlsError::NoClosedBracket)));
    }

    #[test]
    fn alignment_characters_affect_only_last_var() {
        let template = "{name<} - {size^}";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(
            template.parts(),
            &vec![
                Part::Op(VarOp::Replace(Var::Name)),
                Part::Literal(" - ".to_owned()),
                Part::Op(VarOp::Replace(Var::Size))
            ]
        );
        assert_eq!(template.alignment(), Alignment::Center); // alignment set by last variable
    }

    #[test]
    fn malformed_conditional_missing_colon() {
        let template = "{git_status?git_status}";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();

        assert_eq!(
            template.parts(),
            &vec![Part::Op(VarOp::Replace(Var::Unknown(
                "git_status?git_status".to_owned()
            )))]
        );
    }

    #[test]
    fn variable_with_non_ascii_name() {
        let template = "{👽}";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(
            template.parts(),
            &vec![Part::Op(VarOp::Replace(Var::Unknown("👽".to_owned())))]
        );
    }

    #[test]
    fn multiple_escaped_sequences() {
        let template = "Escaped {{ and }} plus {name}";
        let result = Template::parse(template);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(
            template.parts(),
            &vec![
                Part::Literal("Escaped { and } plus ".to_owned()),
                Part::Op(VarOp::Replace(Var::Name))
            ]
        );
    }
}
