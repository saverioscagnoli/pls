# Color System Implementation

## Overview

The per-variable color system allows users to colorize different parts of the file listing output based on file types, extensions, or apply uniform colors to specific variables.

## Architecture

### Configuration (config.rs)

#### `VariableColorConfig` Enum

```rust
pub enum VariableColorConfig {
    Simple(Color),
    Complex {
        kinds: HashMap<FileKind, Color>,
        extensions: HashMap<String, Color>,
        default: Color,
    },
}
```

- **Simple**: Applies a single color to all instances of a variable
- **Complex**: Applies different colors based on file kind or extension with a fallback

#### Color Resolution

The `resolve_color()` method determines which color to use with this priority:

1. **Extension match** (highest priority) - e.g., `.rs`, `.json`
2. **File kind match** - e.g., directory, executable
3. **Default color** (fallback)

### Implementation (list.rs)

#### Key Design Decision: Pre-Coloring

**Problem**: Initially, the implementation colorized values AFTER template formatting by doing string replacement. This caused issues when the same substring appeared multiple times in a row.

**Example Bug**:
```
nlink: 2
size: 2 MB
```
Colorizing "2" for nlink would also colorize the "2" in "2 MB".

**Solution**: Colors are applied to context values BEFORE template formatting. Each variable gets its color applied exactly once at the source, preventing any collision issues.

#### Implementation Flow

```rust
// 1. Create a closure that applies color based on variable config
let apply_color = |var: &ListVariable, value: String| -> String {
    if config.colors.enabled {
        if let Some(var_config) = config.colors.variables.get(var) {
            let color = var_config.resolve_color(kind, ext);
            return color.colorize(&value);
        }
    }
    value
};

// 2. Apply color when inserting into context
let value = name.to_string_lossy().to_string();
let colored = apply_color(&ListVariable::Name, value);
context.insert("name", Value::String(colored));

// 3. Template formatting happens with already-colored values
let formatted = template.format(&context)?;
```

## Configuration Examples

### Simple Coloring

Apply one color to all instances of a variable:

```toml
[ls.colors.variables.size]
"red"
```

```json
{
  "variables": {
    "size": "red"
  }
}
```

### Complex Coloring

Different colors based on file characteristics:

```toml
[ls.colors.variables.name]
default = "white"

[ls.colors.variables.name.kinds]
directory = "bright_blue"
executable = "bright_green"
file = "cyan"

[ls.colors.variables.name.extensions]
rs = "bright_yellow"
json = "yellow"
md = "bright_cyan"
```

```json
{
  "variables": {
    "name": {
      "default": "white",
      "kinds": {
        "directory": "bright_blue",
        "executable": "bright_green",
        "file": "cyan"
      },
      "extensions": {
        "rs": "bright_yellow",
        "json": "yellow",
        "md": "bright_cyan"
      }
    }
  }
}
```

### Color Formats

The system supports multiple color formats:

1. **Named colors**: `"red"`, `"bright_blue"`, `"cyan"`
2. **RGB tuples**: `[255, 165, 0]`
3. **Hex strings**: `"#FF5733"`
4. **ANSI 256 codes**: `214`

## Supported Variables

All list variables support colorization:

- `name` - File/directory name
- `path` - Full path
- `kind` - File type (file, directory, etc.)
- `size` - File size
- `depth` - Directory depth
- `icon` - Icon representation
- `permissions` - File permissions
- `created` - Creation timestamp
- `modified` - Modification timestamp
- `accessed` - Access timestamp
- `owner` - File owner
- `group` - File group
- `nlink` - Number of hard links

## File Kinds

Available file kinds for color matching:

- `file` - Regular file
- `directory` - Directory
- `symlink_file` - Symbolic link to a file
- `symlink_directory` - Symbolic link to a directory
- `executable` - Executable file

## Best Practices

1. **Use extension colors for specific file types**: More specific than kind-based coloring
2. **Set sensible defaults**: Always provide a default color in complex configs
3. **Consider terminal compatibility**: Named colors are most portable
4. **Test with your theme**: Colors may look different in light/dark terminals
5. **Don't overuse colors**: Too many colors can reduce readability

## Performance

- Colors are resolved once per file entry
- No regex matching or complex string manipulation
- Minimal overhead even for large directory listings
- Template formatting happens after color application, preventing duplicate work