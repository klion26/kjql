use crate::group_walker::group_walker;
use crate::parser::selectors_parser;
use serde_json::{json, Value};
use std::string::String;

// give some selectors walk over the JSON file.
pub fn walker(json: &Value, selectors: Option<&str>) -> Result<Value, String> {
    if let Some(selectors) = selectors {
        return match selectors_parser(selectors) {
            Ok(groups) => {
                // Capture groups separated by commas apply the selector for the
                // current group and return a Result of values or an Err early
                // on.
                let inner_groups: Result<Vec<Value>, String> = groups
                    .iter()
                    .map(|group| group_walker(group, json))
                    .collect();
                match inner_groups {
                    Ok(groups) => match groups.len() {
                        0 => Err(String::from("Empty selection")),
                        1 => Ok(json!(groups[0])),
                        _ => Ok(json!(groups)),
                    },
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        };
    }
    Err(String::from("No selector found"))
}
