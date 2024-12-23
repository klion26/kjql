use crate::types::Selector;

// convert an array selector to a readable string
pub fn display_array_selector(capitalized: bool) -> String {
    String::from(if capitalized { "Array" } else { "array" })
}

// convert a range to a readable string.
pub fn display_range_selector(
    (start, end): (usize, usize),
    capitalized: bool,
) -> String {
    [
        if capitalized { "Range [" } else { "range [" },
        start.to_string().as_str(),
        ":",
        end.to_string().as_str(),
        "]",
    ]
    .join("")
}
// convert a range to a readable string.
pub fn display_default_selector(value: &str, capitalized: bool) -> String {
    [
        if capitalized {
            r#"Node ""#
        } else {
            r#"node ""#
        },
        value,
        r#"""#,
    ]
    .join("")
}

pub fn display_index_selector(index: usize, capitalized: bool) -> String {
    [
        if capitalized { "Index [" } else { "index [" },
        index.to_string().as_str(),
        "]",
    ]
    .join("")
}
