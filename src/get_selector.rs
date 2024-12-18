use crate::core::get_range_for_regex;
use crate::types::Selector;
use lazy_static::lazy_static;
use regex::Regex;

// get the trimmed text of the match with a default of an empty string
// if the group didn't participate in the match.
pub fn get_selector(capture: &str) -> Selector {
    let capture = capture.trim();
    if capture.starts_with('\"') {
        // let cap_string = String::from(cap);
        // Drop the enclosing double quotes in this case.
        // let inner_cap = &cap_string[1..cap_string.len() - 1];
        Selector::Default(String::from(&capture[1..capture.len() - 1]))
    } else {
        // Array range, e.g. 0:3.
        lazy_static! {
            static ref RANGE_REGEX: Regex = Regex::new(r"(\d+):(\d+)").unwrap();
        }
        let ranges: Vec<(&str, &str)> =
            get_range_for_regex(capture, &RANGE_REGEX);
        if ranges.is_empty() {
            println!("==== {}", String::from(capture));
            // Returns the initial captured value.
            Selector::Default(String::from(capture))
        } else {
            // Returns the range as a tuple of the from (start, end).
            let (start, end) = ranges[0];
            Selector::Range((
                start.parse::<usize>().unwrap(),
                end.parse::<usize>().unwrap(),
            ))
        }
    }
}
