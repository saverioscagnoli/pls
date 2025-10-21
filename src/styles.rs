use crate::{commands::list::FileKind, util};
use clap::builder::Str;
use serde::Deserialize;
use std::collections::HashMap;

/// Enum representing a color in different forms.
///
/// Note: to represent a non-ansi color, you must use
/// a terminal emulator that can handle truecolor
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Named(String),   // "red" "blue" "green"
    RGB(u8, u8, u8), // [255, 0, 0]
    Hex(String),     // #FF5733
    Ansi(u8),        // ANSI 256 color code
}

impl Default for Color {
    fn default() -> Self {
        Self::Named("white".to_string())
    }
}

impl Color {
    pub fn to_ansi_foreground(&self) -> String {
        self.to_ansi_with_prefix(38)
    }

    pub fn to_ansi_background(&self) -> String {
        self.to_ansi_with_prefix(48)
    }

    fn to_ansi_with_prefix(&self, prefix: u8) -> String {
        match self {
            Self::Named(name) => {
                let code = match name.to_lowercase().as_str() {
                    "black" => 0,
                    "red" => 1,
                    "green" => 2,
                    "yellow" => 3,
                    "blue" => 4,
                    "magenta" => 5,
                    "cyan" => 6,
                    "white" => 7,
                    _ => return String::new(),
                };
                format!("\x1b[{}m", 30 + code + if prefix == 48 { 10 } else { 0 })
            }
            Self::RGB(r, g, b) => format!("\x1b[{};2;{};{};{}m", prefix, r, g, b),
            Self::Hex(hex) => {
                let hex = hex.trim_start_matches('#');
                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return format!("\x1b[{};2;{};{};{}m", prefix, r, g, b);
                    }
                }
                String::new()
            }
            Self::Ansi(code) => format!("\x1b[{};5;{}m", prefix, code),
        }
    }
}

/// Enum representing font styling.
#[derive(Debug, Clone, Deserialize)]
pub enum Text {
    Normal,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Blink,
    Inverse,
    Conceal,
    CrossedOut,
    DoubleUnderline,
}

impl Default for Text {
    fn default() -> Self {
        Self::Normal
    }
}

impl Text {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Bold => "\x1b[1m",
            Self::Italic => "\x1b[3m",
            Self::Underline => "\x1b[4m",
            Self::Strikethrough => "\x1b[9m",
            Self::Blink => "\x1b[5m",
            Self::Inverse => "\x1b[7m",
            Self::Conceal => "\x1b[8m",
            Self::CrossedOut => "\x1b[9m",
            Self::DoubleUnderline => "\x1b[21m",
        }
    }
}

/// Enum representing text configuration per-variable
/// so for example you could wanted to color and bold the "name" variable based
/// on file type or extension, you can set it via this object
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VariableStyle<T: Default> {
    /// This will apply the said style to all instances
    /// of the variable across the listing.
    Simple(T),

    Complex {
        /// Style applied based on the entry kind.
        #[serde(default)]
        kind: HashMap<FileKind, T>,

        /// Style applied based on the file extension.
        #[serde(default)]
        extensions: HashMap<String, T>,

        /// The fallback style if no other rule matches.'
        /// For example, the default color is white and the default text style is normal
        #[serde(default)]
        default: T,
    },
}

impl<T: Default> Default for VariableStyle<T> {
    fn default() -> Self {
        Self::Simple(T::default())
    }
}

/// Configuration object used to style a variable
/// You can set foreground color, background color and text style (bold, italic, etc.)
#[derive(Debug, Clone, Deserialize)]
pub struct TextStyle {
    #[serde(default)]
    pub foreground: VariableStyle<Color>,

    #[serde(default)]
    pub background: VariableStyle<Color>,

    #[serde(default)]
    pub text: VariableStyle<Text>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { foreground: BV }
    }
}

/// This struct is used to actually combine
/// styles into a string, instead, TextStyle, having
/// the same structure which is used to represent the config
#[derive(Debug, Clone)]
pub struct CombinedStyle {
    pub foreground: Color,
    pub background: Color,
    pub text: Text,
}

impl Default for CombinedStyle {
    fn default() -> Self {
        Self {
            foreground: Color::default(),
            background: Color::default(),
            text: Text::default(),
        }
    }
}

impl CombinedStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn text(mut self, text: Text) -> Self {
        self.text = text;
        self
    }

    pub fn apply<S: AsRef<str>>(&self, val: S) -> String {
        format!(
            "{}{}{}{}{}",
            self.background.to_ansi_background(), // background color
            self.foreground.to_ansi_foreground(), // foreground color
            self.text.to_ansi(),                  // text style
            val.as_ref(),
            util::reset_style()
        )
    }
}
