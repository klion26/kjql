use crate::types::{Selection, Selector, Selectors};
use crate::utils::display_node_or_range;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};
use std::string::String;

fn get_range_for_regex<'a>(
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

// get the trimmed text of the match with a default of an empty string
// if the group didn't participate in the match.
fn get_selector(capture: &str) -> Selector {
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
        let ranges: Vec<(&str, &str)> = get_range_for_regex(capture, &RANGE_REGEX);
        if ranges.is_empty() {
            println!("==== {}", String::from(capture));
            // Returns the initial captured value.
            Selector::Default(String::from(capture))
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

// returns a selection based on selectors and some JSON content.
fn get_selections(selectors: &Selectors, json: &Value) -> Selection {
    // local copy of the origin json that will be reused in the loop.
    let mut inner_json = json.clone();
    selectors
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
                            Err([
                                "Node (",
                                raw_selector,
                                ") is not the root element",
                            ]
                            .join(" "))
                        } else {
                            Err([
                                "Node (",
                                raw_selector,
                                ") not found on parent",
                                &display_node_or_range(
                                    &selectors[map_index - 1],
                                    false,
                                ),
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
        .collect()
}

// walks through a group
fn group_walker(
    capture: &regex::Captures<'_>,
    filter: Option<&str>,
    json: &Value,
) -> Selection {
    lazy_static! {
        static ref SUB_GROUP_REGEX: Regex =
            Regex::new(r#"("[^"]+")|([^.]+)"#).unwrap();
    }
    let group = capture.get(0).map_or("", |m| m.as_str().trim());

    println!("** {:?} **", filter);

    // empty group, return early
    if group.is_empty() {
        return Err(String::from("Empty group"));
    }

    // capture sub-groups of doulbe quoted selectors and simple ones surrounded
    // by dots.
    let selectors: Vec<Selector> = SUB_GROUP_REGEX
        .captures_iter(group)
        .map(|capture| get_selector(capture.get(0).map_or("", |m| m.as_str())))
        .collect();

    // perform the same operation on the filter.
    let filter_selectors = match filter {
        None => None,
        Some(filter)   => Some(
            SUB_GROUP_REGEX.captures_iter(filter)
                .map(|capture| {
                    get_selector(capture.get(0).map_or("", |m| m.as_str()))
                }).collect::<Vec<Selector>>()
            )
    };
    println!("filter_selector:{:?}", filter_selectors);
    // Returns a Result of values or an Err early on, stopping the iteration
    // as soon as the latter is encountered.
    let items: Selection = get_selections(&selectors, &json);
    println!("items:{:?}", items);

    // check for empty selection, in this case we assume that the user expects
    // to get back the complete raw JSON back for this group.
    match items {
        Ok(items) => {
            if items.is_empty() {
                println!("==== Empty items {:?}", apply_filter(&json, &filter_selectors));
                apply_filter(&json, &filter_selectors)
            } else {
                Ok(items
                    .iter()
                    .map(|item| apply_filter(&item, &filter_selectors).unwrap())
                    .flatten()
                    .collect::<Vec<Value>>())
            }
        }
        Err(error) => Err(error),
    }
}

// apply the filter selectors to a JSON value and
// returns a selection.
fn apply_filter(json: &Value, filter_selectors: &Option<Vec<Selector>>) -> Selection {
    // Apply the filter iff the provided JSON is an array.
    match json.as_array() {
        Some(array) => {
            let selections: Vec<Selection> = array
                .iter()
                .cloned()
                .map(|partial_json| -> Selection {
                    match filter_selectors {
                        Some(selectors) => get_selections(&selectors, &partial_json),
                        None => Ok(vec![partial_json])
                    }
                }).collect();

            // try to find the first error.
            match selections
                .iter()
                .find_map(|selection| selection.clone().err()) {
                // throw it back.
                Some(error) => Err(error),
                // no error in this case, we can safely unwrap.
                None => Ok(vec![json!(
                    selections
                    .iter()
                    .map(|selection| selection.clone().unwrap())
                    .flatten()
                    .collect::<Vec<Value>>()
                )]),
            }
        }
        None => Ok(vec![json.clone()]),
    }
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
