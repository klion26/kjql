use crate::array_walker::array_walker;
use crate::range_selector::range_selector;
use crate::types::{Selection, Selector, Selectors};
use crate::utils::display_node_or_range;
use serde_json::Value;

// returns a selection based on selectors and some JSON content.
pub fn get_selections(selectors: &Selectors, json: &Value) -> Selection {
    println!("=In get_selectiosn= {:?}", selectors);
    // local copy of the origin json that will be reused in the loop.
    let mut inner_json = json.clone();
    selectors
        .iter()
        .enumerate()
        .map(|(map_index, current_selector)| -> Result<Value, String> {
            match current_selector {
                // Default selector
                Selector::Default(raw_selector) => {
                    // No JSON value has been found (no array).
                    if inner_json.get(raw_selector).is_none() {
                        if map_index == 0 {
                            Err([
                                "Node \"",
                                raw_selector,
                                "\" not found on the parent element",
                            ]
                            .join(""))
                        } else {
                            Err([
                                "Node \"",
                                raw_selector,
                                "\" not found on parent ",
                                &display_node_or_range(
                                    &selectors[map_index - 1],
                                    false,
                                ),
                            ]
                            .join(""))
                        }
                    } else {
                        inner_json = inner_json[raw_selector].clone();
                        eprintln!(
                            "### debug get_selections:map:res for [{:?}] ###  \
                             {:?}",
                            raw_selector, inner_json
                        );
                        Ok(inner_json.clone())
                    }
                }

                // range selector
                Selector::Range((start, end)) => match range_selector(
                    map_index,
                    &inner_json.clone(),
                    *start,
                    *end,
                    &selectors,
                    if map_index == 0 {
                        None
                    } else {
                        Some(&selectors[map_index - 1])
                    },
                ) {
                    Ok(json) => {
                        inner_json = json.clone();
                        Ok(json.clone())
                    }
                    Err(error) => Err(error),
                },

                // Index selector
                Selector::Index(array_index) => match array_walker(
                    map_index,
                    *array_index,
                    &inner_json.clone(),
                    &selectors,
                ) {
                    Ok(json) => {
                        inner_json = json.clone();
                        Ok(json.clone())
                    }
                    Err(error) => Err(error),
                },
            }
        })
        .collect()
}
