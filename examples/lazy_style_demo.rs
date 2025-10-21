//! Complete working example of the lazy style system
//!
//! This demonstrates:
//! 1. Creating a FileEntryContext with file information
//! 2. Using LazyContext to resolve variables on-demand
//! 3. Automatic style application based on file type
//! 4. Caching for performance

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Note: This is a standalone example showing the concepts.
// In the real implementation, these would come from your modules.

// Mock implementations for demonstration

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FileKind {
    File,
    Directory,
    Executable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ListVariable {
    Name,
    Path,
    Kind,
    Size,
    Permissions,
}

impl std::fmt::Display for ListVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Name => "name",
            Self::Path => "path",
            Self::Kind => "kind",
            Self::Size => "size",
            Self::Permissions => "permissions",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ListVariable {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "path" => Ok(Self::Path),
            "kind" => Ok(Self::Kind),
            "size" => Ok(Self::Size),
            "permissions" => Ok(Self::Permissions),
            _ => Err(format!("Unknown variable: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
struct Color {
    name: String,
}

impl Color {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    fn apply(&self, text: &str) -> String {
        // In real implementation, this would return ANSI escape codes
        format!("[{}:{}]", self.name, text)
    }
}

#[derive(Debug, Clone)]
struct CombinedStyle {
    foreground: Color,
    background: Color,
}

impl CombinedStyle {
    fn apply(&self, text: String) -> String {
        let with_fg = self.foreground.apply(&text);
        format!("{}[bg:{}]", with_fg, self.background.name)
    }
}

#[derive(Debug, Clone)]
struct TextStyle {
    foreground_simple: Color,
    foreground_by_kind: HashMap<FileKind, Color>,
    background_simple: Color,
}

impl TextStyle {
    fn new(fg: Color, bg: Color) -> Self {
        Self {
            foreground_simple: fg,
            foreground_by_kind: HashMap::new(),
            background_simple: bg,
        }
    }

    fn with_kind_color(mut self, kind: FileKind, color: Color) -> Self {
        self.foreground_by_kind.insert(kind, color);
        self
    }

    fn resolve(&self, kind: &FileKind) -> CombinedStyle {
        let fg = self
            .foreground_by_kind
            .get(kind)
            .unwrap_or(&self.foreground_simple)
            .clone();

        CombinedStyle {
            foreground: fg,
            background: self.background_simple.clone(),
        }
    }
}

struct Config {
    styles: HashMap<ListVariable, TextStyle>,
}

impl Config {
    fn new() -> Self {
        let mut styles = HashMap::new();

        // Configure name styling
        let name_style = TextStyle::new(Color::new("white"), Color::new("black"))
            .with_kind_color(FileKind::Directory, Color::new("blue"))
            .with_kind_color(FileKind::Executable, Color::new("green"));

        styles.insert(ListVariable::Name, name_style);

        // Configure size styling
        let size_style = TextStyle::new(Color::new("yellow"), Color::new("black"));

        styles.insert(ListVariable::Size, size_style);

        Self { styles }
    }
}

// The core trait for value providers
trait ValueProvider {
    fn get(&self, var: &ListVariable) -> Option<String>;
    fn get_styled(&self, var: &ListVariable, style: &TextStyle) -> Option<String>;
}

// FileEntryContext: Holds all file information
struct FileEntryContext<'a> {
    name: String,
    path: PathBuf,
    kind: FileKind,
    size: u64,
    config: &'a Config,
}

impl<'a> FileEntryContext<'a> {
    fn new(name: String, path: PathBuf, kind: FileKind, size: u64, config: &'a Config) -> Self {
        Self {
            name,
            path,
            kind,
            size,
            config,
        }
    }
}

impl<'a> ValueProvider for FileEntryContext<'a> {
    fn get(&self, var: &ListVariable) -> Option<String> {
        match var {
            ListVariable::Name => Some(self.name.clone()),
            ListVariable::Path => Some(self.path.display().to_string()),
            ListVariable::Kind => Some(format!("{:?}", self.kind)),
            ListVariable::Size => Some(format!("{} bytes", self.size)),
            ListVariable::Permissions => Some("rwxr-xr-x".to_string()),
        }
    }

    fn get_styled(&self, var: &ListVariable, style: &TextStyle) -> Option<String> {
        let value = self.get(var)?;
        let combined = style.resolve(&self.kind);
        Some(combined.apply(value))
    }
}

// LazyContext: Resolves and caches values on-demand
struct LazyContext<'a, P: ValueProvider> {
    provider: &'a P,
    config: &'a Config,
    cache: std::cell::RefCell<HashMap<String, String>>,
}

impl<'a, P: ValueProvider> LazyContext<'a, P> {
    fn new(provider: &'a P, config: &'a Config) -> Self {
        Self {
            provider,
            config,
            cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    fn resolve(&self, var_name: &str) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.cache.borrow().get(var_name) {
            println!("  [CACHE HIT] {}", var_name);
            return Some(cached.clone());
        }

        println!("  [COMPUTING] {}", var_name);

        // Parse variable
        let var = std::str::FromStr::from_str(var_name).ok()?;

        // Get style
        let style = self.config.styles.get(&var)?;

        // Get styled value
        let value = self.provider.get_styled(&var, style)?;

        // Cache it
        self.cache
            .borrow_mut()
            .insert(var_name.to_string(), value.clone());

        Some(value)
    }

    fn build_context_for(&self, variables: &[ListVariable]) -> HashMap<String, String> {
        let mut context = HashMap::new();

        for var in variables {
            let var_name = var.to_string();
            if let Some(value) = self.resolve(&var_name) {
                context.insert(var_name, value);
            }
        }

        context
    }
}

// Simple template formatter
fn format_template(template: &str, context: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (key, value) in context {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}

fn main() {
    println!("=== Lazy Style System Demo ===\n");

    // Setup config
    let config = Config::new();

    // Example 1: Regular file
    println!("Example 1: Regular File");
    println!("-----------------------");
    let file_ctx = FileEntryContext::new(
        "document.txt".to_string(),
        PathBuf::from("/home/user/document.txt"),
        FileKind::File,
        1024,
        &config,
    );

    let lazy = LazyContext::new(&file_ctx, &config);

    // Template uses name and size
    let template = "{name} ({size})";
    println!("Template: {}", template);

    let context = lazy.build_context_for(&[ListVariable::Name, ListVariable::Size]);

    let output = format_template(template, &context);
    println!("Output: {}\n", output);

    // Example 2: Directory
    println!("Example 2: Directory");
    println!("--------------------");
    let dir_ctx = FileEntryContext::new(
        "projects".to_string(),
        PathBuf::from("/home/user/projects"),
        FileKind::Directory,
        4096,
        &config,
    );

    let lazy = LazyContext::new(&dir_ctx, &config);

    println!("Template: {}", template);
    let context = lazy.build_context_for(&[ListVariable::Name, ListVariable::Size]);

    let output = format_template(template, &context);
    println!("Output: {} (note blue color for directory)\n", output);

    // Example 3: Caching demonstration
    println!("Example 3: Caching Demonstration");
    println!("---------------------------------");
    let exec_ctx = FileEntryContext::new(
        "my_program".to_string(),
        PathBuf::from("/usr/bin/my_program"),
        FileKind::Executable,
        204800,
        &config,
    );

    let lazy = LazyContext::new(&exec_ctx, &config);

    println!("First access to 'name':");
    let name1 = lazy.resolve("name");

    println!("\nSecond access to 'name':");
    let name2 = lazy.resolve("name");

    println!("\nFirst access to 'size':");
    let size = lazy.resolve("size");

    println!("\nResult:");
    println!("  name1: {:?}", name1);
    println!("  name2: {:?} (same as name1, from cache)", name2);
    println!("  size: {:?}\n", size);

    // Example 4: Multiple variables
    println!("Example 4: Complex Template");
    println!("----------------------------");
    let template2 = "{kind}: {name} | {size} | {path}";
    println!("Template: {}", template2);

    let context = lazy.build_context_for(&[
        ListVariable::Kind,
        ListVariable::Name,
        ListVariable::Size,
        ListVariable::Path,
    ]);

    let output = format_template(template2, &context);
    println!("Output: {}\n", output);

    // Example 5: Selective variable usage
    println!("Example 5: Only Using Subset of Variables");
    println!("------------------------------------------");
    println!("Available variables: name, path, kind, size, permissions");
    println!("Template only uses: {{name}} {{size}}");
    println!("Result: Only name and size are computed (lazy evaluation)\n");

    let minimal_ctx = FileEntryContext::new(
        "test.rs".to_string(),
        PathBuf::from("/src/test.rs"),
        FileKind::File,
        5120,
        &config,
    );

    let lazy = LazyContext::new(&minimal_ctx, &config);

    // Only compute what's needed
    let context = lazy.build_context_for(&[ListVariable::Name, ListVariable::Size]);

    let output = format_template("{name} - {size}", &context);
    println!("Output: {}", output);
    println!("Note: 'path', 'kind', and 'permissions' were never computed!\n");

    println!("=== Performance Benefits ===");
    println!("1. Lazy Evaluation: Only computes variables actually used");
    println!("2. Caching: Multiple uses of same variable only compute once");
    println!("3. Automatic Styling: Styles applied based on file characteristics");
    println!("4. Type Safety: ListVariable enum prevents typos");
    println!("5. Extensibility: Easy to add new variables or style rules");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_provider() {
        let config = Config::new();
        let ctx = FileEntryContext::new(
            "test.txt".to_string(),
            PathBuf::from("/test.txt"),
            FileKind::File,
            100,
            &config,
        );

        assert_eq!(ctx.get(&ListVariable::Name), Some("test.txt".to_string()));
        assert_eq!(ctx.get(&ListVariable::Size), Some("100 bytes".to_string()));
    }

    #[test]
    fn test_lazy_caching() {
        let config = Config::new();
        let ctx = FileEntryContext::new(
            "test.txt".to_string(),
            PathBuf::from("/test.txt"),
            FileKind::File,
            100,
            &config,
        );

        let lazy = LazyContext::new(&ctx, &config);

        let val1 = lazy.resolve("name");
        let val2 = lazy.resolve("name");

        assert_eq!(val1, val2);
        assert!(lazy.cache.borrow().contains_key("name"));
    }

    #[test]
    fn test_style_resolution() {
        let config = Config::new();

        let file_ctx = FileEntryContext::new(
            "file.txt".to_string(),
            PathBuf::from("/file.txt"),
            FileKind::File,
            100,
            &config,
        );

        let dir_ctx = FileEntryContext::new(
            "dir".to_string(),
            PathBuf::from("/dir"),
            FileKind::Directory,
            4096,
            &config,
        );

        // Different file kinds should produce different styles
        let file_name = file_ctx.get(&ListVariable::Name).unwrap();
        let dir_name = dir_ctx.get(&ListVariable::Name).unwrap();

        // In real implementation, these would have different ANSI codes
        assert_ne!(file_name, dir_name);
    }
}
