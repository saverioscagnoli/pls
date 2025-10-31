# pls

A modern, fast, and highly customizable `ls` successor.

## Installation

#### Manual Installation

```bash
# Clone and build
git clone https://github.com/saverioscagnoli/pls
cd pls
cargo build --release
sudo cp ./target/release/pls /usr/local/bin
```

## Usage

### Basic Listing

```bash
# List current directory
pls

# List specific directory
pls /path/to/dir

# Show hidden files
pls -a

# Pad names so the .dot files look aligned
pls -ap

# Control depth
pls -d 2

# Follow symlinks
pls -f
```

### Find Command

```bash
# Search for files matching a pattern
pls find *.rs

# Search in specific directory
pls find config /etc

# Show all files (including hidden)
pls find test . -a

# Exact name match
pls find main.rs -e

# Limit search depth
pls find *.json -d 3

# Time the search
pls find *.txt -t
```

## Configuration

Configuration file is located at `~/.config/pls/config.json` (created automatically on first run).

### Format Variables

Available template variables for the `format` field:

- `{name}` - File/directory name
- `{path}` - Full path
- `{extension}` - File extension
- `{kind}` - File type (file, directory, executable, etc.)
- `{icon}` - Conditional icon
- `{depth}` - Directory depth
- `{size}` - File size (formatted)
- `{permissions}` - Unix permissions (rwxr-xr-x)
- `{created}` - Creation timestamp
- `{modified}` - Modification timestamp
- `{accessed}` - Access timestamp
- `{owner}` - File owner
- `{group}` - File group
- `{nlink}` - Number of hard links

### Alignment

Use alignment modifiers in templates:

- `{field}` - Left aligned (default)
- `{field>}` - Right aligned
- `{field^}` - Center aligned

### Example Configuration

```json
{
  "$schema": "./config.schema.json",
  "ls": {
    "format": ["{icon} {name}", "{permissions}", "{size>}", "{modified}"],
    "padding": 2,
    "headers": ["Name", "Perms", "Size", "Modified"],
    "size_unit": "auto",
    "styles": {
      "name": {
        "conditions": [
          {
            "variable": "kind",
            "op": "eq",
            "value": "directory",
            "result": {
              "foreground": "blue",
              "text": ["bold"]
            }
          }
        ]
      }
    }
  }
}
```

### Conditional Styling

Apply styles based on file properties:

```json
{
  "styles": {
    "name": {
      "default": {
        "foreground": "white"
      },
      "conditions": [
        {
          "variable": "kind",
          "op": "eq",
          "value": "executable",
          "result": {
            "foreground": "green",
            "text": ["bold"]
          }
        },
        {
          "variable": "size",
          "op": "gt",
          "value": "1000000",
          "result": {
            "foreground": "red"
          }
        }
      ]
    }
  }
}
```

### Supported Operators

- `==` or `eq` - Equal
- `!=` or `ne` - Not equal
- `>` or `gt` - Greater than
- `<` or `lt` - Less than
- `>=` or `gte` - Greater than or equal
- `<=` or `lte` - Less than or equal

### Color Options

Colors can be specified in multiple formats:

- **Named**: `"red"`, `"blue"`, `"green"`, `"yellow"`, `"magenta"`, `"cyan"`, `"white"`, `"black"`
- **Bright**: `"bright red"`, `"bright blue"`, etc.
- **RGB**: `[255, 0, 0]`
- **Hex**: `"#FF5733"`
- **ANSI**: `196` (0-255)

### Text Styles

- `normal`
- `bold`
- `italic`
- `underline`
- `dim`
- `strikethrough`
- `blink`
- `inverse`
- `conceal`
- `crossed out`
- `double underline`

## File Kinds

- `file` - Regular file
- `directory` - Directory
- `executable` - Executable file
- `symlink_file` - Symbolic link to a file
- `symlink_directory` - Symbolic link to a directory
- `broken_symlink` - Broken symbolic link

## License

MIT License (c) Saverio Scagnoli

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
