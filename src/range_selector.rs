use crate::types::Selector;
use crate::utils::{
    display_default_selector, display_index_selector, display_node_or_range,
    display_range_selector,
};
use serde_json::{json, Value};

pub fn range_selector(
    map_index: usize,
    inner_json: &Value,
    start: usize,
    end: usize,
    selectors: &[Selector],
    previous_selector: Option<&Selector>,
) -> Result<Value, String> {
    let is_default = start < end;

    match inner_json.as_array() {
        Some(json_array) => {
            println!("=In range_selector= {:?}", json_array);
            if json_array.len() < start || json_array.len() < (end + 1) {
                return Err(if selectors.len() == 1 {
                    [
                        "Range [",
                        start.to_string().as_str(),
                        ", ",
                        end.to_string().as_str(),
                        "] is out of bound, len: ",
                        json_array.len().to_string().as_str(),
                    ]
                    .join("")
                } else {
                    [
                        "Range [",
                        start.to_string().as_str(),
                        ":",
                        end.to_string().as_str(),
                        "] is out of bound, ",
                        &display_node_or_range(
                            &selectors[map_index - 1],
                            false,
                        ),
                        " has a length of ",
                        &(json_array.len().to_string()),
                    ]
                    .join("")
                });
            }

            // what if start < 0 and end > len?
            Ok(if is_default {
                json!(json_array[start..=end])
            } else {
                // Get the normalized slice selection, i.e from end to start.
                let normalized_range_selection = json!(json_array[end..=start]);
                let reversed_range_selection: Vec<&Value> =
                    normalized_range_selection
                        .as_array()
                        .unwrap()
                        .iter()
                        .rev()
                        .collect();
                json!(reversed_range_selection)
            })
        }
        None => Err([
            (match previous_selector {
                Some(selector) => match selector {
                    Selector::Default(node) => {
                        display_default_selector(node, true)
                    }
                    Selector::Range(range) => {
                        display_range_selector(*range, true)
                    }
                    Selector::Index(index) => {
                        display_index_selector(*index, true)
                    }
                },
                None => String::from("Root element"),
            })
            .as_str(),
            " is not an array",
        ]
        .join("")),
    }
}
