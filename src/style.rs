use serde::Deserialize;

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
                    "bright black" | "brightblack" | "gray" | "grey" => 60,
                    "bright red" | "brightred" => 61,
                    "bright green" | "brightgreen" => 62,
                    "bright yellow" | "brightyellow" => 63,
                    "bright blue" | "brightblue" => 64,
                    "bright magenta" | "brightmagenta" => 65,
                    "bright cyan" | "brightcyan" => 66,
                    "bright white" | "brightwhite" => 67,

                    _ => return String::new(),
                };

                let base = if code >= 60 {
                    if prefix == 48 { 100 - 60 } else { 90 - 60 }
                } else {
                    if prefix == 48 { 40 } else { 30 }
                };

                format!("\x1b[{}m", base + code)
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
#[derive(Debug, Clone)]
pub enum TextStyle {
    Normal,
    Bold,
    Italic,
    Underline,
    Dim,
    Strikethrough,
    Blink,
    Inverse,
    Conceal,
    CrossedOut,
    DoubleUnderline,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl TextStyle {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Bold => "\x1b[1m",
            Self::Italic => "\x1b[3m",
            Self::Underline => "\x1b[4m",
            Self::Dim => "\x1b[2m",
            Self::Strikethrough => "\x1b[9m",
            Self::Blink => "\x1b[5m",
            Self::Inverse => "\x1b[7m",
            Self::Conceal => "\x1b[8m",
            Self::CrossedOut => "\x1b[9m",
            Self::DoubleUnderline => "\x1b[21m",
        }
    }
}

impl<'de> Deserialize<'de> for TextStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "normal" => Ok(TextStyle::Normal),
            "bold" => Ok(TextStyle::Bold),
            "italic" => Ok(TextStyle::Italic),
            "underline" => Ok(TextStyle::Underline),
            "dim" => Ok(TextStyle::Dim),
            "strikethrough" => Ok(TextStyle::Strikethrough),
            "blink" => Ok(TextStyle::Blink),
            "inverse" => Ok(TextStyle::Inverse),
            "conceal" => Ok(TextStyle::Conceal),
            "crossedout" | "crossed_out" => Ok(TextStyle::CrossedOut),
            "doubleunderline" | "double_underline" => Ok(TextStyle::DoubleUnderline),
            _ => Err(serde::de::Error::custom(format!(
                "invalid text style: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

impl<'de> Deserialize<'de> for Op {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "=" | "eq" => Ok(Op::Equal),
            ">" | "gt" => Ok(Op::Greater),
            "<" | "lt" => Ok(Op::Less),
            ">=" | "gte" => Ok(Op::GreaterEqual),
            "<=" | "lte" => Ok(Op::LessEqual),
            _ => Err(serde::de::Error::custom(format!("invalid operator: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConditionalRule {
    pub variable: String,
    pub op: Op,
    pub value: String,
    pub style: VariableStyle,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct VariableStyle {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub text: Option<Vec<TextStyle>>,
}

impl VariableStyle {
    pub fn apply<S: AsRef<str>>(&self, s: S) -> String {
        let mut out = String::new();
        let mut applied = false;

        if let Some(fg) = &self.foreground {
            let code = fg.to_ansi_foreground();
            if !code.is_empty() {
                out.push_str(&code);
                applied = true;
            }
        }

        if let Some(bg) = &self.background {
            let code = bg.to_ansi_background();
            if !code.is_empty() {
                out.push_str(&code);
                applied = true;
            }
        }

        if let Some(text_styles) = &self.text {
            for style in text_styles {
                let code = style.to_ansi();
                if !code.is_empty() {
                    out.push_str(code);
                    applied = true;
                }
            }
        }

        out.push_str(s.as_ref());

        if applied {
            out.push_str("\x1b[0m");
        }

        out
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldStyle {
    #[serde(default)]
    pub default: Option<VariableStyle>,

    #[serde(default)]
    pub conditions: Vec<ConditionalRule>,
}
