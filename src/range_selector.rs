use crate::types::Selector;
use crate::utils::display_node_or_range;
use serde_json::{json, Value};

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
