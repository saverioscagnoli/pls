use crate::args::ListArgs;
use crate::config::{ListConfig, ListVariable};
use crate::err::PlsError;
use figura::Template;
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

pub fn execute(args: &ListArgs, config: &ListConfig) -> Result<(), PlsError> {
    let entries = std::fs::read_dir(&args.path)?;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let used_variables = config.list_variables();
    let mut context = HashMap::new();

    for entry in entries {
        let Ok(entry) = entry else {
            writeln!(handle, "Unreadable entry")?;
            continue;
        };

        // If -a is false, skip hidden files
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            writeln!(handle, "Skipping entry with faulty name")?;
            continue;
        };

        if !args.all && name.starts_with('.') {
            continue;
        }

        let metadata = if used_variables.iter().any(|v| {
            matches!(
                v,
                ListVariable::Size
                    | ListVariable::Permissions
                    | ListVariable::Created
                    | ListVariable::Modified
                    | ListVariable::Accessed
            )
        }) {
            entry.metadata().ok()
        } else {
            None
        };

        for var in &used_variables {
            match var {
                ListVariable::Name => {
                    let mut owned = name.to_string();

                    // If -a is true, add a whitespace at the start
                    // So they are aligned
                    if args.all && !name.starts_with('.') {
                        owned.insert(0, ' ');
                    }

                    context.insert("name", figura::Value::String(owned));
                }

                ListVariable::Path => {
                    context.insert(
                        "path",
                        figura::Value::String(entry.path().to_string_lossy().to_string()),
                    );
                }

                ListVariable::Size => {
                    if let Some(meta) = &metadata {
                        context.insert("size", figura::Value::Int(meta.len() as i64));
                    }
                }

                ListVariable::Permissions => {
                    if let Some(meta) = &metadata {
                        context.insert(
                            "permissions",
                            figura::Value::String(format!("{:o}", meta.permissions().mode())),
                        );
                    }
                }

                ListVariable::Created => {
                    if let Some(meta) = &metadata {
                        if let Ok(time) = meta.created() {
                            if let Ok(datetime) = time.duration_since(std::time::UNIX_EPOCH) {
                                context.insert(
                                    "created",
                                    figura::Value::Int(datetime.as_secs() as i64),
                                );
                            }
                        }
                    }
                }

                ListVariable::Modified => {
                    if let Some(meta) = &metadata {
                        if let Ok(time) = meta.modified() {
                            if let Ok(datetime) = time.duration_since(std::time::UNIX_EPOCH) {
                                context.insert(
                                    "modified",
                                    figura::Value::Int(datetime.as_secs() as i64),
                                );
                            }
                        }
                    }
                }

                ListVariable::Accessed => {
                    if let Some(meta) = &metadata {
                        if let Ok(time) = meta.accessed() {
                            if let Ok(datetime) = time.duration_since(std::time::UNIX_EPOCH) {
                                context.insert(
                                    "accessed",
                                    figura::Value::Int(datetime.as_secs() as i64),
                                );
                            }
                        }
                    }
                }
            }
        }

        let mut result = Vec::new();

        for t in &config.format {
            let template = match Template::<'{', '}'>::parse(&t) {
                Ok(t) => t,
                Err(e) => {
                    writeln!(handle, "{}", e)?;
                    continue;
                }
            };

            let formatted = match template.format(&context) {
                Ok(s) => s,
                Err(e) => {
                    writeln!(handle, "{}", e)?;
                    continue;
                }
            };

            result.push(formatted);
        }

        writeln!(handle, "{}", result.join(" "))?;

        context.clear();
    }

    Ok(())
}
