use crate::types::Selection;
use regex::Regex;
use serde_json::Value;

// get the trimmed text of the match with a default of an empty string
// if the group didn't participate in the match.
fn get_selector(capture: &regex::Captures<'_>) -> String {
    let cap = capture.get(0).map_or("", |m| m.as_str()).trim();
    if cap.starts_with('\"') {
        let cap_string = String::from(cap);
        // Drop the enclosing double quotes in this case.
        let inner_cap = &cap_string[1..cap_string.len() - 1];
        String::from(inner_cap)
    } else {
        String::from(cap)
    }
}

pub fn walker(json: &Value, selector: Option<&str>) -> Option<Selection> {
    let mut inner_json = json;
    if let Some(selector) = selector {
        // Capture groups of double quoted selectors and simple ones surrounded
        // by dots.
        let re = Regex::new(r#"("[^"]+")|([^.]+)"#).unwrap();
        let selector: Vec<String> = re
            .captures_iter(selector)
            .map(|capture| get_selector(&capture))
            .collect();

        // Returns Result of values or Err early on, stopping the iteration.
        let items: Selection = selector
            .iter()
            .enumerate()
            .map(|(i, s)| -> Result<Value, String> {
                // array case
                if let Ok(index) = s.parse::<isize>() {
                    // negative index
                    if index.is_negative() {
                        Err(String::from("Invalid negative array index"))
                    } else {
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
                                            ") is out of bound, root elment \
                                             has a length of",
                                            &array.len().to_string(),
                                        ]
                                        .join(" ")
                                    } else {
                                        [
                                            "Index (",
                                            s,
                                            ") is out of bound, node (",
                                            selector[i - 1].as_str(),
                                            ") has a length of",
                                            &(array.len()).to_string(),
                                        ]
                                        .join(" ")
                                    }
                                }
                                // Trying to acces an index on a node which
                                // is not an arrya.
                                None => {
                                    if selector.len() == 1 {
                                        ["Root element is not an array"]
                                            .join(" ")
                                    } else {
                                        [
                                            "Node (",
                                            selector[i - 1].as_str(),
                                            ") is not an array",
                                        ]
                                        .join(" ")
                                    }
                                }
                            };
                            Err(error_message)
                        } else {
                            // match found.
                            inner_json = &inner_json[index as usize];
                            Ok(inner_json.clone())
                        }
                    }
                } else {
                    // an unterminated selector has been provided.
                    if s.is_empty() {
                        Err(String::from("Unterminated selector found"))
                    } else {
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
                                    selector[i - 1].as_str(),
                                    ")",
                                ]
                                .join(" "))
                            }
                        } else {
                            inner_json = &inner_json[s];
                            Ok(inner_json.clone())
                        }
                    }
                }
            })
            .collect();
        Some(items)
    } else {
        None
    }
}
