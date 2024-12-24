use crate::get_selection::get_selections;
use crate::types::{ExtendedSelection, MaybeArray, Selection, Selector};
use rayon::prelude::*;
use serde_json::{json, Value};

// apply the filter selectors to a JSON value and
// returns a selection.
pub fn apply_filter(
    json: &Value,
    filter_selectors: &[Selector],
) -> ExtendedSelection {
    // Apply the filter iff the provided JSON is an array.
    match json.as_array() {
        Some(array) => {
            let selections: Vec<Selection> = array
                .par_iter()
                .cloned()
                .map(|partial_json| -> Selection {
                    if filter_selectors.is_empty() {
                        Ok(vec![partial_json])
                    } else {
                        get_selections(&filter_selectors, &partial_json)
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
                // no error in this case, we can safely unwrap.e
                None => Ok(MaybeArray::Array(selections.iter().fold(
                    Vec::new(),
                    |mut acc: Vec<Value>, selection| {
                        acc.push(json!(selection
                            .clone()
                            .unwrap()
                            .last()
                            .unwrap()
                            .clone()));
                        acc
                    },
                ))),
            }
        }
        // Not an array, return the JSON content if there's no filter or throw
        // an error.
        None => {
            if filter_selectors.is_empty() {
                Ok(MaybeArray::NonArray(vec![json.clone()]))
            } else {
                Err(String::from("A filter can only be applied to an array"))
            }
        }
    }
}
