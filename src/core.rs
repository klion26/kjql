use crate::types::{Selection, Selector};
use regex::Regex;
use serde_json::{json, Value};
use std::string::String;
use crate::utils::display_node_or_range;

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
        let range_regex = Regex::new(r"(\d+):(\d+)").unwrap();
        let ranges: Vec<(&str, &str)> = range_regex
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

// TODO: extract error messages to a separate function.
pub fn walker(json: &Value, selector: Option<&str>) -> Option<Selection> {
    let mut inner_json = json.clone();
    if let Some(selector) = selector {
        // Capture groups of double quoted selectors and simple ones surrounded
        // by dots.
        let re = Regex::new(r#"("[^"]+")|([^.]+)"#).unwrap();
        let selector: Vec<Selector> = re
            .captures_iter(selector)
            .map(|capture| get_selector(&capture))
            .collect();

        if selector.is_empty() {
            return Some(Err("Unterminated selector found".to_string()))
        }

        // Returns Result of values or Err early on, stopping the iteration.
        let items: Selection = selector
            .iter()
            .enumerate()
            .map(|(i, s)| -> Result<Value, String> {
                println!("*** {:?} ***", s);
                match s {
                    // default selector
                    Selector::Default(s) => {
                        // array case
                        if let Ok(index) = s.parse::<isize>() {
                            return match array_walker(
                                i,
                                index,
                                &inner_json.clone(),
                                s,
                                &selector,
                            ) {
                                Ok(json) => {
                                    inner_json = json.clone();
                                    Ok(json.clone())
                                }
                                Err(error) => Err(error),
                            };
                        }

                        println!("- {} {}", inner_json, s);
                        // found a null value in the object
                        if inner_json[s] == Value::Null {
                            if i == 0 {
                                Err(["Node (", s, ") is not the root element"]
                                    .join(" "))
                            } else {
                                Err([
                                    "Node (",
                                    s,
                                    ") not found on parent (",
                                    match &selector[i - 1] {
                                        Selector::Default(value) => {
                                            value.as_str()
                                        }
                                        Selector::Range(range) => "0:3",
                                    },
                                    ")",
                                ]
                                    .join(" "))
                            }
                        } else {
                            inner_json = inner_json[s].clone();
                            Ok(inner_json.clone())
                        }
                    }
                    // range selector
                    Selector::Range((start, end)) => match
                    range_selector(
                        i,
                        &inner_json.clone(),
                        *start,
                        *end,
                        &selector,
                    ) {
                        Ok(json) => {
                            inner_json = json.clone();
                            Ok(json.clone())
                        }
                        Err(error) => Err(error),
                    }
                }
            })
            .collect();
        Some(items)
    } else {
        None
    }
}

pub fn array_walker(
    i: usize,
    index: isize,
    inner_json: &Value,
    s: &str,
    selector: &[Selector],
) -> Result<Value, String> {
    if index.is_negative() {
        return Err(String::from("Invalid negative array index"));
    }
    // found a null value in the array
    if inner_json[index as usize] == Value::Null {
        let error_message = match inner_json.as_array() {
            // Trying to access an out of bound index on a
            // node
            // or on the root element.
            Some(array) => {
                if selector.len() == 1 {
                    [
                        "Index (",
                        s,
                        ") is out of bound, root elment has a length of",
                        &array.len().to_string(),
                    ]
                        .join(" ")
                } else {
                    [
                        "Index (",
                        s,
                        ") is out of bound,",
                        &display_node_or_range(&selector[i - 1], false),
                        "has a length of",
                        &(array.len()).to_string(),
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
                        &display_node_or_range(&selector[i - 1], true),
                        "is not an array",
                    ]
                        .join(" ")
                }
            }
        };
        return Err(error_message);
    }

    println!("FIx it {}", &inner_json[index as usize]);
    Ok(inner_json[index as usize].clone())
}

pub fn range_selector(
    i: usize,
    inner_json: &Value,
    start: usize,
    end: usize,
    selector: &[Selector],
) -> Result<Value, String> {
    let is_default = start < end;
    // check the range validity
    // if this is array
    if let Some(inner_arrar) = inner_json.as_array() {
        if inner_arrar.len() < start || inner_arrar.len() < (end + 1) {
            return Err(if selector.len() == 1 {
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
                    &display_node_or_range(&selector[i - 1], false),
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
    } else {
        if selector.len() == 1 {
            return Err(["Root element is not an array"].join(" "));
        } else {
            return Err([
                &display_node_or_range(&selector[i - 1], true),
                ") is not an array",
            ]
                .join(" "));
        }
    }
}
