use crate::types::Selector;
use crate::utils::display_node_or_range;
use serde_json::Value;

// walks through a JSON array
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
    // No JSON value has been found (array).
    if inner_json.get(array_index as usize).is_none() {
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
                if selector.len() == 1 || map_index == 0 {
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
