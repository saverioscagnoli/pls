use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlsError {
    /// Error returned when trying to replace a template,
    /// but there's no value with that name
    ///
    /// # Example
    /// ```no_run
    /// let values = HashMap::new(); // Empty
    /// let template = "Hello, my name is {name}";
    ///
    /// format_template(template, &values); // -> Returns NoValueForTemplate("name")
    /// ```
    NoValueForTemplate(String),

    /// Error returned when trying to parse a template,
    /// but a there's an open bracket without a corresponding closed one
    ///
    /// # Example
    /// ```no_run
    /// "Hello I am {name and I am {age}" // -> Returns NoClosedBracket
    /// ```
    NoClosedBracket,

    InvalidConditionalFormat,
}

impl Display for PlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlsError::NoValueForTemplate(val) => {
                write!(f, "There is no value for template '{}'", val)
            }

            PlsError::NoClosedBracket => {
                write!(
                    f,
                    "Encountered an open bracket without a corresponding closed one."
                )
            }

            PlsError::InvalidConditionalFormat => {
                write!(f, "Encountered invalid format for a conditional operation")
            }
        }
    }
}

impl std::error::Error for PlsError {}
