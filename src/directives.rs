use figura::{DefaultParser, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FontStyle {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Dim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    // Standard 8 colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    // Bright/High-intensity variants
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    // Extended 256-color palette (some common ones)
    Orange,
    Pink,
    Purple,
    Brown,
    Gray,
    DarkGray,
    LightGray,
    DarkRed,
    DarkGreen,
    DarkBlue,
    DarkYellow,
    DarkMagenta,
    DarkCyan,
}

impl Color {
    fn to_ansi_fg(&self) -> String {
        match self {
            // Standard 8 colors (30-37)
            Color::Black => "30".to_string(),
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Yellow => "33".to_string(),
            Color::Blue => "34".to_string(),
            Color::Magenta => "35".to_string(),
            Color::Cyan => "36".to_string(),
            Color::White => "37".to_string(),
            // Bright colors (90-97)
            Color::BrightBlack => "90".to_string(),
            Color::BrightRed => "91".to_string(),
            Color::BrightGreen => "92".to_string(),
            Color::BrightYellow => "93".to_string(),
            Color::BrightBlue => "94".to_string(),
            Color::BrightMagenta => "95".to_string(),
            Color::BrightCyan => "96".to_string(),
            Color::BrightWhite => "97".to_string(),
            // Extended colors using 256-color palette
            Color::Orange => "38;5;208".to_string(),
            Color::Pink => "38;5;205".to_string(),
            Color::Purple => "38;5;129".to_string(),
            Color::Brown => "38;5;130".to_string(),
            Color::Gray => "38;5;244".to_string(),
            Color::DarkGray => "38;5;240".to_string(),
            Color::LightGray => "38;5;252".to_string(),
            Color::DarkRed => "38;5;88".to_string(),
            Color::DarkGreen => "38;5;22".to_string(),
            Color::DarkBlue => "38;5;18".to_string(),
            Color::DarkYellow => "38;5;136".to_string(),
            Color::DarkMagenta => "38;5;90".to_string(),
            Color::DarkCyan => "38;5;30".to_string(),
        }
    }

    fn to_ansi_bg(&self) -> String {
        match self {
            // Standard 8 colors (40-47)
            Color::Black => "40".to_string(),
            Color::Red => "41".to_string(),
            Color::Green => "42".to_string(),
            Color::Yellow => "43".to_string(),
            Color::Blue => "44".to_string(),
            Color::Magenta => "45".to_string(),
            Color::Cyan => "46".to_string(),
            Color::White => "47".to_string(),
            // Bright colors (100-107)
            Color::BrightBlack => "100".to_string(),
            Color::BrightRed => "101".to_string(),
            Color::BrightGreen => "102".to_string(),
            Color::BrightYellow => "103".to_string(),
            Color::BrightBlue => "104".to_string(),
            Color::BrightMagenta => "105".to_string(),
            Color::BrightCyan => "106".to_string(),
            Color::BrightWhite => "107".to_string(),
            // Extended colors using 256-color palette
            Color::Orange => "48;5;208".to_string(),
            Color::Pink => "48;5;205".to_string(),
            Color::Purple => "48;5;129".to_string(),
            Color::Brown => "48;5;130".to_string(),
            Color::Gray => "48;5;244".to_string(),
            Color::DarkGray => "48;5;240".to_string(),
            Color::LightGray => "48;5;252".to_string(),
            Color::DarkRed => "48;5;88".to_string(),
            Color::DarkGreen => "48;5;22".to_string(),
            Color::DarkBlue => "48;5;18".to_string(),
            Color::DarkYellow => "48;5;136".to_string(),
            Color::DarkMagenta => "48;5;90".to_string(),
            Color::DarkCyan => "48;5;30".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct StyleDirective {
    value: String,
    styles: Vec<FontStyle>,
    background_color: Option<Color>,
    foreground_color: Option<Color>,
}

impl figura::Directive for StyleDirective {
    fn execute(&self, ctx: &figura::Context) -> Result<String, figura::TemplateError> {
        let mut result = String::new();

        // Start with ANSI escape sequence
        result.push_str("\x1b[");

        let mut codes = Vec::new();

        // Add font styles
        for style in &self.styles {
            match style {
                FontStyle::Bold => codes.push("1".to_string()),
                FontStyle::Italic => codes.push("3".to_string()),
                FontStyle::Underline => codes.push("4".to_string()),
                FontStyle::Strikethrough => codes.push("9".to_string()),
                FontStyle::Dim => codes.push("2".to_string()),
            }
        }

        // Add foreground color
        if let Some(fg_color) = &self.foreground_color {
            codes.push(fg_color.to_ansi_fg());
        }

        // Add background color
        if let Some(bg_color) = &self.background_color {
            codes.push(bg_color.to_ansi_bg());
        }

        // Join all codes with semicolons
        if !codes.is_empty() {
            result.push_str(&codes.join(";"));
        }

        // Close the ANSI sequence
        result.push('m');

        match ctx.get(self.value.as_str()) {
            Some(value) => result.push_str(&value.to_string()),
            None => result.push_str(&self.value),
        }

        // Reset all styles at the end
        result.push_str("\x1b[0m");

        Ok(result)
    }
}

pub struct CustomParser;

impl figura::Parser for CustomParser {
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn figura::Directive>> {
        match tokens {
            [
                Token::Slice(text),
                Token::Symbol('$'),
                Token::Symbol('['),
                style_tokens @ ..,
                Token::Symbol(']'),
            ] => {
                let mut styles = Vec::new();
                let mut foreground_color = None;
                let mut background_color = None;

                // Parse style tokens
                let mut i = 0;
                while i < style_tokens.len() {
                    match &style_tokens[i] {
                        Token::Slice(style_name) => {
                            match style_name.as_str() {
                                "bold" => styles.push(FontStyle::Bold),
                                "italic" => styles.push(FontStyle::Italic),
                                "underline" => styles.push(FontStyle::Underline),
                                "strikethrough" => styles.push(FontStyle::Strikethrough),
                                "dim" => styles.push(FontStyle::Dim),
                                "fg" | "foreground" => {
                                    // Look for color after fg
                                    if i + 1 < style_tokens.len() {
                                        if let Token::Slice(color_name) = &style_tokens[i + 1] {
                                            foreground_color = parse_color(color_name);
                                            i += 1; // Skip the color token
                                        }
                                    }
                                }
                                "bg" | "background" => {
                                    // Look for color after bg
                                    if i + 1 < style_tokens.len() {
                                        if let Token::Slice(color_name) = &style_tokens[i + 1] {
                                            background_color = parse_color(color_name);
                                            i += 1; // Skip the color token
                                        }
                                    }
                                }
                                _ => {
                                    // Try to parse as a direct color
                                    if let Some(color) = parse_color(style_name) {
                                        if foreground_color.is_none() {
                                            foreground_color = Some(color);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }

                Some(Box::new(StyleDirective {
                    value: text.clone(),
                    styles,
                    background_color,
                    foreground_color,
                }))
            }

            _ => DefaultParser::parse(tokens, content),
        }
    }
}

fn parse_color(color_name: &str) -> Option<Color> {
    match color_name.to_lowercase().as_str() {
        // Standard colors
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        // Bright colors
        "bright_black" | "brightblack" => Some(Color::BrightBlack),
        "bright_red" | "brightred" => Some(Color::BrightRed),
        "bright_green" | "brightgreen" => Some(Color::BrightGreen),
        "bright_yellow" | "brightyellow" => Some(Color::BrightYellow),
        "bright_blue" | "brightblue" => Some(Color::BrightBlue),
        "bright_magenta" | "brightmagenta" => Some(Color::BrightMagenta),
        "bright_cyan" | "brightcyan" => Some(Color::BrightCyan),
        "bright_white" | "brightwhite" => Some(Color::BrightWhite),
        // Extended colors
        "orange" => Some(Color::Orange),
        "pink" => Some(Color::Pink),
        "purple" => Some(Color::Purple),
        "brown" => Some(Color::Brown),
        "gray" | "grey" => Some(Color::Gray),
        "dark_gray" | "darkgray" | "dark_grey" | "darkgrey" => Some(Color::DarkGray),
        "light_gray" | "lightgray" | "light_grey" | "lightgrey" => Some(Color::LightGray),
        "dark_red" | "darkred" => Some(Color::DarkRed),
        "dark_green" | "darkgreen" => Some(Color::DarkGreen),
        "dark_blue" | "darkblue" => Some(Color::DarkBlue),
        "dark_yellow" | "darkyellow" => Some(Color::DarkYellow),
        "dark_magenta" | "darkmagenta" => Some(Color::DarkMagenta),
        "dark_cyan" | "darkcyan" => Some(Color::DarkCyan),
        _ => None,
    }
}

// Example usage:
// "Hello World"$[bold red bg blue] - bold red text on blue background
// "Text"$[bright_green] - bright green text
// "Warning"$[yellow bg dark_red] - yellow text on dark red background
// "Info"$[cyan underline] - underlined cyan text
// "Error"$[bright_red bold] - bold bright red text
