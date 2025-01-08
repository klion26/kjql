use thiserror::Error;

fn display_content(content: &str) -> String {
    if content.is_empty() {
        String::new()
    } else {
        format!(" after {content}")
    }
}

/// Error type returned by the parser.
#[derive(Debug, Error, PartialEq)]
pub enum KjqlParserError {
    /// Empty input error.
    #[error("Empty input")]
    EmptyInputError,

    /// Parser error.
    #[error("Unable to parse input {unparsed}{}", display_content(tokens))]
    ParsingError {
        /// Tokens found while parsing.
        tokens: String,
        /// Unparsed content.
        unparsed: String,
    },

    /// Truncate error.
    #[error("Truncate operator found as non last element or multiple times in {0}")]
    TruncateError(String),

    /// Unknow error.
    #[error("Unknown error")]
    UnknownError,
}

#[cfg(test)]
mod tests {
    use super::display_content;

    #[test]
    fn check_display_content() {
        assert_eq!(display_content("some"), " after some");
        assert_eq!(display_content(""), "");
    }
}
