use crate::types::Selector;
use crate::utils::display_node_or_range;
use serde_json::Value;

// walks through a JSON array
pub fn array_walker(
    map_index: usize,
    array_index: usize,
    inner_json: &Value,
    selector: &[Selector],
) -> Result<Value, String> {
    // No JSON value has been found (array).
    if inner_json.get(array_index).is_none() {
        let error_message = match inner_json.as_array() {
            // Trying to access an out of bound index on a
            // node
            // or on the root element.
            Some(array) => {
                if selector.len() == 1 {
                    [
                        "Index [",
                        array_index.to_string().as_str(),
                        "] is out of bound, root element has a length of ",
                        &array.len().to_string(),
                    ]
                    .join("")
                } else {
                    println!("=====>");
                    [
                        "Index [",
                        array_index.to_string().as_str(),
                        "] is out of bound, ",
                        &display_node_or_range(&selector[map_index - 1], false),
                        " has a length of ",
                        &array.len().to_string(),
                    ]
                    .join("")
                }
            }
            // Trying to acces an index on a node which
            // is not an arrya.
            None => {
                if selector.len() == 1 || map_index == 0 {
                    String::from("Root element is not an array")
                } else {
                    [
                        &display_node_or_range(&selector[map_index - 1], true),
                        " is not an array",
                    ]
                    .join("")
                }
            }
        };
        return Err(error_message);
    }

    Ok(inner_json.get(array_index).unwrap().clone())
}
