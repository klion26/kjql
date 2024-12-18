use crate::get_selection::get_selections;
use crate::types::{Selection, Selector};
use serde_json::{json, Value};

// apply the filter selectors to a JSON value and
// returns a selection.
pub fn apply_filter(
    json: &Value,
    filter_selectors: &Option<Vec<Selector>>,
) -> Selection {
    // Apply the filter iff the provided JSON is an array.
    match json.as_array() {
        Some(array) => {
            println!(
                "### debug ### apply filter {:?}, json:{:?}",
                filter_selectors, json
            );
            let selections: Vec<Selection> = array
                .iter()
                .cloned()
                .map(|partial_json| -> Selection {
                    match filter_selectors {
                        Some(selectors) => {
                            println!(
                                "### debug ### get selection for [{:?}], \
                                 [{:?}]",
                                filter_selectors, partial_json
                            );
                            get_selections(&selectors, &partial_json)
                        }
                        None => Ok(vec![partial_json]),
                    }
                })
                .collect();

            // try to find the first error.
            match selections
                .iter()
                .find_map(|selection| selection.clone().err())
            {
                // throw it back.
                Some(error) => Err(error),
                // no error in this case, we can safely unwrap.
                // TODO: Need to figure out this logci here
                None => Ok(vec![json!(selections
                    .iter()
                    .map(|selection| selection
                        .clone()
                        .unwrap()
                        .last()
                        .unwrap()
                        .clone())
                    .collect::<Vec<Value>>())]),
            }
        }
        None => Ok(vec![json.clone()]),
    }
}
