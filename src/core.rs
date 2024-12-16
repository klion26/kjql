use crate::types::{Selection, Selector};
use crate::utils::display_node_or_range;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};
use std::string::String;

// get the trimmed text of the match with a default of an empty string
// if the group didn't participate in the match.
fn get_selector(capture: &regex::Captures<'_>) -> Selector {
    let cap = capture.get(0).map_or("", |m| m.as_str()).trim();
    if cap.starts_with('\"') {
        // let cap_string = String::from(cap);
        // Drop the enclosing double quotes in this case.
        // let inner_cap = &cap_string[1..cap_string.len() - 1];
        Selector::Default(String::from(&cap[1..cap.len() - 1]))
    } else {
        // Array range, e.g. 0:3.
        lazy_static! {
            static ref RANGE_REGEX: Regex = Regex::new(r"(\d+):(\d+)").unwrap();
        }
        let ranges: Vec<(&str, &str)> = RANGE_REGEX
            .captures_iter(cap)
            .map(|capture| {
                (
                    capture.get(1).map_or("", |m| m.as_str()),
                    capture.get(2).map_or("", |m| m.as_str()),
                )
            })
            .collect();
        if ranges.is_empty() {
            println!("==== {}", String::from(cap));
            // Returns the initial captured value.
            Selector::Default(String::from(cap))
        } else {
            // Returns the range as a tuple of the from (start, end).
            let (start, end) = ranges[0];
            Selector::Range((
                usize::from_str_radix(start, 10).unwrap(),
                usize::from_str_radix(end, 10).unwrap(),
            ))
        }
    }
}

// walks through a group
fn group_walker(capture: &regex::Captures<'_>, json: &Value) -> Selection {
    lazy_static! {
        static ref SUB_GROUP_REGEX: Regex =
            Regex::new(r#"("[^"]+")|([^.]+)"#).unwrap();
    }
    let mut inner_json = json.clone();
    let group = capture.get(0).map_or("", |m| m.as_str().trim());

    if group.is_empty() {
        return Err(String::from("Empty group"))
    }

    // capture sub-groups of doulbe quoted selectors and simple ones surrounded
    // by dots.
    let selectors: Vec<Selector> = SUB_GROUP_REGEX
        .captures_iter(group)
        .map(|capture| get_selector(&capture))
        .collect();
    // Returns a Result of values or an Err early on, stopping the iteration
    // as soon as the latter is encountered.
    let items: Selection = selectors
        .iter()
        .enumerate()
        .map(|(map_index, current_selector)| -> Result<Value, String> {
            match current_selector {
                // Default selector
                Selector::Default(raw_selector) => {
                    // Array case.
                    if let Ok(array_index) = raw_selector.parse::<isize>() {
                        return match array_walker(
                            map_index,
                            array_index,
                            &inner_json.clone(),
                            raw_selector,
                            &selectors,
                        ) {
                            Ok(json) => {
                                inner_json = json.clone();
                                Ok(json.clone())
                            }
                            Err(error) => Err(error),
                        };
                    }

                    // A JSON null value has been found (non array).
                    if inner_json[raw_selector] == Value::Null {
                        if map_index == 0 {
                            Err(["Node (", raw_selector, ") is not the root element"]
                                .join(" "))
                        } else {
                            Err([
                                "Node (",
                                raw_selector,
                                ") not found on parent",
                                &display_node_or_range(&selectors[map_index - 1], false),
                            ]
                            .join(" "))
                        }
                    } else {
                        inner_json = inner_json[raw_selector].clone();
                        Ok(inner_json.clone())
                    }
                }

                // range selector
                Selector::Range((start, end)) => match range_selector(
                    map_index,
                    &inner_json.clone(),
                    *start,
                    *end,
                    &selectors,
                ) {
                    Ok(json) => {
                        inner_json = json.clone();
                        Ok(json.clone())
                    }
                    Err(error) => Err(error),
                },
            }
        })
        .collect();

    // check for empty selection, in this case we assume that the user expects
    // to get back the complete raw JSON back for this group.
    match items {
        Ok(items) => {
            if items.is_empty() {
                Ok(vec![json.clone()])
            } else {
                Ok(items)
            }
        }
        Err(error) => Err(error),
    }
}
// give some selector walk over the JSON file.
pub fn walker(json: &Value, selector: Option<&str>) -> Result<Value, String> {
    if let Some(selector) = selector {
        lazy_static! {
            static ref GROUP_REGEX: Regex = Regex::new(r"([^,]+)").unwrap();
        }
        // capture groups separated by commas, apply the selector for the
        // curernt gorup and return a Result of values or an Err early on.
        let groups: Result<Vec<Value>, String> = GROUP_REGEX
            .captures_iter(selector)
            .map(|capture| group_walker(&capture, json))
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

pub fn array_walker(
    map_index: usize,
    array_index: isize,
    inner_json: &Value,
    raw_selector: &str,
    selector: &[Selector],
) -> Result<Value, String> {
    if array_index.is_negative() {
        return Err(String::from("Invalid negative array index"));
    }
    // found a null value in the array
    if inner_json[array_index as usize] == Value::Null {
        let error_message = match inner_json.as_array() {
            // Trying to access an out of bound index on a
            // node
            // or on the root element.
            Some(array) => {
                if selector.len() == 1 {
                    [
                        "Index (",
                        raw_selector,
                        ") is out of bound, root elment has a length of",
                        &array.len().to_string(),
                    ]
                    .join(" ")
                } else {
                    [
                        "Index (",
                        raw_selector,
                        ") is out of bound,",
                        &display_node_or_range(&selector[map_index - 1], false),
                        "has a length of",
                        &array.len().to_string(),
                    ]
                    .join(" ")
                }
            }
            // Trying to acces an index on a node which
            // is not an arrya.
            None => {
                if selector.len() == 1 {
                    ["Root element is not an array"].join(" ")
                } else {
                    [
                        &display_node_or_range(&selector[map_index - 1], true),
                        "is not an array",
                    ]
                    .join(" ")
                }
            }
        };
        return Err(error_message);
    }

    Ok(inner_json[array_index as usize].clone())
}

pub fn range_selector(
    map_index: usize,
    inner_json: &Value,
    start: usize,
    end: usize,
    selectors: &[Selector],
) -> Result<Value, String> {
    let is_default = start < end;
    // check the range validity
    // if this is array
    if let Some(inner_arrar) = inner_json.as_array() {
        if inner_arrar.len() < start || inner_arrar.len() < (end + 1) {
            return Err(if selectors.len() == 1 {
                [
                    "Range (",
                    start.to_string().as_str(),
                    ", ",
                    end.to_string().as_str(),
                    ") is out of bound",
                    ", len:",
                    inner_arrar.len().to_string().as_str(),
                ]
                .join(" ")
            } else {
                [
                    "Range (",
                    start.to_string().as_str(),
                    ":",
                    end.to_string().as_str(),
                    ") is out of bound,",
                    &display_node_or_range(&selectors[map_index - 1], false),
                    "has a length of",
                    &(inner_arrar.len().to_string()),
                ]
                .join(" ")
            });
        }

        // what if start < 0 and end > len?
        Ok(if is_default {
            json!(inner_arrar[start..(end + 1)])
        } else {
            // Get the normalized slice selection, i.e from end to start.
            let normalized_range_selection =
                json!(inner_arrar[end..(start + 1)]);
            let reversed_range_selection: Vec<&Value> =
                normalized_range_selection
                    .as_array()
                    .unwrap()
                    .iter()
                    .rev()
                    .collect();
            json!(reversed_range_selection)
        })
    } else if selectors.len() == 1 {
        Err(["Root element is not an array"].join(" "))
    } else {
        Err([
            &display_node_or_range(&selectors[map_index - 1], true),
            ") is not an array",
        ]
        .join(" "))
    }
}
