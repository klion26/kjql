use crate::types::Selection;
use serde_json::Value;

pub fn walker(json: &Value, selector: Option<&str>) -> Option<Selection> {
    let mut inner_json = json;
    if let Some(selector) = selector {
        let selector: Vec<&str> = selector.split('.').collect();
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
                                Some(array) => [
                                    "Index (",
                                    s,
                                    ") is out of bound, node (",
                                    selector[i - 1],
                                    ") has a length of",
                                    &(array.len()).to_string(),
                                ]
                                .join(" "),
                                // Trying to acces an index on a node which
                                // is not an arrya.
                                None => {
                                    if selector.len() == 1 {
                                        ["Root element is not an array"].join(" ")
                                    } else {
                                        [
                                            "Node (",
                                            selector[i - 1],
                                            ") is not an array",
                                        ]
                                            .join(" ")
                                    }
                                },
                            };
                            return Err(error_message);
                        } else {
                            // match found.
                            inner_json = &inner_json[index as usize];
                            Ok(inner_json.clone())
                        }
                    }
                } else {
                    // an unterminated selector has been provided.
                    if s.is_empty() {
                        return Err(String::from("Unterminated selector found"))
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
                                    selector[i - 1],
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
