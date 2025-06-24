use colored::{Color, Colorize, Style};
use figura::Token;

#[derive(Debug)]
pub struct StyleDirective {
    color: Color,
    style: Style,
}

impl figura::Directive for StyleDirective {
    fn execute(&self, ctx: &figura::Context) -> Result<String, figura::TemplateError> {
        Ok(String::from("dfd"))
    }
}

pub struct CustomParser;

impl CustomParser {
    fn extract_slices(tokens: &[Token]) -> Option<Vec<&str>> {
        // Find the position of '[' and ']'
        let start = tokens.iter().position(|t| *t == Token::Symbol('['))?;
        let end = tokens.iter().position(|t| *t == Token::Symbol(']'))?;

        if end <= start + 1 {
            return Some(vec![]); // empty list
        }

        let mut slices = Vec::new();
        let mut i = start + 1;

        while i < end {
            match &tokens[i] {
                Token::Slice(s) => {
                    slices.push(s.as_str());
                    i += 1;

                    if i < end {
                        // Expect a comma if not at the end
                        match &tokens[i] {
                            Token::Symbol(',') => i += 1,
                            _ => return None, // Invalid format
                        }
                    }
                }

                _ => return None, // Invalid format
            }
        }

        Some(slices)
    }
}

impl figura::Parser for CustomParser {
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn figura::Directive>> {
        // template: "{varname$[]}"

        if let [
            Token::Slice(varname),
            Token::Symbol('$'),
            Token::Symbol('['),
            ..,
        ] = tokens
        {
            if let Some(slices) = Self::extract_slices(tokens) {
                for s in slices {
                    match s.to_lowercase().as_str() {
                        "bold" => varname.bold(),
                        "dimmed" => varname.dimmed(),
                        _ => {}
                    }
                }
            }
        }

        None
    }
}
