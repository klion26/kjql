use crate::group_walker::group_walker;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};
use std::string::String;

pub fn get_range_for_regex<'a>(
    cap: &'a str,
    reg: &Regex,
) -> Vec<(&'a str, &'a str)> {
    reg.captures_iter(cap)
        .map(|capture| {
            (
                capture.get(1).map_or("", |m| m.as_str()),
                capture.get(2).map_or("", |m| m.as_str()),
            )
        })
        .collect()
}

// give some selector walk over the JSON file.
pub fn walker(json: &Value, selector: Option<&str>) -> Result<Value, String> {
    if let Some(selector) = selector {
        lazy_static! {
            static ref FILTER_REGEX: Regex =
                Regex::new(r"^(.*)\|([^|]+)$").unwrap();
            static ref GROUP_REGEX: Regex = Regex::new(r"([^,]+)").unwrap();
        }

        let selection_with_filter: Vec<(&str, &str)> =
            get_range_for_regex(selector, &FILTER_REGEX);

        println!("selection_with_filter::{:?}", selection_with_filter);
        let selector_and_filter = if selection_with_filter.is_empty() {
            (selector, None)
        } else {
            (selection_with_filter[0].0, Some(selection_with_filter[0].1))
        };

        // capture groups separated by commas, apply the selector for the
        // curernt gorup and return a Result of values or an Err early on.
        let groups: Result<Vec<Value>, String> = GROUP_REGEX
            .captures_iter(selector_and_filter.0)
            .map(|capture| group_walker(&capture, selector_and_filter.1, json))
            .map(|s| -> Result<Value, String> {
                match s {
                    Ok(item) => Ok(item.last().unwrap().clone()),
                    Err(error) => Err(error.clone()),
                }
            })
            .collect();
        return match groups {
            Ok(groups) => match groups.len() {
                0 => Err(String::from("Empty selection")),
                1 => Ok(json!(groups[0])),
                _ => Ok(json!(groups)),
            },
            Err(error) => Err(error),
        };
    }
    Err(String::from("No selector found"))
}
