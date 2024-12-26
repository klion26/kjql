use crate::types::{Display, Selections, Selectors};
use serde_json::{json, Value};

// walks through a JSON array. Iterate over the indexes of the array, returns
// a Result of values or an Err early on.
pub fn array_walker(
    array_index: &[usize],
    inner_json: &Value,
    map_index: usize,
    selector: &Selectors,
) -> Result<Value, String> {
    let results: Selections = array_index
        .iter()
        .map(|index| {
            // No JSON value has been found (array).
            if inner_json.get(index).is_none() {
                let error_message = match inner_json.as_array() {
                    // Trying to access an out of bound index on a
                    // node
                    // or on the root element.
                    Some(array) => {
                        if selector.len() == 1 {
                            [
                                "Index [",
                                index.to_string().as_str(),
                                "] is out of bound, root element has a length \
                                 of ",
                                &array.len().to_string(),
                            ]
                            .join("")
                        } else {
                            [
                                "Index [",
                                index.to_string().as_str(),
                                "] is out of bound, ",
                                &selector[map_index - 1].as_str(false),
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
                                &selector[map_index - 1].as_str(true),
                                " is not an array",
                            ]
                            .join("")
                        }
                    }
                };
                return Err(error_message);
            }

            Ok(inner_json.get(index).unwrap().clone())
        })
        .collect();

    match results {
        Ok(values) => Ok(if array_index.len() == 1 {
            values[0].clone()
        } else {
            json!(values)
        }),
        Err(error) => Err(error),
    }
}
