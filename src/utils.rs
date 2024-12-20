use crate::types::Selector;
use std::fs::File;
use std::io::prelude::Read;
use std::io::BufReader;
use toml::Value;

// convert a range to a readable string.
fn display_range_selector(
    (start, end): (usize, usize),
    capitalized: bool,
) -> String {
    [
        if capitalized { "Range (" } else { "range (" },
        start.to_string().as_str(),
        ":",
        end.to_string().as_str(),
        ")",
    ]
    .join(" ")
}
// convert a range to a readable string.
fn display_default_selector(value: &str, capitalized: bool) -> String {
    [if capitalized { "Node (" } else { "node (" }, value, ")"].join(" ")
}

// return the node or the range of Selector as a string.
pub fn display_node_or_range(selector: &Selector, capitalized: bool) -> String {
    match selector {
        Selector::Default(value) => {
            display_default_selector(value, capitalized)
        }
        Selector::Range(range) => display_range_selector(*range, capitalized),
    }
}
